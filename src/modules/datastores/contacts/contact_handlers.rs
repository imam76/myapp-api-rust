use crate::{
  AppResult, AppState,
  modules::datastores::contacts::{contact_models::ContactResponse, contact_repository::create_contact_repository},
  responses::{ApiResponse, GetContactsQuery, PaginatedResponse, PaginationMeta},
};
use axum::{
  Json,
  extract::{Query, State},
};

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 100;

#[axum::debug_handler]
pub async fn get_list(
  State(state): State<AppState>,
  Query(params): Query<GetContactsQuery>,
) -> AppResult<Json<ApiResponse<PaginatedResponse<ContactResponse>>>> {
  let repository = create_contact_repository(state.db);

  let page = params.page.unwrap_or(DEFAULT_PAGE);
  let mut limit = params.limit.unwrap_or(DEFAULT_LIMIT);

  if limit > MAX_LIMIT {
    limit = MAX_LIMIT;
  }

  tracing::debug!("Fetching contacts: page={}, limit={}", page, limit);

  let (contacts, total) = repository.find_all_paginated(page, limit).await?;
  let pagination = PaginationMeta::new(page, limit, total);

  tracing::debug!("Retrieved {} contacts", contacts.len());

  let response = ApiResponse::success(
    PaginatedResponse {
      list: contacts.into_iter().map(ContactResponse::from).collect(),
      pagination,
    },
    "Contacts retrieved successfully",
  );

  Ok(Json(response))
}
