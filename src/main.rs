use axum::{Router, routing::get};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing::{Level, info};

pub mod errors;
pub mod modules;
pub mod state;

pub use errors::AppError;
pub use state::AppState;

#[tokio::main]
async fn main() {
  // Initialize logging
  tracing_subscriber::fmt().with_max_level(Level::INFO).init();

  dotenv().ok();
  let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
  let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
  let addr = format!("{}:{}", host, port);

  // Database connection
  let db_pool = PgPoolOptions::new()
    .max_connections(10)
    .min_connections(1)
    .connect(&db_url)
    .await
    .expect("Failed to connect to the database");
  info!("✅ Connected to database{}", db_url);

  // Create app state
  let app_state = AppState { db: db_pool };

  let public_routes = Router::new().route("/", get(|| async { "🚀 Welcome to the My Rust Base API!" }));

  let private_routes = Router::new()
    // Add your private routes here, e.g.:
    .nest("/api/v1/contacts", modules::datastores::contacts::contact_routes::router());

  let app = Router::new()
    .merge(public_routes) // Public routes without auth
    .merge(private_routes) // Private routes with auth
    .with_state(app_state)
    .method_not_allowed_fallback(modules::fallback_handler::method_not_allowed)
    .fallback(modules::fallback_handler::method_not_found);

  let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind to address");

  info!("🚀 Server running on http://{}", &addr);
  axum::serve(listener, app).await.expect("Failed to start server");
}
