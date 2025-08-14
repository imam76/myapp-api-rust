use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::product_models::{CreateProductRequest, Product, ProductFilters, UpdateProductRequest};
use crate::{
  AppResult,
  utils::code_generator::{CodeGenerator, CodeGeneratorConfig},
};

#[async_trait]
pub trait ProductRepository {
  // Core workspace-scoped methods - these are the only ones we need
  async fn create_by_workspace(&self, product: CreateProductRequest, workspace_id: Uuid, user_id: Uuid) -> AppResult<Product>;
  async fn find_all_by_workspace_paginated(&self, workspace_id: Uuid, user_id: Uuid, page: u32, limit: u32) -> AppResult<(Vec<Product>, u64)>;
  async fn find_by_id_and_workspace(&self, id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Option<Product>>;
  async fn find_by_code_and_workspace(&self, code: &str, workspace_id: Uuid) -> AppResult<Option<Product>>;
  async fn update_by_workspace(
    &self,
    id: Uuid,
    workspace_id: Uuid,
    product_data: UpdateProductRequest,
    updated_by: Uuid,
  ) -> AppResult<Option<Product>>;
  async fn delete_by_workspace_and_user(&self, id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<bool>;

  // Code generation methods
  async fn get_next_available_code(&self, workspace_id: Uuid, product_name: &str) -> AppResult<String>;
  async fn code_exists(&self, code: &str, workspace_id: Uuid) -> AppResult<bool>;

  // Optional methods for specific use cases
  async fn find_by_category_and_workspace(&self, category_id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>>;
  async fn find_by_supplier_and_workspace(&self, supplier_id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>>;
  async fn find_active_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>>;
  async fn find_low_stock_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>>;

  // Advanced filtering method
  async fn find_by_filters_paginated(
    &self,
    workspace_id: Uuid,
    user_id: Uuid,
    page: u32,
    limit: u32,
    filters: ProductFilters,
  ) -> AppResult<(Vec<Product>, u64)>;
}

pub struct SqlxProductRepository {
  db: PgPool,
}

impl SqlxProductRepository {
  pub fn new(db: PgPool) -> Self {
    Self { db }
  }

  /// Get access to the underlying database pool
  pub fn get_pool(&self) -> PgPool {
    self.db.clone()
  }
}

#[async_trait]
impl ProductRepository for SqlxProductRepository {
  // Workspace-scoped methods

  async fn create_by_workspace(&self, product: CreateProductRequest, workspace_id: Uuid, user_id: Uuid) -> AppResult<Product> {
    let new_product = sqlx::query_as!(
      Product,
      r#"
                INSERT INTO products (
                    code, name, category_id, base_unit, unit_on_report_preview,
                    selling_price, unit_cost, supplier_id, track_inventory,
                    description, sku, barcode, minimum_stock, maximum_stock,
                    reorder_level, current_stock, tax_type, tax_rate, tax_amount,
                    workspace_id, created_by
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
                RETURNING 
                    id, code, name, category_id, base_unit, unit_on_report_preview,
                    selling_price, unit_cost, supplier_id, track_inventory,
                    description, sku, barcode, minimum_stock, maximum_stock,
                    reorder_level, current_stock, tax_type, tax_rate, tax_amount,
                    is_active, workspace_id, created_by, updated_by, created_at, updated_at
            "#,
      product.code,
      product.name,
      product.category_id,
      product.base_unit,
      product.unit_on_report_preview,
      product.selling_price,
      product.unit_cost,
      product.supplier_id,
      product.track_inventory.unwrap_or(false),
      product.description,
      product.sku,
      product.barcode,
      product.minimum_stock,
      product.maximum_stock,
      product.reorder_level,
      product.current_stock,
      product.tax_type,
      product.tax_rate,
      product.tax_amount,
      workspace_id,
      user_id
    )
    .fetch_one(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to create product: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "INSERT INTO products")
    })?;

    Ok(new_product)
  }

  async fn find_all_by_workspace_paginated(&self, workspace_id: Uuid, user_id: Uuid, page: u32, limit: u32) -> AppResult<(Vec<Product>, u64)> {
    let offset = (page - 1) * limit;

    let total_count = sqlx::query_scalar!(
      r#"
                SELECT COUNT(*) 
                FROM products 
                WHERE workspace_id = $1
                  AND id IN (
                    SELECT p.id FROM products p
                    JOIN workspaces w ON p.workspace_id = w.id
                    JOIN workspace_users wu ON w.id = wu.workspace_id
                    WHERE wu.user_id = $2
                  )
            "#,
      workspace_id,
      user_id
    )
    .fetch_one(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to count products: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "COUNT products")
    })?
    .unwrap_or(0) as u64;

    let products = sqlx::query_as!(
      Product,
      r#"
                SELECT 
                    id, code, name, category_id, base_unit, unit_on_report_preview,
                    selling_price, unit_cost, supplier_id, track_inventory,
                    description, sku, barcode, minimum_stock, maximum_stock,
                    reorder_level, current_stock, tax_type, tax_rate, tax_amount,
                    is_active, workspace_id, created_by, updated_by, created_at, updated_at
                FROM products
                WHERE workspace_id = $1
                  AND id IN (
                    SELECT p.id FROM products p
                    JOIN workspaces w ON p.workspace_id = w.id
                    JOIN workspace_users wu ON w.id = wu.workspace_id
                    WHERE wu.user_id = $2
                  )
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
            "#,
      workspace_id,
      user_id,
      limit as i64,
      offset as i64
    )
    .fetch_all(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to fetch products: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "SELECT FROM products")
    })?;

    Ok((products, total_count))
  }

  async fn find_by_id_and_workspace(&self, id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Option<Product>> {
    let product = sqlx::query_as!(
      Product,
      r#"
                SELECT 
                    id, code, name, category_id, base_unit, unit_on_report_preview,
                    selling_price, unit_cost, supplier_id, track_inventory,
                    description, sku, barcode, minimum_stock, maximum_stock,
                    reorder_level, current_stock, tax_type, tax_rate, tax_amount,
                    is_active, workspace_id, created_by, updated_by, created_at, updated_at
                FROM products 
                WHERE id = $1 AND workspace_id = $2
                  AND id IN (
                    SELECT p.id FROM products p
                    JOIN workspaces w ON p.workspace_id = w.id
                    JOIN workspace_users wu ON w.id = wu.workspace_id
                    WHERE wu.user_id = $3
                  )
            "#,
      id,
      workspace_id,
      user_id
    )
    .fetch_optional(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to fetch product by id: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "SELECT FROM products WHERE id")
    })?;

    Ok(product)
  }

  async fn find_by_code_and_workspace(&self, code: &str, workspace_id: Uuid) -> AppResult<Option<Product>> {
    let product = sqlx::query_as!(
      Product,
      r#"
                SELECT 
                    id, code, name, category_id, base_unit, unit_on_report_preview,
                    selling_price, unit_cost, supplier_id, track_inventory,
                    description, sku, barcode, minimum_stock, maximum_stock,
                    reorder_level, current_stock, tax_type, tax_rate, tax_amount,
                    is_active, workspace_id, created_by, updated_by, created_at, updated_at
                FROM products 
                WHERE code = $1 AND workspace_id = $2
            "#,
      code,
      workspace_id
    )
    .fetch_optional(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to fetch product by code: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "SELECT FROM products WHERE code")
    })?;

    Ok(product)
  }

  async fn update_by_workspace(
    &self,
    id: Uuid,
    workspace_id: Uuid,
    product_data: UpdateProductRequest,
    updated_by: Uuid,
  ) -> AppResult<Option<Product>> {
    let updated_product = sqlx::query_as!(
      Product,
      r#"
                UPDATE products 
                SET 
                    code = COALESCE($3, code),
                    name = COALESCE($4, name),
                    category_id = COALESCE($5, category_id),
                    base_unit = COALESCE($6, base_unit),
                    unit_on_report_preview = COALESCE($7, unit_on_report_preview),
                    selling_price = COALESCE($8, selling_price),
                    unit_cost = COALESCE($9, unit_cost),
                    supplier_id = COALESCE($10, supplier_id),
                    track_inventory = COALESCE($11, track_inventory),
                    description = COALESCE($12, description),
                    sku = COALESCE($13, sku),
                    barcode = COALESCE($14, barcode),
                    minimum_stock = COALESCE($15, minimum_stock),
                    maximum_stock = COALESCE($16, maximum_stock),
                    reorder_level = COALESCE($17, reorder_level),
                    current_stock = COALESCE($18, current_stock),
                    tax_type = COALESCE($19, tax_type),
                    tax_rate = COALESCE($20, tax_rate),
                    tax_amount = COALESCE($21, tax_amount),
                    is_active = COALESCE($22, is_active),
                    updated_by = $23,
                    updated_at = NOW()
                WHERE id = $1 AND workspace_id = $2
                RETURNING 
                    id, code, name, category_id, base_unit, unit_on_report_preview,
                    selling_price, unit_cost, supplier_id, track_inventory,
                    description, sku, barcode, minimum_stock, maximum_stock,
                    reorder_level, current_stock, tax_type, tax_rate, tax_amount,
                    is_active, workspace_id, created_by, updated_by, created_at, updated_at
            "#,
      id,
      workspace_id,
      product_data.code,
      product_data.name,
      product_data.category_id,
      product_data.base_unit,
      product_data.unit_on_report_preview,
      product_data.selling_price,
      product_data.unit_cost,
      product_data.supplier_id,
      product_data.track_inventory,
      product_data.description,
      product_data.sku,
      product_data.barcode,
      product_data.minimum_stock,
      product_data.maximum_stock,
      product_data.reorder_level,
      product_data.current_stock,
      product_data.tax_type,
      product_data.tax_rate,
      product_data.tax_amount,
      product_data.is_active,
      updated_by
    )
    .fetch_optional(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to update product: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "UPDATE products")
    })?;

    Ok(updated_product)
  }

  async fn delete_by_workspace_and_user(&self, id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query!(
      r#"
                DELETE FROM products 
                WHERE id = $1 AND workspace_id = $2
                  AND id IN (
                    SELECT p.id FROM products p
                    JOIN workspaces w ON p.workspace_id = w.id
                    JOIN workspace_users wu ON w.id = wu.workspace_id
                    WHERE wu.user_id = $3
                  )
            "#,
      id,
      workspace_id,
      user_id
    )
    .execute(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to delete product: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "DELETE FROM products")
    })?;

    Ok(result.rows_affected() > 0)
  }

  // Code generation methods
  async fn get_next_available_code(&self, workspace_id: Uuid, product_name: &str) -> AppResult<String> {
    let code_generator = CodeGenerator::new(self.db.clone());
    let config = CodeGeneratorConfig {
      table_name: "products".to_string(),
      code_column: "code".to_string(),
      workspace_column: Some("workspace_id".to_string()),
      prefix_length: 2,
      number_length: 5,
      separator: "-".to_string(),
    };

    code_generator.get_next_available_code(&config, product_name, Some(workspace_id)).await
  }

  async fn code_exists(&self, code: &str, workspace_id: Uuid) -> AppResult<bool> {
    let count = sqlx::query_scalar!(
      r#"
                SELECT COUNT(*) 
                FROM products 
                WHERE code = $1 AND workspace_id = $2
            "#,
      code,
      workspace_id
    )
    .fetch_one(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to check if product code exists: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "SELECT COUNT FROM products")
    })?
    .unwrap_or(0);

    Ok(count > 0)
  }

  // Optional methods for specific use cases
  async fn find_by_category_and_workspace(&self, category_id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>> {
    let products = sqlx::query_as!(
      Product,
      r#"
                SELECT 
                    id, code, name, category_id, base_unit, unit_on_report_preview,
                    selling_price, unit_cost, supplier_id, track_inventory,
                    description, sku, barcode, minimum_stock, maximum_stock,
                    reorder_level, current_stock, tax_type, tax_rate, tax_amount,
                    is_active, workspace_id, created_by, updated_by, created_at, updated_at
                FROM products 
                WHERE category_id = $1 AND workspace_id = $2 AND is_active = true
                  AND id IN (
                    SELECT p.id FROM products p
                    JOIN workspaces w ON p.workspace_id = w.id
                    JOIN workspace_users wu ON w.id = wu.workspace_id
                    WHERE wu.user_id = $3
                  )
                ORDER BY name ASC
            "#,
      category_id,
      workspace_id,
      user_id
    )
    .fetch_all(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to fetch products by category: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "SELECT FROM products WHERE category_id")
    })?;

    Ok(products)
  }

  async fn find_by_supplier_and_workspace(&self, supplier_id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>> {
    let products = sqlx::query_as!(
      Product,
      r#"
                SELECT 
                    id, code, name, category_id, base_unit, unit_on_report_preview,
                    selling_price, unit_cost, supplier_id, track_inventory,
                    description, sku, barcode, minimum_stock, maximum_stock,
                    reorder_level, current_stock, tax_type, tax_rate, tax_amount,
                    is_active, workspace_id, created_by, updated_by, created_at, updated_at
                FROM products 
                WHERE supplier_id = $1 AND workspace_id = $2 AND is_active = true
                  AND id IN (
                    SELECT p.id FROM products p
                    JOIN workspaces w ON p.workspace_id = w.id
                    JOIN workspace_users wu ON w.id = wu.workspace_id
                    WHERE wu.user_id = $3
                  )
                ORDER BY name ASC
            "#,
      supplier_id,
      workspace_id,
      user_id
    )
    .fetch_all(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to fetch products by supplier: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "SELECT FROM products WHERE supplier_id")
    })?;

    Ok(products)
  }

  async fn find_active_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>> {
    let products = sqlx::query_as!(
      Product,
      r#"
                SELECT 
                    id, code, name, category_id, base_unit, unit_on_report_preview,
                    selling_price, unit_cost, supplier_id, track_inventory,
                    description, sku, barcode, minimum_stock, maximum_stock,
                    reorder_level, current_stock, tax_type, tax_rate, tax_amount,
                    is_active, workspace_id, created_by, updated_by, created_at, updated_at
                FROM products 
                WHERE workspace_id = $1 AND is_active = true
                  AND id IN (
                    SELECT p.id FROM products p
                    JOIN workspaces w ON p.workspace_id = w.id
                    JOIN workspace_users wu ON w.id = wu.workspace_id
                    WHERE wu.user_id = $2
                  )
                ORDER BY name ASC
            "#,
      workspace_id,
      user_id
    )
    .fetch_all(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to fetch active products: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "SELECT FROM products WHERE is_active")
    })?;

    Ok(products)
  }

  async fn find_low_stock_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>> {
    let products = sqlx::query_as!(
      Product,
      r#"
                SELECT 
                    id, code, name, category_id, base_unit, unit_on_report_preview,
                    selling_price, unit_cost, supplier_id, track_inventory,
                    description, sku, barcode, minimum_stock, maximum_stock,
                    reorder_level, current_stock, tax_type, tax_rate, tax_amount,
                    is_active, workspace_id, created_by, updated_by, created_at, updated_at
                FROM products 
                WHERE workspace_id = $1 
                    AND is_active = true 
                    AND track_inventory = true
                    AND current_stock IS NOT NULL 
                    AND reorder_level IS NOT NULL
                    AND current_stock <= reorder_level
                    AND id IN (
                      SELECT p.id FROM products p
                      JOIN workspaces w ON p.workspace_id = w.id
                      JOIN workspace_users wu ON w.id = wu.workspace_id
                      WHERE wu.user_id = $2
                    )
                ORDER BY current_stock ASC
            "#,
      workspace_id,
      user_id
    )
    .fetch_all(&self.db)
    .await
    .map_err(|e| {
      tracing::error!("Failed to fetch low stock products: {}", e);
      crate::errors::AppError::from_sqlx_error(e, "SELECT FROM products WHERE low stock")
    })?;

    Ok(products)
  }

  // Advanced filtering method
  async fn find_by_filters_paginated(
    &self,
    workspace_id: Uuid,
    user_id: Uuid,
    page: u32,
    limit: u32,
    filters: ProductFilters,
  ) -> AppResult<(Vec<Product>, u64)> {
    let (select_sql, count_sql) = super::product_query_builder::ProductQueryBuilder::build_filtered_query(workspace_id, user_id, &filters);

    // Execute count query
    let total_count_result = sqlx::query_scalar::<_, i64>(&count_sql)
      .bind(workspace_id)
      .fetch_one(&self.db)
      .await
      .map_err(|e| {
        tracing::error!("Failed to count filtered products: {}", e);
        crate::errors::AppError::from_sqlx_error(e, "COUNT filtered products")
      })?;

    let total_count = total_count_result as u64;

    // Calculate pagination
    let offset = (page - 1) * limit;

    // Execute select query with pagination
    let final_query = format!("{} LIMIT {} OFFSET {}", select_sql, limit, offset);

    let products = sqlx::query_as::<_, Product>(&final_query)
      .bind(workspace_id)
      .fetch_all(&self.db)
      .await
      .map_err(|e| {
        tracing::error!("Failed to fetch filtered products: {}", e);
        crate::errors::AppError::from_sqlx_error(e, "SELECT filtered products")
      })?;

    Ok((products, total_count))
  }
}
