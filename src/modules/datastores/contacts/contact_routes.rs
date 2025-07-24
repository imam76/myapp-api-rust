use axum::{
  Router,
  routing::{delete, get, post, put},
};

use crate::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", get(|| async { "GET Contact route" }))
    .route("/", post(|| async { "POST Contact route" }))
    .route("/{id}", get(|| async { "GET Contact by ID route" }))
    .route("/{id}", put(|| async { "PUT Contact by ID route" }))
    .route("/{id}", delete(|| async { "DELETE Contact by ID route" }))
}
