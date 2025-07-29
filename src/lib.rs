//! # My App API
//!
//! This crate provides the core application logic for the My App API.
//! It is structured as a library to allow for easy integration testing and separation of concerns.
//!
//! The main components are:
//! - `app()`: Builds the Axum router and defines the application's routes.
//! - `setup_state()`: Initializes the application state, including the database connection pool.
//! - `run()`: Starts the web server.
//!
//! The application follows a modular structure, with features like contacts, errors, and state
//! management organized into their respective modules.

use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::{Level, info};

use crate::modules::datastores::contacts::contact_repository::SqlxContactRepository;

pub mod errors;
pub mod modules;
pub mod responses;
pub mod state;

pub use errors::AppError;
pub use state::AppState;

/// A convenient `Result` type alias for the application.
///
/// This wraps the standard `Result` type and uses `AppError` as the default error type.
/// This helps to reduce boilerplate code in function signatures throughout the application.
pub type AppResult<T> = Result<T, AppError>;

/// Configures and builds the main Axum `Router`.
///
/// This function sets up the application's routes, distinguishing between public and private endpoints.
/// It takes an `AppState` as an argument, which is then shared across all handlers.
///
/// # Arguments
///
/// * `app_state` - The shared application state, containing the database pool and other resources.
///
/// # Returns
///
/// * `Router` - The configured Axum router, ready to be served.
pub fn app(app_state: AppState) -> Router {
  let public_routes = Router::new().route("/", get(|| async { "🚀 Welcome to the My Rust Base API!" }));

  let private_routes = Router::new()
    // Add your private routes here, e.g.:
    .nest("/api/v1/contacts", modules::datastores::contacts::contact_routes::router());

  Router::new()
    .merge(public_routes) // Public routes without auth
    .merge(private_routes) // Private routes with auth
    .with_state(app_state)
    .fallback(modules::method_not_allowed_handler::fallback)
}

/// Initializes the shared `AppState`.
///
/// This asynchronous function is responsible for setting up the application's initial state.
/// It performs the following key tasks:
/// 1. Loads environment variables from a `.env` file.
/// 2. Establishes a connection pool to the PostgreSQL database.
/// 3. Creates and returns an `AppState` instance containing the database pool and initialized repositories.
///
/// # Panics
///
/// This function will panic if:
/// - The `DATABASE_URL` environment variable is not set.
/// - It fails to connect to the database.
pub async fn setup_state() -> AppState {
  dotenvy::dotenv().ok();
  let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

  let db_pool = PgPoolOptions::new()
    .max_connections(10)
    .min_connections(1)
    .connect(&db_url)
    .await
    .expect("Failed to connect to the database");
  info!("✅ Connected to database {}", db_url);

  AppState {
    db: db_pool.clone(),
    contact_repository: Arc::new(SqlxContactRepository::new(db_pool)),
  }
}

/// The main entry point for running the application server.
///
/// This function performs the following steps:
/// 1. Initializes the `tracing` subscriber for structured logging.
/// 2. Reads the `HOST` and `PORT` from environment variables, with default fallbacks.
/// 3. Calls `setup_state()` to create the application state.
/// 4. Binds a TCP listener to the specified address.
/// 5. Starts the Axum server and serves the application.
///
/// # Panics
///
/// This function will panic if it fails to bind the TCP listener or start the server.
pub async fn run() {
  dotenvy::dotenv().ok();
  tracing_subscriber::fmt().with_max_level(Level::INFO).init();

  let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
  let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
  let addr = format!("{}:{}", host, port);

  let app_state = setup_state().await;
  let app = app(app_state);

  let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind to address");

  info!("🚀 Server running on http://{}", &addr);
  axum::serve(listener, app).await.expect("Failed to start server");
}
