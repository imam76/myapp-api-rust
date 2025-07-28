use axum::{
  Router,
  routing::{delete, get, post, put},
};

use crate::{AppState, modules::datastores::contacts::contact_handlers};

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", get(contact_handlers::get_list))
    .route("/", post(|| async { "POST Contact route" }))
    .route("/{id}", get(|| async { "GET Contact by ID route" }))
    .route("/{id}", put(|| async { "PUT Contact by ID route" }))
    .route("/{id}", delete(|| async { "DELETE Contact by ID route" }))
}
