use std::sync::Arc;

use axum::{
  Router,
  routing::{get, post},
};

use crate::{
  modules::auth::auth_handler::{get_current_user_handler, login_user_handler, register_user_handler},
  state::AppState,
};

/// Returns public authentication routes (register and login)
pub fn public_auth_routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/register", post(register_user_handler))
    .route("/login", post(login_user_handler))
}

/// Returns protected authentication routes (me endpoint)
pub fn protected_auth_routes() -> Router<Arc<AppState>> {
  Router::new().route("/me", get(get_current_user_handler))
}
