use axum::{
  Router,
  routing::{delete, get, post, put},
};

use crate::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/contact", get(|| async { "GET Contact route" }))
    .route("/contact", post(|| async { "POST Contact route" }))
    .route("/contact/{id}", get(|| async { "GET Contact by ID route" }))
    .route("/contact/{id}", put(|| async { "PUT Contact by ID route" }))
    .route("/contact/{id}", delete(|| async { "DELETE Contact by ID route" }))
}
