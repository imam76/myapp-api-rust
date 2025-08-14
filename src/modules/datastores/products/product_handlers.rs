use std::sync::Arc;

use crate::{
  AppResult, AppState,
  errors::{AppError, NotFoundError},
  helper::{WorkspaceContext, workspace::check_workspace_permission},
  impl_next_code_handler,
  modules::{
    auth::current_user::CurrentUser,
    datastores::{
      products::product_models::{CreateProductRequest, GetProductsQuery, ProductFilters, ProductResponse, UpdateProductRequest},
      workspaces::workspace_models::WorkspaceRole,
    },
  },
  responses::{ApiResponse, PaginatedResponse, PaginationMeta},
  utils::{code_generator::CodeGeneratorConfig, next_code_macro::NextCodeQuery},
};
use axum::{
  Json,
  extract::{
    Path, Query, State,
    rejection::{JsonRejection, QueryRejection},
  },
  http::StatusCode,
};
use uuid::Uuid;
use validator::Validate;

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 100;

// Generate next_code handler using macro
impl_next_code_handler!(
  get_next_code,
  "product",
  CodeGeneratorConfig {
    table_name: "products".to_string(),
    code_column: "code".to_string(),
    workspace_column: Some("workspace_id".to_string()),
    prefix_length: 2,
    number_length: 5,
    separator: "-".to_string(),
  }
);

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
  query_params: Result<Query<GetProductsQuery>, QueryRejection>,
  current_user: CurrentUser,
  WorkspaceContext(workspace_id): WorkspaceContext, // Extracted from request headers
) -> AppResult<Json<ApiResponse<PaginatedResponse<ProductResponse>>>> {
  let repository = &state.product_repository;

  // Handle query parameter parsing errors
  let params = match query_params {
    Ok(Query(params)) => params,
    Err(rejection) => return Err(crate::errors::AppError::from(rejection)),
  };

  let page = params.page.unwrap_or(DEFAULT_PAGE);
  let mut limit = params.limit.unwrap_or(DEFAULT_LIMIT);

  if limit > MAX_LIMIT {
    limit = MAX_LIMIT;
  }

  tracing::debug!(
    "Fetching products for workspace_id {}: page={}, limit={}, has_filters={}",
    workspace_id,
    page,
    limit,
    super::product_query_builder::has_filters(&params)
  );

  // Check workspace permissions
  let workspace_repository = &state.workspace_repository;
  if !check_workspace_permission(workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
    return Err(AppError::Authorization("You don't have permission to access this workspace".to_string()));
  }

  let (products, total) = if super::product_query_builder::has_filters(&params) {
    let filters = ProductFilters::from(params);
    repository
      .find_by_filters_paginated(workspace_id, current_user.user_id, page, limit, filters)
      .await?
  } else {
    repository
      .find_all_by_workspace_paginated(workspace_id, current_user.user_id, page, limit)
      .await?
  };
  let pagination = PaginationMeta::new(page, limit, total);

  tracing::debug!("Retrieved {} products for workspace {}", products.len(), workspace_id);

  let response = ApiResponse::success(
    PaginatedResponse {
      list: products.into_iter().map(ProductResponse::from).collect(),
      pagination,
    },
    "Products retrieved successfully",
  );
  Ok(Json(response))
}

/// Handles the request to create a new product for the authenticated user.
/// The product will be created in the specified workspace or user's default workspace.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `current_user`: The authenticated user extracted from the JWT token.
/// * `payload`: The JSON payload containing the new product's data.
///
/// # Returns
///
/// A `Json` response containing the newly created `ProductResponse`.
#[axum::debug_handler]
pub async fn create(
  State(state): State<Arc<AppState>>,
  current_user: CurrentUser,
  WorkspaceContext(workspace_id): WorkspaceContext, // Extracted from request headers
  payload: Result<Json<CreateProductRequest>, JsonRejection>,
) -> AppResult<(StatusCode, Json<ApiResponse<ProductResponse>>)> {
  let repository = &state.product_repository;

  // Extract payload first
  let Json(mut payload) = payload?;

  // Auto-generate code if field is empty
  if payload.code.trim().is_empty() {
    let generated_code = repository.get_next_available_code(workspace_id, &payload.name).await?;
    tracing::debug!("Auto-generated code: {} for name: '{}'", generated_code, payload.name);
    payload.code = generated_code;
  }

  // Now validate with the final code
  payload.validate()?;

  tracing::debug!(
    "Creating product with code: {} for user: {} in workspace: {}",
    payload.code,
    current_user.user_id,
    workspace_id
  );

  // Check workspace permissions
  let workspace_repository = &state.workspace_repository;
  if !check_workspace_permission(workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
    return Err(AppError::Authorization(
      "You don't have permission to create products in this workspace".to_string(),
    ));
  }

  // Check if code already exists in this workspace
  if repository.code_exists(&payload.code, workspace_id).await? {
    return Err(AppError::Conflict("Product code already exists in this workspace".to_string()));
  }

  let new_product = repository.create_by_workspace(payload, workspace_id, current_user.user_id).await?;

  tracing::info!(
    "Product created successfully: id={}, code={}, name={}",
    new_product.id,
    new_product.code,
    new_product.name
  );

  let response = ApiResponse::success(ProductResponse::from(new_product), "Product created successfully");
  Ok((StatusCode::CREATED, Json(response)))
}

/// Handles the request to retrieve a specific product by its ID.
/// This handler ensures that the product belongs to the user's workspace.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The UUID of the product to retrieve.
/// * `current_user`: The authenticated user extracted from the JWT token.
///
/// # Returns
///
/// A `Json` response containing the requested `ProductResponse`.
#[axum::debug_handler]
pub async fn get_by_id(
  State(state): State<Arc<AppState>>,
  Path(id): Path<Uuid>,
  current_user: CurrentUser,
  WorkspaceContext(workspace_id): WorkspaceContext, // Extracted from request headers
) -> AppResult<Json<ApiResponse<ProductResponse>>> {
  let repository = &state.product_repository;

  tracing::debug!(
    "Fetching product with id: {} for user: {} in workspace: {}",
    id,
    current_user.user_id,
    workspace_id
  );

  // Check workspace permissions
  let workspace_repository = &state.workspace_repository;
  if !check_workspace_permission(workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
    return Err(AppError::Authorization("You don't have permission to access this workspace".to_string()));
  }

  let product = repository
    .find_by_id_and_workspace(id, workspace_id, current_user.user_id)
    .await?
    .ok_or_else(|| {
      AppError::NotFound(NotFoundError {
        resource: "Product".to_string(),
        id: Some(id),
      })
    })?;

  let response = ApiResponse::success(ProductResponse::from(product), "Product retrieved successfully");
  Ok(Json(response))
}

/// Handles the request to update an existing product.
/// This handler ensures that the product belongs to the user's workspace.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The UUID of the product to update.
/// * `current_user`: The authenticated user extracted from the JWT token.
/// * `payload`: The JSON payload containing the updated product data.
///
/// # Returns
///
/// A `Json` response containing the updated `ProductResponse`.
#[axum::debug_handler]
pub async fn update(
  State(state): State<Arc<AppState>>,
  Path(id): Path<Uuid>,
  current_user: CurrentUser,
  WorkspaceContext(workspace_id): WorkspaceContext, // Extracted from request headers
  payload: Result<Json<UpdateProductRequest>, JsonRejection>,
) -> AppResult<Json<ApiResponse<ProductResponse>>> {
  let repository = &state.product_repository;
  let Json(payload) = payload?;

  tracing::debug!(
    "Updating product with id: {} for user: {} in workspace: {}",
    id,
    current_user.user_id,
    workspace_id
  );

  // Check workspace permissions
  let workspace_repository = &state.workspace_repository;
  if !check_workspace_permission(workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
    return Err(AppError::Authorization(
      "You don't have permission to update products in this workspace".to_string(),
    ));
  }

  // Check if the product exists before updating
  if repository
    .find_by_id_and_workspace(id, workspace_id, current_user.user_id)
    .await?
    .is_none()
  {
    return Err(AppError::NotFound(NotFoundError {
      resource: "Product".to_string(),
      id: Some(id),
    }));
  }

  // If updating code, check if the new code already exists (excluding current product)
  if let Some(ref new_code) = payload.code {
    let existing_product = repository.find_by_code_and_workspace(new_code, workspace_id).await?;
    if let Some(existing) = existing_product {
      if existing.id != id {
        return Err(AppError::Conflict("Product code already exists in this workspace".to_string()));
      }
    }
  }

  let updated_product = repository
    .update_by_workspace(id, workspace_id, payload, current_user.user_id)
    .await?
    .ok_or_else(|| {
      AppError::NotFound(NotFoundError {
        resource: "Product".to_string(),
        id: Some(id),
      })
    })?;

  tracing::info!(
    "Product updated successfully: id={}, code={}, name={}",
    updated_product.id,
    updated_product.code,
    updated_product.name
  );

  let response = ApiResponse::success(ProductResponse::from(updated_product), "Product updated successfully");
  Ok(Json(response))
}

/// Handles the request to delete a product.
/// This handler ensures that the product belongs to the user's workspace.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The UUID of the product to delete.
/// * `current_user`: The authenticated user extracted from the JWT token.
///
/// # Returns
///
/// A `Json` response confirming the deletion.
#[axum::debug_handler]
pub async fn delete(
  State(state): State<Arc<AppState>>,
  Path(id): Path<Uuid>,
  current_user: CurrentUser,
  WorkspaceContext(workspace_id): WorkspaceContext, // Extracted from request headers
) -> AppResult<Json<ApiResponse<()>>> {
  let repository = &state.product_repository;

  tracing::debug!(
    "Deleting product with id: {} for user: {} in workspace: {}",
    id,
    current_user.user_id,
    workspace_id
  );

  // Check workspace permissions
  let workspace_repository = &state.workspace_repository;
  if !check_workspace_permission(workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
    return Err(AppError::Authorization(
      "You don't have permission to delete products in this workspace".to_string(),
    ));
  }

  let deleted = repository.delete_by_workspace_and_user(id, workspace_id, current_user.user_id).await?;

  if !deleted {
    return Err(AppError::NotFound(NotFoundError {
      resource: "Product".to_string(),
      id: Some(id),
    }));
  }

  tracing::info!("Product deleted successfully: id={}", id);

  let response = ApiResponse::success((), "Product deleted successfully");
  Ok(Json(response))
}
