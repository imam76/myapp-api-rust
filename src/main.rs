//! The main entry point for the application binary.
//!
//! This file is responsible for setting up the Tokio runtime and starting the
//! application server by calling the `run` function from the `myapp_api_rust` library crate.
//! Keeping `main.rs` minimal allows the core application logic to reside in the library,
//! which makes it easier to test and reuse.

use myapp_api_rust::run;

/// The asynchronous main function.
///
/// It initializes the Tokio runtime using the `#[tokio::main]` macro and
/// awaits the `run` function, which contains the application's primary logic.
#[tokio::main]
async fn main() {
  run().await;
}
