use std::sync::Arc;

use axum::{
  Router,
  routing::{delete, get, post, put},
};

use crate::{AppState, modules::datastores::contacts::contact_handlers};

pub fn router() -> Router<Arc<AppState>> {
  Router::new()
    .route("/", get(contact_handlers::get_list))
    .route("/", post(contact_handlers::create))
    .route("/next-code", get(contact_handlers::get_next_code))
    .route("/:id", get(contact_handlers::get_by_id))
    .route("/:id", put(contact_handlers::update))
    .route("/:id", delete(contact_handlers::delete))
}
