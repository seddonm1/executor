use crate::guest::{self, component::workflow::abi::HostToGuest};
use anyhow::Result;
use rand::{thread_rng, SeedableRng};
use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use uuid::Uuid;
use wasmtime::{
    component::{Component, Linker},
    Engine, Store,
};

pub struct Workflow {
    engine: Engine,
    state: State,
    binary: Vec<u8>,
}

#[allow(dead_code)]
impl Workflow {
    pub fn new(engine: &Engine, binary: &Vec<u8>) -> Self {
        Self {
            engine: engine.to_owned(),
            state: State::default(),
            binary: binary.to_owned(),
        }
    }

    pub fn with_state(&mut self, state: State) -> &mut Self {
        self.state = state;
        self
    }

    pub async fn execute(&mut self) -> Result<()> {
        self.state.start_execution();

        let mut store = Store::new(&self.engine, self.state.clone());
        store.set_fuel(u64::MAX)?;
        store.fuel_async_yield_interval(Some(10000))?;

        let mut linker = Linker::new(&self.engine);
        guest::Workflow::add_to_linker(&mut linker, |state: &mut State| state)?;
        let component = Component::from_binary(&self.engine, &self.binary)?;
        let workflow = guest::Workflow::instantiate_async(&mut store, &component, &linker).await?;

        Ok(workflow
            .call_execute(&mut store)
            .await?
            .inspect(|_| self.state = store.data().clone())
            .inspect_err(|err| {
                if let Some(id) = &err.id {
                    store.data_mut().set_failure(id);
                    self.state = store.data().clone()
                }
            })?)
    }
}

/// Represents the state of a workflow execution.
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct State {
    /// Unique identifier for the state.
    pub id: Uuid,
    /// Time when the state was created.
    created: SystemTime,
    /// List of executions associated with this state.
    executions: Vec<Execution>,
    /// Random number generator.
    pub rng: Arc<Mutex<::rand::rngs::StdRng>>,
}

impl Default for State {
    /// Creates a new `State` with default values.
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            created: SystemTime::now(),
            executions: vec![],
            rng: Arc::new(Mutex::new(
                ::rand::rngs::StdRng::from_rng(thread_rng()).expect("rand should not fail"),
            )),
        }
    }
}

#[allow(dead_code)]
impl State {
    pub fn with_rng(mut self, rng: ::rand::rngs::StdRng) -> Self {
        self.rng = Arc::new(Mutex::new(rng));
        self
    }

    /// Starts a new execution and returns a new `State` with the added execution.
    pub fn start_execution(&mut self) {
        if let Some(execution) = self.executions.last() {
            self.executions.push(Execution::new(&execution.log))
        } else {
            self.executions.push(Execution::new(&[]))
        };
    }
}

/// Represents a single execution within a workflow state.
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Execution {
    /// Unique identifier for the execution.
    id: Uuid,
    /// Time when the execution was created.
    created: SystemTime,
    /// Current position in the execution log.
    position: usize,
    /// Log of messages for this execution.
    log: Vec<LogMessage>,
}

impl Execution {
    /// Creates a new `Execution` with the given log.
    fn new(log: &[LogMessage]) -> Self {
        Self {
            id: Uuid::new_v4(),
            created: SystemTime::now(),
            position: 0,
            log: log.to_owned(),
        }
    }
}

/// Represents a log message within an execution.
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct LogMessage {
    /// Time when the log message was created.
    created: SystemTime,
    /// Indicates whether the operation was successful.
    success: bool,
    /// The actual message content.
    message: HostToGuest,
}

impl LogMessage {
    /// Creates a new `LogMessage` with the given success status and message.
    fn new(success: bool, message: HostToGuest) -> Self {
        Self {
            created: SystemTime::now(),
            success,
            message,
        }
    }
}

impl State {
    /// Retrieves a message from the current execution log or generates a new one using the provided function.
    pub async fn retrieve_or_else<F, T>(&mut self, f: F) -> Result<HostToGuest>
    where
        F: Fn() -> T,
        T: Future<Output = Result<HostToGuest>>,
    {
        let execution = self.execution();

        let message = match execution.log.get_mut(execution.position) {
            Some(log_message) => {
                if log_message.success {
                    log_message.message.to_owned()
                } else {
                    *log_message = LogMessage::new(true, f().await?);
                    log_message.message.to_owned()
                }
            }
            None => {
                let log_message = LogMessage::new(true, f().await?);
                execution.log.push(log_message.to_owned());
                log_message.message
            }
        };

        execution.position += 1;

        Ok(message)
    }

    /// Marks all log messages with the given ID as failed in the last execution.
    pub fn set_failure(&mut self, id: &str) {
        if let Some(execution) = self.executions.last_mut() {
            execution.log.iter_mut().for_each(|log_message| {
                if log_message.message.id == id {
                    log_message.success = false;
                }
            });
        }
    }

    /// Returns a mutable reference to the current execution in the state.
    fn execution(&mut self) -> &mut Execution {
        self.executions.last_mut().unwrap()
    }
}
