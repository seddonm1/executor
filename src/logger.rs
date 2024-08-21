use crate::{
    guest::{
        component::workflow::abi::{Content, Level},
        GuestToHost, HostToGuest,
    },
    workflow::State,
};
use anyhow::Result;
use uuid::Uuid;

pub async fn call(state: &mut State, request: GuestToHost) -> Result<HostToGuest> {
    match request {
        GuestToHost::Log(level) => {
            match level {
                Level::Trace(msg) => println!("{} Trace: {}", state.id, msg),
                Level::Debug(msg) => println!("{} Debug: {}", state.id, msg),
                Level::Info(msg) => println!("{} Info: {}", state.id, msg),
                Level::Warn(msg) => println!("{} Warn: {}", state.id, msg),
                Level::Error(msg) => println!("{} Error: {}", state.id, msg),
            };

            Ok(HostToGuest {
                id: Uuid::new_v4().into(),
                content: Content::Unit,
            })
        }
        _ => unreachable!(),
    }
}
