use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::{
    errors::AppError,
    modules::auth::{
        auth_service::{login_user, register_user},
        user_dto::{LoginUserDto, RegisterUserDto},
    },
    state::AppState,
};

pub async fn register_user_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterUserDto>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let user = register_user(state, body).await?;
    let user_response = json!({"status": "success", "data": serde_json::to_value(user).unwrap()});
    Ok((StatusCode::CREATED, Json(user_response)))
}

pub async fn login_user_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginUserDto>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let token = login_user(state, body).await?;
    let token_response = json!({"status": "success", "token": token});
    Ok((StatusCode::OK, Json(token_response)))
}
