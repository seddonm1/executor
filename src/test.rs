use crate::workflow::{self, State};
use anyhow::Result;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use rand::{Rng, SeedableRng};
use std::time::Duration;
use wasmtime::{Config, Engine};

#[tokio::test(flavor = "multi_thread")]
async fn sim() -> Result<()> {
    let (close_tx, close_rx) = tokio::sync::oneshot::channel();
    let server_handle = tokio::spawn(async {
        axum::serve(
            tokio::net::TcpListener::bind("127.0.0.1:3000")
                .await
                .unwrap(),
            Router::new()
                .route(
                    "/iss/now",
                    get(|| async {
                        if rand::thread_rng().gen::<bool>() {
                            Ok("{\"message\": \"success\", \"iss_position\": {\"latitude\": \"0.9969\", \"longitude\": \"-120.6400\"}, \"timestamp\": 1725167813}")
                        } else {
                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }),
                )
                .route(
                    "/email/send",
                    post(|| async {
                        if rand::thread_rng().gen::<bool>() {
                            Ok(format!("Email sent! Receipt: {}", uuid::Uuid::new_v4() )  )
                        } else {
                            Err(StatusCode::FORBIDDEN)
                        }
                    }),
                )
            .route(
                "/database/update",
                post(|| async {
                    if rand::thread_rng().gen::<bool>() {
                        Ok("1 row updated!")
                    } else {
                        Err(StatusCode::INSUFFICIENT_STORAGE)
                    }
                }),
            ),
        )
        .with_graceful_shutdown(async move {
            _ = close_rx.await;
        })
        .await
        .unwrap();
    });

    // Enable component model (which isn't supported by default)
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    config.consume_fuel(true);

    // Create a wasmtime execution context
    let engine = Engine::new(&config)?;
    let binary = std::fs::read("target/wasm32-unknown-unknown/release/workflow_example.wasm")?;

    for i in 0..10 {
        println!("\nstart seed {i}");
        let mut workflow = workflow::Workflow::new(&engine, &binary);
        workflow.with_state(State::default().with_rng(::rand::rngs::StdRng::seed_from_u64(i)));

        loop {
            match workflow.execute().await {
                Ok(_) => break,
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
            }
        }
    }

    _ = close_tx.send(());
    _ = server_handle.await;

    Ok(())
}
