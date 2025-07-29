use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::sync::Arc;

use crate::{
    errors::{AppError, AuthError},
    modules::auth::auth_service::Claims,
    state::AppState,
};

/// JWT middleware that validates the Authorization header and extracts user claims.
///
/// This middleware checks for a valid JWT token in the Authorization header
/// and extracts the user ID for use in protected routes.
pub async fn jwt_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract the Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| AppError::Authentication(AuthError::MissingToken))?;

    // Check if the header starts with "Bearer "
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Authentication(AuthError::InvalidToken));
    }

    // Extract the token part (after "Bearer ")
    let token = &auth_header[7..];

    // Decode and validate the JWT token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Authentication(AuthError::InvalidToken))?;

    // Extract the user ID from the token claims
    let user_id = token_data.claims.sub;

    // Add the user ID to the request extensions for use in handlers
    request.extensions_mut().insert(user_id);

    // Continue to the next middleware/handler
    Ok(next.run(request).await)
}
