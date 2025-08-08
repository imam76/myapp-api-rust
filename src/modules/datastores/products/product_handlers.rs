use std::sync::Arc;

use crate::{
  AppResult, AppState,
  errors::{AppError, NotFoundError},
  helper::workspace::check_workspace_permission,
  modules::{
    auth::current_user::CurrentUser,
    datastores::{
      products::product_models::{CreateProductRequest, GetProductsQuery, ProductResponse, UpdateProductRequest},
      workspaces::workspace_models::WorkspaceRole,
    },
  },
  responses::{ApiResponse, PaginatedResponse, PaginationMeta},
};
use axum::{
  Json,
  extract::{Path, Query, State, rejection::JsonRejection},
  http::StatusCode,
};
use uuid::Uuid;
use validator::Validate;

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 100;

/// Handles the request to retrieve a paginated list of products for the authenticated user.
/// This handler will get products from the user's default workspace or all accessible workspaces.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Query(params)`: The query parameters for pagination (`page`, `limit`).
/// * `current_user`: The authenticated user extracted from the JWT token.
///
/// # Returns
///
/// A `Json` response containing a paginated list of `ProductResponse` objects that belong to the user.
#[axum::debug_handler]
pub async fn get_list(
  State(state): State<Arc<AppState>>,
  Query(params): Query<GetProductsQuery>,
  current_user: CurrentUser,
) -> AppResult<Json<ApiResponse<PaginatedResponse<ProductResponse>>>> {
  let repository = &state.product_repository;

  let page = params.page.unwrap_or(DEFAULT_PAGE);
  let mut limit = params.limit.unwrap_or(DEFAULT_LIMIT);

  if limit > MAX_LIMIT {
    limit = MAX_LIMIT;
  }

  tracing::debug!("Fetching products for user {}: page={}, limit={}", current_user.user_id, page, limit);

  let (products, total) = repository.find_all_by_user_paginated(current_user.user_id, page, limit).await?;
  let pagination = PaginationMeta::new(page, limit, total);

  tracing::debug!("Retrieved {} products for user {}", products.len(), current_user.user_id);

  let response = ApiResponse::success(
    PaginatedResponse {
      list: products.into_iter().map(ProductResponse::from).collect(),
      pagination,
    },
    "Products retrieved successfully",
  );

  Ok(Json(response))
}

/// Handles the request to create a new products for the authenticated user.
/// The products will be created in the specified workspace or user's default workspace.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `current_user`: The authenticated user extracted from the JWT token.
/// * `payload`: The JSON payload containing the new products's data.
///
/// # Returns
///
/// A `Json` response containing the newly created `ProductResponse`.
#[axum::debug_handler]
pub async fn create(
  State(state): State<Arc<AppState>>,
  current_user: CurrentUser,
  payload: Result<Json<CreateProductRequest>, JsonRejection>,
) -> AppResult<(StatusCode, Json<ApiResponse<ProductResponse>>)> {
  let repository = &state.product_repository;

  // Extract and validate the payload
  let Json(payload) = payload?;
  payload.validate()?;

  tracing::debug!("Creating products with code: {} for user: {}", payload.code, current_user.user_id);

  // Check if workspace_id is provided and validate access
  if let Some(workspace_id) = payload.workspace_id {
    let workspace_repository = &state.workspace_repository;
    if !check_workspace_permission(&workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
      return Err(AppError::Authorization(
        "You don't have permission to create products in this workspace".to_string(),
      ));
    }

    // Check if code already exists in this workspace
    if let Some(_) = repository.find_by_code_and_workspace(&payload.code, workspace_id).await? {
      return Err(AppError::validation_with_code(
        "code",
        "Product code already exists in this workspace",
        "DUPLICATE_CODE",
      ));
    }
  } else {
    // Check if code already exists for this user (global check for user-level products)
    if let Some(_) = repository.find_by_code(&payload.code).await? {
      return Err(AppError::validation_with_code("code", "Product code already exists", "DUPLICATE_CODE"));
    }
  }

  let products = repository.create(payload, current_user.user_id).await?;

  tracing::info!("Product created successfully with ID: {} for user: {}", products.id, current_user.user_id);

  let response = ApiResponse::success(ProductResponse::from(products), "Product created successfully");

  Ok((StatusCode::CREATED, Json(response)))
}

/// Handles the request to retrieve a single products by its ID for the authenticated user.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The ID of the products to retrieve, extracted from the URL path.
/// * `current_user`: The authenticated user extracted from the JWT token.
///
/// # Returns
///
/// A `Json` response containing the `ProductResponse` if found and accessible by the user, otherwise a 404 Not Found error.
#[axum::debug_handler]
pub async fn get_by_id(
  State(state): State<Arc<AppState>>,
  Path(id): Path<String>,
  current_user: CurrentUser,
) -> AppResult<Json<ApiResponse<ProductResponse>>> {
  let repository = &state.product_repository;

  // Parse UUID with global error handling
  let id = id.parse::<Uuid>()?;

  tracing::debug!("Fetching products with ID: {} for user: {}", id, current_user.user_id);

  let products = repository.find_by_id_and_user(id, current_user.user_id).await?.ok_or_else(|| {
    AppError::NotFound(NotFoundError {
      resource: "Product".to_string(),
      id: Some(id),
    })
  })?;

  tracing::debug!("Product with ID {} found for user {}", id, current_user.user_id);

  let response = ApiResponse::success(ProductResponse::from(products), "Product retrieved successfully");
  Ok(Json(response))
}

/// Handles the request to update an existing products for the authenticated user.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The ID of the products to update.
/// * `current_user`: The authenticated user extracted from the JWT token.
/// * `payload`: The JSON payload with the fields to update.
///
/// # Returns
///
/// A `Json` response containing the updated `ProductResponse` if successful, otherwise a 404 error.
#[axum::debug_handler]
pub async fn update(
  State(state): State<Arc<AppState>>,
  Path(id): Path<String>,
  current_user: CurrentUser,
  payload: Result<Json<UpdateProductRequest>, JsonRejection>,
) -> AppResult<Json<ApiResponse<ProductResponse>>> {
  let repository = &state.product_repository;
  let Json(payload) = payload?;

  // Parse UUID with global error handling
  let id = id.parse::<Uuid>()?;

  tracing::debug!("Updating products with ID: {} for user: {}", id, current_user.user_id);

  // Check if workspace_id is provided and validate access
  if let Some(workspace_id) = payload.workspace_id {
    let workspace_repository = &state.workspace_repository;
    if !check_workspace_permission(&workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
      return Err(AppError::Authorization(
        "You don't have permission to update products in this workspace".to_string(),
      ));
    }

    let updated_products = repository
      .update_by_workspace(id, workspace_id, payload, current_user.user_id)
      .await?
      .ok_or_else(|| {
        AppError::NotFound(NotFoundError {
          resource: "Product".to_string(),
          id: Some(id),
        })
      })?;

    tracing::info!("Product with ID {} updated successfully for workspace {}", id, workspace_id);
    let response = ApiResponse::success(ProductResponse::from(updated_products), "Product updated successfully");
    Ok(Json(response))
  } else {
    let updated_products = repository.update(id, payload, current_user.user_id).await?.ok_or_else(|| {
      AppError::NotFound(NotFoundError {
        resource: "Product".to_string(),
        id: Some(id),
      })
    })?;

    tracing::info!("Product with ID {} updated successfully for user {}", id, current_user.user_id);
    let response = ApiResponse::success(ProductResponse::from(updated_products), "Product updated successfully");
    Ok(Json(response))
  }
}

/// Handles the request to delete a products by its ID for the authenticated user.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The ID of the products to delete.
/// * `current_user`: The authenticated user extracted from the JWT token.
///
/// # Returns
///
/// A `Json` response with a success message if the deletion was successful, otherwise a 404 error.
#[axum::debug_handler]
pub async fn delete(State(state): State<Arc<AppState>>, Path(id): Path<String>, current_user: CurrentUser) -> AppResult<Json<ApiResponse<()>>> {
  let repository = &state.product_repository;

  // Parse UUID with global error handling
  let id = id.parse::<Uuid>()?;

  tracing::debug!("Deleting products with ID: {} for user: {}", id, current_user.user_id);

  let deleted = repository.delete(id, current_user.user_id).await?;

  if !deleted {
    return Err(AppError::NotFound(NotFoundError {
      resource: "Product".to_string(),
      id: Some(id),
    }));
  }

  tracing::info!("Product with ID {} deleted successfully for user {}", id, current_user.user_id);

  let response = ApiResponse::success((), "Product deleted successfully");
  Ok(Json(response))
}
