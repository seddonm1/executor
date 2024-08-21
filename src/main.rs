mod guest;
mod http;
mod logger;
mod rand;
mod time;
mod workflow;

#[cfg(test)]
mod test;

use anyhow::Result;
use wasmtime::{self, Config, Engine};

#[tokio::main]
async fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("USAGE: demo WASM");
    let binary = std::fs::read(path)?;

    // Enable component model (which isn't supported by default)
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    config.consume_fuel(true);

    // Create a wasmtime execution context
    let engine = Engine::new(&config)?;

    let mut workflow = workflow::Workflow::new(&engine, &binary);
    workflow.execute().await?;

    Ok(())
}
