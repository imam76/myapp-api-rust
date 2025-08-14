use sea_query::extension::postgres::PgExpr;
use sea_query::{Expr, Iden, Order, PostgresQueryBuilder, Query, SelectStatement};
use uuid::Uuid;

use super::product_models::{GetProductsQuery, ProductFilters};

// Define table and column enums for type safety
#[derive(Iden)]
enum Products {
  Table,
  Id,
  Code,
  Name,
  CategoryId,
  BaseUnit,
  UnitOnReportPreview,
  SellingPrice,
  UnitCost,
  SupplierId,
  TrackInventory,
  Description,
  Sku,
  Barcode,
  MinimumStock,
  MaximumStock,
  ReorderLevel,
  CurrentStock,
  TaxType,
  TaxRate,
  TaxAmount,
  IsActive,
  WorkspaceId,
  CreatedBy,
  UpdatedBy,
  CreatedAt,
  UpdatedAt,
}

pub struct ProductQueryBuilder;

impl ProductQueryBuilder {
  pub fn build_filtered_query(workspace_id: Uuid, _user_id: Uuid, filters: &ProductFilters) -> (String, String) {
    // Build select query
    let select_sql = Self::build_select_query(workspace_id, _user_id, filters);

    // Build count query
    let count_sql = Self::build_count_query(workspace_id, _user_id, filters);

    (select_sql, count_sql)
  }

  fn build_select_query(workspace_id: Uuid, _user_id: Uuid, filters: &ProductFilters) -> String {
    let mut query = Query::select()
      .columns([
        Products::Id,
        Products::Code,
        Products::Name,
        Products::CategoryId,
        Products::BaseUnit,
        Products::UnitOnReportPreview,
        Products::SellingPrice,
        Products::UnitCost,
        Products::SupplierId,
        Products::TrackInventory,
        Products::Description,
        Products::Sku,
        Products::Barcode,
        Products::MinimumStock,
        Products::MaximumStock,
        Products::ReorderLevel,
        Products::CurrentStock,
        Products::TaxType,
        Products::TaxRate,
        Products::TaxAmount,
        Products::IsActive,
        Products::WorkspaceId,
        Products::CreatedBy,
        Products::UpdatedBy,
        Products::CreatedAt,
        Products::UpdatedAt,
      ])
      .from(Products::Table)
      .and_where(Expr::col(Products::WorkspaceId).eq(workspace_id.to_string()))
      .to_owned();

    // Apply filters
    Self::apply_filters(&mut query, filters);

    // Apply sorting
    Self::apply_sorting(&mut query, &filters.sort_by, &filters.sort_order);

    query.to_string(PostgresQueryBuilder)
  }

  fn build_count_query(workspace_id: Uuid, _user_id: Uuid, filters: &ProductFilters) -> String {
    let mut query = Query::select()
      .expr(Expr::col((Products::Table, Products::Id)).count())
      .from(Products::Table)
      .and_where(Expr::col(Products::WorkspaceId).eq(workspace_id.to_string()))
      .to_owned();

    // Apply the same filters as select query (except sorting)
    Self::apply_filters(&mut query, filters);

    query.to_string(PostgresQueryBuilder)
  }

  fn apply_filters(query: &mut SelectStatement, filters: &ProductFilters) {
    // Search filter (across multiple fields)
    if let Some(search) = &filters.search {
      let search_pattern = format!("%{}%", search.to_lowercase());
      query.and_where(
        Expr::col(Products::Name)
          .ilike(&search_pattern)
          .or(Expr::col(Products::Code).ilike(&search_pattern))
          .or(Expr::col(Products::Sku).ilike(&search_pattern))
          .or(Expr::col(Products::Barcode).ilike(&search_pattern))
          .or(Expr::col(Products::Description).ilike(&search_pattern)),
      );
    }

    // Category filter
    if let Some(category_id) = filters.category_id {
      query.and_where(Expr::col(Products::CategoryId).eq(category_id.to_string()));
    }

    // Supplier filter
    if let Some(supplier_id) = filters.supplier_id {
      query.and_where(Expr::col(Products::SupplierId).eq(supplier_id.to_string()));
    }

    // Active filter
    if let Some(is_active) = filters.is_active {
      query.and_where(Expr::col(Products::IsActive).eq(is_active));
    }

    // Track inventory filter
    if let Some(track_inventory) = filters.track_inventory {
      query.and_where(Expr::col(Products::TrackInventory).eq(track_inventory));
    }

    // Code filter
    if let Some(code) = &filters.code {
      query.and_where(Expr::col(Products::Code).eq(code));
    }

    // SKU filter
    if let Some(sku) = &filters.sku {
      query.and_where(Expr::col(Products::Sku).eq(sku));
    }

    // Barcode filter
    if let Some(barcode) = &filters.barcode {
      query.and_where(Expr::col(Products::Barcode).eq(barcode));
    }

    // Base unit filter
    if let Some(base_unit) = &filters.base_unit {
      query.and_where(Expr::col(Products::BaseUnit).eq(base_unit));
    }

    // Tax type filter
    if let Some(tax_type) = &filters.tax_type {
      query.and_where(Expr::col(Products::TaxType).eq(tax_type));
    }

    // Include categories
    if !filters.include_categories.is_empty() {
      let category_strings: Vec<String> = filters.include_categories.iter().map(|uuid| uuid.to_string()).collect();
      query.and_where(Expr::col(Products::CategoryId).is_in(category_strings));
    }

    // Exclude categories
    if !filters.exclude_categories.is_empty() {
      let category_strings: Vec<String> = filters.exclude_categories.iter().map(|uuid| uuid.to_string()).collect();
      query.and_where(Expr::col(Products::CategoryId).is_not_in(category_strings));
    }

    // Include suppliers
    if !filters.include_suppliers.is_empty() {
      let supplier_strings: Vec<String> = filters.include_suppliers.iter().map(|uuid| uuid.to_string()).collect();
      query.and_where(Expr::col(Products::SupplierId).is_in(supplier_strings));
    }

    // Exclude suppliers
    if !filters.exclude_suppliers.is_empty() {
      let supplier_strings: Vec<String> = filters.exclude_suppliers.iter().map(|uuid| uuid.to_string()).collect();
      query.and_where(Expr::col(Products::SupplierId).is_not_in(supplier_strings));
    }

    // Include IDs
    if !filters.include_ids.is_empty() {
      let id_strings: Vec<String> = filters.include_ids.iter().map(|uuid| uuid.to_string()).collect();
      query.and_where(Expr::col(Products::Id).is_in(id_strings));
    }

    // Exclude IDs
    if !filters.exclude_ids.is_empty() {
      let id_strings: Vec<String> = filters.exclude_ids.iter().map(|uuid| uuid.to_string()).collect();
      query.and_where(Expr::col(Products::Id).is_not_in(id_strings));
    }

    // Price filters
    if let Some(min_selling_price) = filters.min_selling_price {
      query.and_where(Expr::col(Products::SellingPrice).gte(min_selling_price.to_string()));
    }

    if let Some(max_selling_price) = filters.max_selling_price {
      query.and_where(Expr::col(Products::SellingPrice).lte(max_selling_price.to_string()));
    }

    if let Some(min_unit_cost) = filters.min_unit_cost {
      query.and_where(Expr::col(Products::UnitCost).gte(min_unit_cost.to_string()));
    }

    if let Some(max_unit_cost) = filters.max_unit_cost {
      query.and_where(Expr::col(Products::UnitCost).lte(max_unit_cost.to_string()));
    }

    // Stock filters
    if let Some(min_current_stock) = filters.min_current_stock {
      query.and_where(Expr::col(Products::CurrentStock).gte(min_current_stock));
    }

    if let Some(max_current_stock) = filters.max_current_stock {
      query.and_where(Expr::col(Products::CurrentStock).lte(max_current_stock));
    }

    // Low stock filter
    if let Some(true) = filters.low_stock {
      query.and_where(
        Expr::col(Products::TrackInventory)
          .eq(true)
          .and(Expr::col(Products::CurrentStock).is_not_null())
          .and(Expr::col(Products::ReorderLevel).is_not_null())
          .and(Expr::col(Products::CurrentStock).lte(Expr::col(Products::ReorderLevel))),
      );
    }
  }

  fn apply_sorting(query: &mut SelectStatement, sort_by: &str, sort_order: &str) {
    let order = if sort_order.to_uppercase() == "ASC" { Order::Asc } else { Order::Desc };

    let column = match sort_by {
      "name" => Products::Name,
      "code" => Products::Code,
      "selling_price" => Products::SellingPrice,
      "unit_cost" => Products::UnitCost,
      "created_at" => Products::CreatedAt,
      "updated_at" => Products::UpdatedAt,
      _ => Products::CreatedAt, // default
    };

    query.order_by(column, order);
  }
}

/// Utility function to check if any filters are applied
pub fn has_filters(query: &GetProductsQuery) -> bool {
  query.search.is_some()
    || query.category_id.is_some()
    || query.supplier_id.is_some()
    || query.is_active.is_some()
    || query.track_inventory.is_some()
    || query.code.is_some()
    || query.sku.is_some()
    || query.barcode.is_some()
    || query.base_unit.is_some()
    || query.tax_type.is_some()
    || query.include_categories.is_some()
    || query.exclude_categories.is_some()
    || query.include_suppliers.is_some()
    || query.exclude_suppliers.is_some()
    || query.include_ids.is_some()
    || query.exclude_ids.is_some()
    || query.min_selling_price.is_some()
    || query.max_selling_price.is_some()
    || query.min_unit_cost.is_some()
    || query.max_unit_cost.is_some()
    || query.min_current_stock.is_some()
    || query.max_current_stock.is_some()
    || query.low_stock.is_some()
}
