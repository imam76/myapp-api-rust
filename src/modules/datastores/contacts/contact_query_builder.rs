use sea_query::{Expr, Iden, Order, PostgresQueryBuilder, Query, SelectStatement};
use uuid::Uuid;

use super::contact_models::{ContactFilters, GetContactsQuery};

// Define table and column enums for type safety
#[derive(Iden)]
enum Contacts {
  Table,
  Id,
  Code,
  Name,
  Email,
  Position,
  Type,
  Address,
  IsActive,
  WorkspaceId,
  CreatedBy,
  UpdatedBy,
  CreatedAt,
  UpdatedAt,
}

#[derive(Iden)]
enum Workspaces {
  Table,
  Id,
}

#[derive(Iden)]
enum WorkspaceUsers {
  Table,
  WorkspaceId,
  UserId,
}

pub struct ContactQueryBuilder;

impl ContactQueryBuilder {
  pub fn build_filtered_query(workspace_id: Uuid, user_id: Uuid, filters: &ContactFilters) -> (String, String) {
    // Build select query
    let select_sql = Self::build_select_query(workspace_id, user_id, filters);

    // Build count query
    let count_sql = Self::build_count_query(workspace_id, user_id, filters);

    (select_sql, count_sql)
  }

  fn build_select_query(workspace_id: Uuid, user_id: Uuid, filters: &ContactFilters) -> String {
    let mut query = Query::select();

    // Select columns with alias
    query
      .columns([
        (Contacts::Table, Contacts::Id),
        (Contacts::Table, Contacts::Code),
        (Contacts::Table, Contacts::Name),
        (Contacts::Table, Contacts::Email),
        (Contacts::Table, Contacts::Position),
        (Contacts::Table, Contacts::Type),
        (Contacts::Table, Contacts::Address),
        (Contacts::Table, Contacts::IsActive),
        (Contacts::Table, Contacts::WorkspaceId),
        (Contacts::Table, Contacts::CreatedBy),
        (Contacts::Table, Contacts::UpdatedBy),
        (Contacts::Table, Contacts::CreatedAt),
        (Contacts::Table, Contacts::UpdatedAt),
      ])
      .from(Contacts::Table)
      .inner_join(
        Workspaces::Table,
        Expr::col((Contacts::Table, Contacts::WorkspaceId)).equals((Workspaces::Table, Workspaces::Id)),
      )
      .inner_join(
        WorkspaceUsers::Table,
        Expr::col((Workspaces::Table, Workspaces::Id)).equals((WorkspaceUsers::Table, WorkspaceUsers::WorkspaceId)),
      );

    // Base conditions - use string values to avoid UUID conversion issues
    query
      .and_where(Expr::col((Contacts::Table, Contacts::WorkspaceId)).eq(workspace_id.to_string()))
      .and_where(Expr::col((WorkspaceUsers::Table, WorkspaceUsers::UserId)).eq(user_id.to_string()));

    // Apply filters
    Self::apply_filters(&mut query, filters);

    // Apply sorting
    let sort_column = match filters.sort_by.as_str() {
      "name" => Contacts::Name,
      "email" => Contacts::Email,
      "code" => Contacts::Code,
      "contact_type" | "type" => Contacts::Type,
      "updated_at" => Contacts::UpdatedAt,
      _ => Contacts::CreatedAt,
    };

    let sort_order = if filters.sort_order == "ASC" { Order::Asc } else { Order::Desc };

    query.order_by((Contacts::Table, sort_column), sort_order);

    // Build SQL
    query.to_string(PostgresQueryBuilder)
  }

  fn build_count_query(workspace_id: Uuid, user_id: Uuid, filters: &ContactFilters) -> String {
    let mut query = Query::select();

    query
      .expr(Expr::col((Contacts::Table, Contacts::Id)).count())
      .from(Contacts::Table)
      .inner_join(
        Workspaces::Table,
        Expr::col((Contacts::Table, Contacts::WorkspaceId)).equals((Workspaces::Table, Workspaces::Id)),
      )
      .inner_join(
        WorkspaceUsers::Table,
        Expr::col((Workspaces::Table, Workspaces::Id)).equals((WorkspaceUsers::Table, WorkspaceUsers::WorkspaceId)),
      );

    // Base conditions - use string values to avoid UUID conversion issues
    query
      .and_where(Expr::col((Contacts::Table, Contacts::WorkspaceId)).eq(workspace_id.to_string()))
      .and_where(Expr::col((WorkspaceUsers::Table, WorkspaceUsers::UserId)).eq(user_id.to_string()));

    // Apply same filters
    Self::apply_filters(&mut query, filters);

    // Build SQL
    query.to_string(PostgresQueryBuilder)
  }

  fn apply_filters(query: &mut SelectStatement, filters: &ContactFilters) {
    // Search filter (across multiple fields)
    if let Some(search) = &filters.search {
      let search_pattern = format!("%{}%", search);
      let search_condition = Expr::col((Contacts::Table, Contacts::Name))
        .like(&search_pattern)
        .or(Expr::col((Contacts::Table, Contacts::Email)).like(&search_pattern))
        .or(Expr::col((Contacts::Table, Contacts::Code)).like(&search_pattern))
        .or(Expr::col((Contacts::Table, Contacts::Position)).like(&search_pattern));

      query.and_where(search_condition);
    }

    // Contact type filter
    if let Some(contact_type) = &filters.contact_type {
      query.and_where(Expr::col((Contacts::Table, Contacts::Type)).eq(contact_type.as_str()));
    }

    // Active status filter
    if let Some(is_active) = filters.is_active {
      query.and_where(Expr::col((Contacts::Table, Contacts::IsActive)).eq(is_active));
    }

    // Code filter
    if let Some(code) = &filters.code {
      query.and_where(Expr::col((Contacts::Table, Contacts::Code)).like(format!("%{}%", code)));
    }

    // Email filter
    if let Some(email) = &filters.email {
      query.and_where(Expr::col((Contacts::Table, Contacts::Email)).like(format!("%{}%", email)));
    }

    // Include types filter
    if !filters.include_types.is_empty() {
      let types: Vec<&str> = filters.include_types.iter().map(|s| s.as_str()).collect();
      query.and_where(Expr::col((Contacts::Table, Contacts::Type)).is_in(types));
    }

    // Exclude types filter
    if !filters.exclude_types.is_empty() {
      let types: Vec<&str> = filters.exclude_types.iter().map(|s| s.as_str()).collect();
      query.and_where(Expr::col((Contacts::Table, Contacts::Type)).is_not_in(types));
    }

    // Include IDs filter - convert UUIDs to strings
    if !filters.include_ids.is_empty() {
      let id_strings: Vec<String> = filters.include_ids.iter().map(|id| id.to_string()).collect();
      query.and_where(Expr::col((Contacts::Table, Contacts::Id)).is_in(id_strings));
    }

    // Exclude IDs filter - convert UUIDs to strings
    if !filters.exclude_ids.is_empty() {
      let id_strings: Vec<String> = filters.exclude_ids.iter().map(|id| id.to_string()).collect();
      query.and_where(Expr::col((Contacts::Table, Contacts::Id)).is_not_in(id_strings));
    }
  }
}

/// Helper function to check if query has any filters applied
pub fn has_filters(query: &GetContactsQuery) -> bool {
  query.search.is_some()
    || query.contact_type.is_some()
    || query.is_active.is_some()
    || query.code.is_some()
    || query.email.is_some()
    || query.include_types.is_some()
    || query.exclude_types.is_some()
    || query.include_ids.is_some()
    || query.exclude_ids.is_some()
    || query.sort_by.is_some()
    || query.sort_order.is_some()
}
