use std::sync::Arc;

use axum::{routing::post, Router};

use crate::{modules::auth::auth_handler::{login_user_handler, register_user_handler}, state::AppState};

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register_user_handler))
        .route("/login", post(login_user_handler))
}
