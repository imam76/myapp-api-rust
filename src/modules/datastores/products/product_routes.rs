use std::sync::Arc;

use axum::{
  Router,
  routing::{delete, get, post, put},
};

use crate::{AppState, modules::datastores::products::product_handlers};

pub fn router() -> Router<Arc<AppState>> {
  Router::new()
    .route("/", get(product_handlers::get_list))
    .route("/", post(product_handlers::create))
    .route("/:id", get(product_handlers::get_by_id))
    .route("/:id", put(product_handlers::update))
    .route("/:id", delete(product_handlers::delete))
}
