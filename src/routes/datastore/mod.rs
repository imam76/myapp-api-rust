use axum::Router;

use crate::AppState;

pub mod contact;

pub fn router() -> Router<AppState> {
    Router::new()
        // Semua data store routes butuh authentication
        .merge(contact::router())
}
