use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "tax_type", rename_all = "snake_case")]
pub enum TaxType {
  Percentage,
  FixedAmount,
}

/// Represents a product record in the database.
/// This struct is derived from `sqlx::FromRow` to allow direct mapping from database query results.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
  pub id: Uuid,
  pub code: String,
  pub name: String,
  pub category_id: Option<Uuid>,
  pub base_unit: String,
  pub unit_on_report_preview: Option<String>,
  pub selling_price: rust_decimal::Decimal,
  pub unit_cost: rust_decimal::Decimal,
  pub supplier_id: Option<Uuid>,
  pub track_inventory: bool,

  // Additional fields
  pub description: Option<String>,
  pub sku: Option<String>,
  pub barcode: Option<String>,
  pub minimum_stock: Option<i32>,
  pub maximum_stock: Option<i32>,
  pub reorder_level: Option<i32>,
  pub stock: Option<i32>,
  pub tax_type: Option<TaxType>,
  pub tax_rate: Option<rust_decimal::Decimal>,
  pub tax_amount: Option<rust_decimal::Decimal>,
  pub is_active: bool,

  // Metadata
  pub workspace_id: Option<Uuid>,
  pub created_by: Option<Uuid>,
  pub updated_by: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// Represents the payload for creating a new product.
/// This struct uses `validator` to enforce declarative validation rules on the incoming data.
/// The `created_by` field is automatically set from the authenticated user.
/// The `workspace_id` is now extracted from request headers via WorkspaceContext, not from the body.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
  #[validate(length(min = 1, message = "Code is required"))]
  pub code: String,
  #[validate(length(min = 1, message = "Name is required"))]
  pub name: String,
  pub category_id: Option<Uuid>,
  #[validate(length(min = 1, message = "Base unit is required"))]
  pub base_unit: String,
  pub unit_on_report_preview: Option<String>,
  pub selling_price: rust_decimal::Decimal,
  pub unit_cost: rust_decimal::Decimal,
  pub supplier_id: Option<Uuid>,
  pub track_inventory: Option<bool>,

  // Additional fields
  pub description: Option<String>,
  pub sku: Option<String>,
  pub barcode: Option<String>,
  pub minimum_stock: Option<i32>,
  pub maximum_stock: Option<i32>,
  pub reorder_level: Option<i32>,
  pub stock: Option<i32>,
  pub tax_type: Option<TaxType>,
  pub tax_rate: Option<rust_decimal::Decimal>,
  pub tax_amount: Option<rust_decimal::Decimal>,
}

/// Represents the payload for updating an existing product.
/// All fields are optional, allowing for partial updates.
/// The `updated_by` field is automatically set from the authenticated user.
/// The `workspace_id` cannot be changed via update - it's workspace-scoped.
#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
  pub code: Option<String>,
  pub name: Option<String>,
  pub category_id: Option<Uuid>,
  pub base_unit: Option<String>,
  pub unit_on_report_preview: Option<String>,
  pub selling_price: Option<rust_decimal::Decimal>,
  pub unit_cost: Option<rust_decimal::Decimal>,
  pub supplier_id: Option<Uuid>,
  pub track_inventory: Option<bool>,

  // Additional fields
  pub description: Option<String>,
  pub sku: Option<String>,
  pub barcode: Option<String>,
  pub minimum_stock: Option<i32>,
  pub maximum_stock: Option<i32>,
  pub reorder_level: Option<i32>,
  pub stock: Option<i32>,
  pub tax_type: Option<TaxType>,
  pub tax_rate: Option<rust_decimal::Decimal>,
  pub tax_amount: Option<rust_decimal::Decimal>,
  pub is_active: Option<bool>,
}

/// Represents the data structure for a product response.
/// This struct defines the public-facing representation of a product,
/// including ownership and audit information.
#[derive(Debug, Serialize)]
pub struct ProductResponse {
  pub id: Uuid,
  pub code: String,
  pub name: String,
  pub category_id: Option<Uuid>,
  pub base_unit: String,
  pub unit_on_report_preview: Option<String>,
  pub selling_price: rust_decimal::Decimal,
  pub unit_cost: rust_decimal::Decimal,
  pub supplier_id: Option<Uuid>,
  pub track_inventory: bool,

  // Additional fields
  pub description: Option<String>,
  pub sku: Option<String>,
  pub barcode: Option<String>,
  pub minimum_stock: Option<i32>,
  pub maximum_stock: Option<i32>,
  pub reorder_level: Option<i32>,
  pub stock: Option<i32>,
  pub tax_type: Option<TaxType>,
  pub tax_rate: Option<rust_decimal::Decimal>,
  pub tax_amount: Option<rust_decimal::Decimal>,
  pub is_active: bool,

  // Metadata
  pub workspace_id: Option<Uuid>,
  pub created_by: Option<Uuid>,
  pub updated_by: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// Converts a `Product` model into a `ProductResponse`.
/// This `From` implementation facilitates the transformation of the internal
/// database model into the public API response structure.
impl From<Product> for ProductResponse {
  fn from(product: Product) -> Self {
    Self {
      id: product.id,
      code: product.code,
      name: product.name,
      category_id: product.category_id,
      base_unit: product.base_unit,
      unit_on_report_preview: product.unit_on_report_preview,
      selling_price: product.selling_price,
      unit_cost: product.unit_cost,
      supplier_id: product.supplier_id,
      track_inventory: product.track_inventory,

      // Additional fields
      description: product.description,
      sku: product.sku,
      barcode: product.barcode,
      minimum_stock: product.minimum_stock,
      maximum_stock: product.maximum_stock,
      reorder_level: product.reorder_level,
      stock: product.stock,
      tax_type: product.tax_type,
      tax_rate: product.tax_rate,
      tax_amount: product.tax_amount,
      is_active: product.is_active,

      // Metadata
      workspace_id: product.workspace_id,
      created_by: product.created_by,
      updated_by: product.updated_by,
      created_at: product.created_at,
      updated_at: product.updated_at,
    }
  }
}

/// Query parameters for paginated requests with advanced filtering
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetProductsQuery {
  // Pagination
  pub page: Option<u32>,
  pub limit: Option<u32>,

  // Basic filtering
  pub search: Option<String>,
  pub category_id: Option<Uuid>,
  pub supplier_id: Option<Uuid>,
  pub is_active: Option<bool>,
  pub track_inventory: Option<bool>,

  // Advanced filtering
  pub code: Option<String>,
  pub sku: Option<String>,
  pub barcode: Option<String>,
  pub base_unit: Option<String>,
  pub tax_type: Option<String>,
  pub include_categories: Option<String>, // comma-separated UUIDs
  pub exclude_categories: Option<String>, // comma-separated UUIDs
  pub include_suppliers: Option<String>,  // comma-separated UUIDs
  pub exclude_suppliers: Option<String>,  // comma-separated UUIDs
  pub include_ids: Option<String>,        // comma-separated UUIDs
  pub exclude_ids: Option<String>,        // comma-separated UUIDs

  // Price filtering
  pub min_selling_price: Option<rust_decimal::Decimal>,
  pub max_selling_price: Option<rust_decimal::Decimal>,
  pub min_unit_cost: Option<rust_decimal::Decimal>,
  pub max_unit_cost: Option<rust_decimal::Decimal>,

  // Stock filtering
  pub min_current_stock: Option<i32>,
  pub max_current_stock: Option<i32>,
  pub low_stock: Option<bool>, // Products with current_stock <= reorder_level

  // Sorting
  pub sort_by: Option<String>,    // "name", "code", "selling_price", "unit_cost", "created_at", "updated_at"
  pub sort_order: Option<String>, // "asc" or "desc"
}

// Constants for consistency with handler
const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 10;

#[derive(Debug, Clone)]
pub struct ProductFilters {
  pub search: Option<String>,
  pub category_id: Option<Uuid>,
  pub supplier_id: Option<Uuid>,
  pub is_active: Option<bool>,
  pub track_inventory: Option<bool>,
  pub code: Option<String>,
  pub sku: Option<String>,
  pub barcode: Option<String>,
  pub base_unit: Option<String>,
  pub tax_type: Option<String>,
  pub include_categories: Vec<Uuid>,
  pub exclude_categories: Vec<Uuid>,
  pub include_suppliers: Vec<Uuid>,
  pub exclude_suppliers: Vec<Uuid>,
  pub include_ids: Vec<Uuid>,
  pub exclude_ids: Vec<Uuid>,

  // Price filtering
  pub min_selling_price: Option<rust_decimal::Decimal>,
  pub max_selling_price: Option<rust_decimal::Decimal>,
  pub min_unit_cost: Option<rust_decimal::Decimal>,
  pub max_unit_cost: Option<rust_decimal::Decimal>,

  // Stock filtering
  pub min_current_stock: Option<i32>,
  pub max_current_stock: Option<i32>,
  pub low_stock: Option<bool>,

  pub sort_by: String,
  pub sort_order: String,
}

impl From<GetProductsQuery> for ProductFilters {
  fn from(query: GetProductsQuery) -> Self {
    // Parse include/exclude categories
    let include_categories = query
      .include_categories
      .map(|s| s.split(',').filter_map(|id| Uuid::parse_str(id.trim()).ok()).collect())
      .unwrap_or_default();

    let exclude_categories = query
      .exclude_categories
      .map(|s| s.split(',').filter_map(|id| Uuid::parse_str(id.trim()).ok()).collect())
      .unwrap_or_default();

    // Parse include/exclude suppliers
    let include_suppliers = query
      .include_suppliers
      .map(|s| s.split(',').filter_map(|id| Uuid::parse_str(id.trim()).ok()).collect())
      .unwrap_or_default();

    let exclude_suppliers = query
      .exclude_suppliers
      .map(|s| s.split(',').filter_map(|id| Uuid::parse_str(id.trim()).ok()).collect())
      .unwrap_or_default();

    // Parse include/exclude IDs
    let include_ids = query
      .include_ids
      .map(|s| s.split(',').filter_map(|id| Uuid::parse_str(id.trim()).ok()).collect())
      .unwrap_or_default();

    let exclude_ids = query
      .exclude_ids
      .map(|s| s.split(',').filter_map(|id| Uuid::parse_str(id.trim()).ok()).collect())
      .unwrap_or_default();

    // Validate and set sort parameters
    let sort_by = match query.sort_by.as_deref() {
      Some("name") => "name",
      Some("code") => "code",
      Some("selling_price") => "selling_price",
      Some("unit_cost") => "unit_cost",
      Some("created_at") => "created_at",
      Some("updated_at") => "updated_at",
      _ => "created_at", // default
    }
    .to_string();

    let sort_order = match query.sort_order.as_deref() {
      Some("asc") | Some("ASC") => "ASC",
      Some("desc") | Some("DESC") => "DESC",
      _ => "DESC", // default
    }
    .to_string();

    Self {
      search: query.search,
      category_id: query.category_id,
      supplier_id: query.supplier_id,
      is_active: query.is_active,
      track_inventory: query.track_inventory,
      code: query.code,
      sku: query.sku,
      barcode: query.barcode,
      base_unit: query.base_unit,
      tax_type: query.tax_type,
      include_categories,
      exclude_categories,
      include_suppliers,
      exclude_suppliers,
      include_ids,
      exclude_ids,
      min_selling_price: query.min_selling_price,
      max_selling_price: query.max_selling_price,
      min_unit_cost: query.min_unit_cost,
      max_unit_cost: query.max_unit_cost,
      min_current_stock: query.min_current_stock,
      max_current_stock: query.max_current_stock,
      low_stock: query.low_stock,
      sort_by,
      sort_order,
    }
  }
}

impl Default for GetProductsQuery {
  fn default() -> Self {
    Self {
      page: Some(DEFAULT_PAGE),
      limit: Some(DEFAULT_LIMIT),
      search: None,
      category_id: None,
      supplier_id: None,
      is_active: None,
      track_inventory: None,
      code: None,
      sku: None,
      barcode: None,
      base_unit: None,
      tax_type: None,
      include_categories: None,
      exclude_categories: None,
      include_suppliers: None,
      exclude_suppliers: None,
      include_ids: None,
      exclude_ids: None,
      min_selling_price: None,
      max_selling_price: None,
      min_unit_cost: None,
      max_unit_cost: None,
      min_current_stock: None,
      max_current_stock: None,
      low_stock: None,
      sort_by: None,
      sort_order: None,
    }
  }
}
