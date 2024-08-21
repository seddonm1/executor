# cargo component build --release --package workflow --target wasm32-unknown-unknown
cargo component build --release --package workflow-example --target wasm32-unknown-unknown
cargo run --release target/wasm32-unknown-unknown/release/workflow_example.wasm
