use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

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
  pub selling_price: Decimal,
  pub unit_cost: Decimal,
  pub supplier_id: Option<Uuid>,
  pub track_inventory: bool,

  // Additional fields
  pub description: Option<String>,
  pub sku: Option<String>,
  pub barcode: Option<String>,
  pub minimum_stock: Option<i32>,
  pub maximum_stock: Option<i32>,
  pub reorder_level: Option<i32>,
  pub current_stock: Option<i32>,
  pub tax_type: Option<String>,
  pub tax_rate: Option<Decimal>,
  pub tax_amount: Option<Decimal>,
  pub is_active: bool,

  // Metadata
  pub workspace_id: Option<Uuid>,
  pub created_by: Option<Uuid>,
  pub updated_by: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// Represents a product category record in the database.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProductCategory {
  pub id: Uuid,
  pub code: String,
  pub name: String,
  pub description: Option<String>,
  pub parent_id: Option<Uuid>,
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
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
  #[validate(length(min = 1, max = 20, message = "Code is required and must be max 20 characters"))]
  pub code: String,

  #[validate(length(min = 1, max = 255, message = "Name is required and must be max 255 characters"))]
  pub name: String,

  pub category_id: Option<Uuid>,

  #[validate(length(min = 1, max = 50, message = "Base unit is required"))]
  pub base_unit: String,

  pub unit_on_report_preview: Option<String>,
  pub selling_price: Option<Decimal>,
  pub unit_cost: Option<Decimal>,
  pub supplier_id: Option<Uuid>,
  pub track_inventory: Option<bool>,

  // Additional fields
  pub description: Option<String>,

  #[validate(length(max = 100, message = "SKU must be max 100 characters"))]
  pub sku: Option<String>,

  #[validate(length(max = 100, message = "Barcode must be max 100 characters"))]
  pub barcode: Option<String>,

  pub minimum_stock: Option<i32>,
  pub maximum_stock: Option<i32>,
  pub reorder_level: Option<i32>,
  pub current_stock: Option<i32>,

  #[validate(custom(function = "validate_tax_type"))]
  pub tax_type: Option<String>,

  pub tax_rate: Option<Decimal>,
  pub tax_amount: Option<Decimal>,
  pub is_active: Option<bool>,

  pub workspace_id: Option<Uuid>,
}

/// Represents the payload for updating an existing product.
/// All fields are optional, allowing for partial updates.
/// The `updated_by` field is automatically set from the authenticated user.
#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
  pub code: Option<String>,
  pub name: Option<String>,
  pub category_id: Option<Uuid>,
  pub base_unit: Option<String>,
  pub unit_on_report_preview: Option<String>,
  pub selling_price: Option<Decimal>,
  pub unit_cost: Option<Decimal>,
  pub supplier_id: Option<Uuid>,
  pub track_inventory: Option<bool>,
  pub description: Option<String>,
  pub sku: Option<String>,
  pub barcode: Option<String>,
  pub minimum_stock: Option<i32>,
  pub maximum_stock: Option<i32>,
  pub reorder_level: Option<i32>,
  pub current_stock: Option<i32>,
  pub tax_type: Option<String>,
  pub tax_rate: Option<Decimal>,
  pub tax_amount: Option<Decimal>,
  pub is_active: Option<bool>,

  pub workspace_id: Option<Uuid>,
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
  pub selling_price: Decimal,
  pub unit_cost: Decimal,
  pub supplier_id: Option<Uuid>,
  pub track_inventory: bool,
  pub description: Option<String>,
  pub sku: Option<String>,
  pub barcode: Option<String>,
  pub minimum_stock: Option<i32>,
  pub maximum_stock: Option<i32>,
  pub reorder_level: Option<i32>,
  pub current_stock: Option<i32>,
  pub tax_type: Option<String>,
  pub tax_rate: Option<Decimal>,
  pub tax_amount: Option<Decimal>,
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
      description: product.description,
      sku: product.sku,
      barcode: product.barcode,
      minimum_stock: product.minimum_stock,
      maximum_stock: product.maximum_stock,
      reorder_level: product.reorder_level,
      current_stock: product.current_stock,
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

/// Query parameters for paginated requests
#[derive(Debug, serde::Deserialize)]
pub struct GetProductsQuery {
  pub page: Option<u32>,
  pub limit: Option<u32>,
  pub search: Option<String>,
  pub category_id: Option<String>,
  pub is_active: Option<bool>,
}

// Constants untuk consistency dengan handler
const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 10;

impl Default for GetProductsQuery {
  fn default() -> Self {
    Self {
      page: Some(DEFAULT_PAGE),
      limit: Some(DEFAULT_LIMIT),
      search: None,
      category_id: None,
      is_active: None,
    }
  }
}

/// Custom validator for tax_type field
/// Validates that tax_type is either "percentage" or "fixed_amount" when provided
fn validate_tax_type(tax_type: &str) -> Result<(), validator::ValidationError> {
  let valid_tax_types = vec!["percentage", "fixed_amount"];

  if valid_tax_types.contains(&tax_type) {
    Ok(())
  } else {
    let mut error = validator::ValidationError::new("invalid_tax_type");
    error.message = Some("Tax type must be either 'percentage' or 'fixed_amount'".into());
    Err(error)
  }
}
