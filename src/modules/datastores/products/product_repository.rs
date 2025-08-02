use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::product_models::{CreateProductRequest, Product, ProductCategory, UpdateProductRequest};
use crate::AppResult;

#[async_trait]
pub trait ProductRepository {
  async fn find_all(&self) -> AppResult<Vec<Product>>;
  async fn find_all_paginated(&self, page: u32, limit: u32) -> AppResult<(Vec<Product>, u64)>;
  async fn find_all_by_user(&self, user_id: Uuid) -> AppResult<Vec<Product>>;
  async fn find_all_by_user_paginated(&self, user_id: Uuid, page: u32, limit: u32) -> AppResult<(Vec<Product>, u64)>;
  async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Product>>;
  async fn find_by_id_and_user(&self, id: Uuid, user_id: Uuid) -> AppResult<Option<Product>>;
  async fn find_by_code(&self, code: &str) -> AppResult<Option<Product>>;
  async fn find_by_sku(&self, sku: &str) -> AppResult<Option<Product>>;
  async fn find_by_barcode(&self, barcode: &str) -> AppResult<Option<Product>>;
  async fn create(&self, product: CreateProductRequest, user_id: Uuid) -> AppResult<Product>;
  async fn update(&self, id: Uuid, product: UpdateProductRequest, user_id: Uuid) -> AppResult<Option<Product>>;
  async fn delete(&self, id: Uuid, user_id: Uuid) -> AppResult<bool>;
  async fn find_by_category(&self, category_id: Uuid) -> AppResult<Vec<Product>>;
  async fn find_by_category_and_user(&self, category_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>>;
  async fn find_active(&self) -> AppResult<Vec<Product>>;
  async fn find_active_by_user(&self, user_id: Uuid) -> AppResult<Vec<Product>>;
  async fn find_low_stock(&self, user_id: Uuid) -> AppResult<Vec<Product>>;

  // Workspace-scoped methods
  async fn find_all_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>>;
  async fn find_all_by_workspace_paginated(&self, workspace_id: Uuid, user_id: Uuid, page: u32, limit: u32) -> AppResult<(Vec<Product>, u64)>;
  async fn find_by_id_and_workspace(&self, id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Option<Product>>;
  async fn find_by_category_and_workspace(&self, category_id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>>;
  async fn find_by_category_and_workspace_paginated(
    &self,
    category_id: Uuid,
    workspace_id: Uuid,
    user_id: Uuid,
    page: u32,
    limit: u32,
  ) -> AppResult<(Vec<Product>, u64)>;
  async fn find_by_code_and_workspace(&self, code: &str, workspace_id: Uuid) -> AppResult<Option<Product>>;
  async fn update_by_workspace(
    &self,
    id: Uuid,
    workspace_id: Uuid,
    product_data: UpdateProductRequest,
    updated_by: Uuid,
  ) -> AppResult<Option<Product>>;
  async fn delete_by_workspace(&self, id: Uuid, workspace_id: Uuid) -> AppResult<bool>;
  async fn find_active_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>>;
  async fn find_low_stock_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>>;

  // Product Categories methods
  async fn find_all_categories(&self) -> AppResult<Vec<ProductCategory>>;
  async fn find_category_by_id(&self, id: Uuid) -> AppResult<Option<ProductCategory>>;
  async fn find_category_by_code(&self, code: &str) -> AppResult<Option<ProductCategory>>;
  async fn find_active_categories(&self) -> AppResult<Vec<ProductCategory>>;
}

pub struct SqlxProductRepository {
  db: PgPool,
}

impl SqlxProductRepository {
  pub fn new(db: PgPool) -> Self {
    Self { db }
  }
}

#[async_trait]
impl ProductRepository for SqlxProductRepository {
  async fn find_all(&self) -> AppResult<Vec<Product>> {
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
        ORDER BY created_at DESC
      "#
    )
    .fetch_all(&self.db)
    .await?;

    Ok(products)
  }

  async fn find_all_paginated(&self, page: u32, limit: u32) -> AppResult<(Vec<Product>, u64)> {
    let offset = (page - 1) * limit;
    let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
      .fetch_one(&self.db)
      .await?
      .unwrap_or(0);

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
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
      "#,
      limit as i64,
      offset as i64
    )
    .fetch_all(&self.db)
    .await?;

    Ok((products, total_count as u64))
  }

  async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Product>> {
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
        WHERE id = $1
      "#,
      id
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(product)
  }

  async fn find_by_code(&self, code: &str) -> AppResult<Option<Product>> {
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
        WHERE code = $1
      "#,
      code
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(product)
  }

  async fn find_by_sku(&self, sku: &str) -> AppResult<Option<Product>> {
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
        WHERE sku = $1
      "#,
      sku
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(product)
  }

  async fn find_by_barcode(&self, barcode: &str) -> AppResult<Option<Product>> {
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
        WHERE barcode = $1
      "#,
      barcode
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(product)
  }

  async fn create(&self, product: CreateProductRequest, user_id: Uuid) -> AppResult<Product> {
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
      product.selling_price.unwrap_or_default(),
      product.unit_cost.unwrap_or_default(),
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
      product.workspace_id,
      user_id
    )
    .fetch_one(&self.db)
    .await?;

    Ok(new_product)
  }

  async fn update(&self, id: Uuid, product: UpdateProductRequest, user_id: Uuid) -> AppResult<Option<Product>> {
    // First check if product exists and belongs to user
    let existing = self.find_by_id_and_user(id, user_id).await?;
    if existing.is_none() {
      return Ok(None);
    }

    let updated_product = sqlx::query_as!(
      Product,
      r#"
        UPDATE products 
        SET 
          code = COALESCE($1, code),
          name = COALESCE($2, name),
          category_id = COALESCE($3, category_id),
          base_unit = COALESCE($4, base_unit),
          unit_on_report_preview = COALESCE($5, unit_on_report_preview),
          selling_price = COALESCE($6, selling_price),
          unit_cost = COALESCE($7, unit_cost),
          supplier_id = COALESCE($8, supplier_id),
          track_inventory = COALESCE($9, track_inventory),
          description = COALESCE($10, description),
          sku = COALESCE($11, sku),
          barcode = COALESCE($12, barcode),
          minimum_stock = COALESCE($13, minimum_stock),
          maximum_stock = COALESCE($14, maximum_stock),
          reorder_level = COALESCE($15, reorder_level),
          current_stock = COALESCE($16, current_stock),
          tax_type = COALESCE($17, tax_type),
          tax_rate = COALESCE($18, tax_rate),
          tax_amount = COALESCE($19, tax_amount),
          is_active = COALESCE($20, is_active),
          workspace_id = COALESCE($21, workspace_id),
          updated_by = $22,
          updated_at = NOW()
        WHERE id = $23 AND created_by = $24
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
      product.track_inventory,
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
      product.is_active,
      product.workspace_id,
      user_id,
      id,
      user_id
    )
    .fetch_one(&self.db)
    .await?;

    Ok(Some(updated_product))
  }

  async fn delete(&self, id: Uuid, user_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query!("DELETE FROM products WHERE id = $1 AND created_by = $2", id, user_id)
      .execute(&self.db)
      .await?;

    Ok(result.rows_affected() > 0)
  }

  async fn find_by_category(&self, category_id: Uuid) -> AppResult<Vec<Product>> {
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
        WHERE category_id = $1 AND is_active = true
        ORDER BY created_at DESC
      "#,
      category_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(products)
  }

  async fn find_active(&self) -> AppResult<Vec<Product>> {
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
        WHERE is_active = true
        ORDER BY created_at DESC
      "#
    )
    .fetch_all(&self.db)
    .await?;

    Ok(products)
  }

  // User-specific methods for data isolation
  async fn find_all_by_user(&self, user_id: Uuid) -> AppResult<Vec<Product>> {
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
        WHERE created_by = $1
        ORDER BY created_at DESC
      "#,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(products)
  }

  async fn find_all_by_user_paginated(&self, user_id: Uuid, page: u32, limit: u32) -> AppResult<(Vec<Product>, u64)> {
    let offset = (page - 1) * limit;

    // Get total count for pagination
    let total_result = sqlx::query!("SELECT COUNT(*) FROM products WHERE created_by = $1", user_id)
      .fetch_one(&self.db)
      .await?;

    let total = total_result.count.unwrap_or(0) as u64;

    // Get paginated results
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
        WHERE created_by = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
      "#,
      user_id,
      limit as i64,
      offset as i64
    )
    .fetch_all(&self.db)
    .await?;

    Ok((products, total))
  }

  async fn find_by_id_and_user(&self, id: Uuid, user_id: Uuid) -> AppResult<Option<Product>> {
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
        WHERE id = $1 AND created_by = $2
      "#,
      id,
      user_id
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(product)
  }

  async fn find_by_category_and_user(&self, category_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>> {
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
        WHERE category_id = $1 AND created_by = $2 AND is_active = true
        ORDER BY created_at DESC
      "#,
      category_id,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(products)
  }

  async fn find_active_by_user(&self, user_id: Uuid) -> AppResult<Vec<Product>> {
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
        WHERE created_by = $1 AND is_active = true
        ORDER BY created_at DESC
      "#,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(products)
  }

  async fn find_low_stock(&self, user_id: Uuid) -> AppResult<Vec<Product>> {
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
        WHERE created_by = $1 
          AND is_active = true 
          AND track_inventory = true
          AND current_stock <= reorder_level
        ORDER BY current_stock ASC
      "#,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(products)
  }

  // Workspace-scoped methods
  async fn find_all_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>> {
    let products = sqlx::query_as!(
      Product,
      r#"
        SELECT 
          p.id, p.code, p.name, p.category_id, p.base_unit, p.unit_on_report_preview,
          p.selling_price, p.unit_cost, p.supplier_id, p.track_inventory,
          p.description, p.sku, p.barcode, p.minimum_stock, p.maximum_stock,
          p.reorder_level, p.current_stock, p.tax_type, p.tax_rate, p.tax_amount,
          p.is_active, p.workspace_id, p.created_by, p.updated_by, p.created_at, p.updated_at
        FROM products p
        JOIN workspaces w ON w.id = $1
        JOIN workspace_users wu ON w.id = wu.workspace_id
        WHERE wu.user_id = $2
        ORDER BY p.created_at DESC
      "#,
      workspace_id,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(products)
  }

  async fn find_all_by_workspace_paginated(&self, workspace_id: Uuid, user_id: Uuid, page: u32, limit: u32) -> AppResult<(Vec<Product>, u64)> {
    let offset = (page - 1) * limit;

    let total_count = sqlx::query_scalar!(
      r#"
        SELECT COUNT(*) 
        FROM products p
        JOIN workspaces w ON w.id = $1
        JOIN workspace_users wu ON w.id = wu.workspace_id
        WHERE wu.user_id = $2
      "#,
      workspace_id,
      user_id
    )
    .fetch_one(&self.db)
    .await?
    .unwrap_or(0);

    let products = sqlx::query_as!(
      Product,
      r#"
        SELECT 
          p.id, p.code, p.name, p.category_id, p.base_unit, p.unit_on_report_preview,
          p.selling_price, p.unit_cost, p.supplier_id, p.track_inventory,
          p.description, p.sku, p.barcode, p.minimum_stock, p.maximum_stock,
          p.reorder_level, p.current_stock, p.tax_type, p.tax_rate, p.tax_amount,
          p.is_active, p.workspace_id, p.created_by, p.updated_by, p.created_at, p.updated_at
        FROM products p
        JOIN workspaces w ON w.id = $1
        JOIN workspace_users wu ON w.id = wu.workspace_id
        WHERE wu.user_id = $2
        ORDER BY p.created_at DESC
        LIMIT $3 OFFSET $4
      "#,
      workspace_id,
      user_id,
      limit as i64,
      offset as i64
    )
    .fetch_all(&self.db)
    .await?;

    Ok((products, total_count as u64))
  }

  async fn find_by_id_and_workspace(&self, id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Option<Product>> {
    let product = sqlx::query_as!(
      Product,
      r#"
        SELECT 
          p.id, p.code, p.name, p.category_id, p.base_unit, p.unit_on_report_preview,
          p.selling_price, p.unit_cost, p.supplier_id, p.track_inventory,
          p.description, p.sku, p.barcode, p.minimum_stock, p.maximum_stock,
          p.reorder_level, p.current_stock, p.tax_type, p.tax_rate, p.tax_amount,
          p.is_active, p.workspace_id, p.created_by, p.updated_by, p.created_at, p.updated_at
        FROM products p
        JOIN workspaces w ON w.id = $2
        JOIN workspace_users wu ON w.id = wu.workspace_id
        WHERE p.id = $1 AND wu.user_id = $3
      "#,
      id,
      workspace_id,
      user_id
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(product)
  }

  async fn find_by_category_and_workspace(&self, category_id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>> {
    let products = sqlx::query_as!(
      Product,
      r#"
        SELECT 
          p.id, p.code, p.name, p.category_id, p.base_unit, p.unit_on_report_preview,
          p.selling_price, p.unit_cost, p.supplier_id, p.track_inventory,
          p.description, p.sku, p.barcode, p.minimum_stock, p.maximum_stock,
          p.reorder_level, p.current_stock, p.tax_type, p.tax_rate, p.tax_amount,
          p.is_active, p.workspace_id, p.created_by, p.updated_by, p.created_at, p.updated_at
        FROM products p
        JOIN workspaces w ON w.id = $2
        JOIN workspace_users wu ON w.id = wu.workspace_id
        WHERE p.category_id = $1 AND wu.user_id = $3
        ORDER BY p.created_at DESC
      "#,
      category_id,
      workspace_id,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(products)
  }

  async fn find_active_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>> {
    let products = sqlx::query_as!(
      Product,
      r#"
        SELECT 
          p.id, p.code, p.name, p.category_id, p.base_unit, p.unit_on_report_preview,
          p.selling_price, p.unit_cost, p.supplier_id, p.track_inventory,
          p.description, p.sku, p.barcode, p.minimum_stock, p.maximum_stock,
          p.reorder_level, p.current_stock, p.tax_type, p.tax_rate, p.tax_amount,
          p.is_active, p.workspace_id, p.created_by, p.updated_by, p.created_at, p.updated_at
        FROM products p
        JOIN workspaces w ON w.id = $1
        JOIN workspace_users wu ON w.id = wu.workspace_id
        WHERE wu.user_id = $2 AND p.is_active = true
        ORDER BY p.created_at DESC
      "#,
      workspace_id,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(products)
  }

  async fn find_low_stock_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Product>> {
    let products = sqlx::query_as!(
      Product,
      r#"
        SELECT 
          p.id, p.code, p.name, p.category_id, p.base_unit, p.unit_on_report_preview,
          p.selling_price, p.unit_cost, p.supplier_id, p.track_inventory,
          p.description, p.sku, p.barcode, p.minimum_stock, p.maximum_stock,
          p.reorder_level, p.current_stock, p.tax_type, p.tax_rate, p.tax_amount,
          p.is_active, p.workspace_id, p.created_by, p.updated_by, p.created_at, p.updated_at
        FROM products p
        JOIN workspaces w ON w.id = $1
        JOIN workspace_users wu ON w.id = wu.workspace_id
        WHERE wu.user_id = $2 
          AND p.is_active = true 
          AND p.track_inventory = true
          AND p.current_stock <= p.reorder_level
        ORDER BY p.current_stock ASC
      "#,
      workspace_id,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(products)
  }

  async fn find_by_category_and_workspace_paginated(
    &self,
    category_id: Uuid,
    workspace_id: Uuid,
    user_id: Uuid,
    page: u32,
    limit: u32,
  ) -> AppResult<(Vec<Product>, u64)> {
    let offset = (page - 1) * limit;

    // Get total count
    let total = sqlx::query_scalar!(
      r#"
        SELECT COUNT(*) 
        FROM products p
        JOIN workspaces w ON w.id = $2
        JOIN workspace_users wu ON w.id = wu.workspace_id
        WHERE p.category_id = $1 AND wu.user_id = $3
      "#,
      category_id,
      workspace_id,
      user_id
    )
    .fetch_one(&self.db)
    .await?
    .unwrap_or(0) as u64;

    // Get paginated products
    let products = sqlx::query_as!(
      Product,
      r#"
        SELECT 
          p.id, p.code, p.name, p.category_id, p.base_unit, p.unit_on_report_preview,
          p.selling_price, p.unit_cost, p.supplier_id, p.track_inventory,
          p.description, p.sku, p.barcode, p.minimum_stock, p.maximum_stock,
          p.reorder_level, p.current_stock, p.tax_type, p.tax_rate, p.tax_amount,
          p.is_active, p.workspace_id, p.created_by, p.updated_by, p.created_at, p.updated_at
        FROM products p
        JOIN workspaces w ON w.id = $2
        JOIN workspace_users wu ON w.id = wu.workspace_id
        WHERE p.category_id = $1 AND wu.user_id = $3
        ORDER BY p.created_at DESC
        LIMIT $4 OFFSET $5
      "#,
      category_id,
      workspace_id,
      user_id,
      limit as i64,
      offset as i64
    )
    .fetch_all(&self.db)
    .await?;

    Ok((products, total))
  }

  async fn find_by_code_and_workspace(&self, code: &str, workspace_id: Uuid) -> AppResult<Option<Product>> {
    let product = sqlx::query_as!(
      Product,
      r#"
        SELECT 
          p.id, p.code, p.name, p.category_id, p.base_unit, p.unit_on_report_preview,
          p.selling_price, p.unit_cost, p.supplier_id, p.track_inventory,
          p.description, p.sku, p.barcode, p.minimum_stock, p.maximum_stock,
          p.reorder_level, p.current_stock, p.tax_type, p.tax_rate, p.tax_amount,
          p.is_active, p.workspace_id, p.created_by, p.updated_by, p.created_at, p.updated_at
        FROM products p
        JOIN workspaces w ON w.id = $2
        WHERE p.code = $1
      "#,
      code,
      workspace_id
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(product)
  }

  async fn update_by_workspace(
    &self,
    id: Uuid,
    workspace_id: Uuid,
    product_data: UpdateProductRequest,
    updated_by: Uuid,
  ) -> AppResult<Option<Product>> {
    let product = sqlx::query_as!(
      Product,
      r#"
        UPDATE products 
        SET 
          code = COALESCE($1, code),
          name = COALESCE($2, name),
          category_id = COALESCE($3, category_id),
          base_unit = COALESCE($4, base_unit),
          unit_on_report_preview = COALESCE($5, unit_on_report_preview),
          selling_price = COALESCE($6, selling_price),
          unit_cost = COALESCE($7, unit_cost),
          supplier_id = COALESCE($8, supplier_id),
          track_inventory = COALESCE($9, track_inventory),
          description = COALESCE($10, description),
          sku = COALESCE($11, sku),
          barcode = COALESCE($12, barcode),
          minimum_stock = COALESCE($13, minimum_stock),
          maximum_stock = COALESCE($14, maximum_stock),
          reorder_level = COALESCE($15, reorder_level),
          current_stock = COALESCE($16, current_stock),
          tax_type = COALESCE($17, tax_type),
          tax_rate = COALESCE($18, tax_rate),
          tax_amount = COALESCE($19, tax_amount),
          is_active = COALESCE($20, is_active),
          updated_by = $21,
          updated_at = NOW()
        WHERE id = $22 
          AND EXISTS (
            SELECT 1 FROM workspaces w
            JOIN workspace_users wu ON w.id = wu.workspace_id
            WHERE w.id = $23 AND wu.user_id = $21
          )
        RETURNING 
          id, code, name, category_id, base_unit, unit_on_report_preview,
          selling_price, unit_cost, supplier_id, track_inventory,
          description, sku, barcode, minimum_stock, maximum_stock,
          reorder_level, current_stock, tax_type, tax_rate, tax_amount,
          is_active, workspace_id, created_by, updated_by, created_at, updated_at
      "#,
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
      updated_by,
      id,
      workspace_id
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(product)
  }

  async fn delete_by_workspace(&self, id: Uuid, workspace_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query!(
      r#"
        DELETE FROM products 
        WHERE id = $1 
          AND EXISTS (
            SELECT 1 FROM workspaces w
            WHERE w.id = $2
          )
      "#,
      id,
      workspace_id
    )
    .execute(&self.db)
    .await?;

    Ok(result.rows_affected() > 0)
  }

  // Product Categories methods
  async fn find_all_categories(&self) -> AppResult<Vec<ProductCategory>> {
    let categories = sqlx::query_as!(
      ProductCategory,
      r#"
        SELECT 
          id, code, name, description, parent_id, is_active,
          workspace_id, created_by, updated_by, created_at, updated_at
        FROM product_categories 
        ORDER BY created_at DESC
      "#
    )
    .fetch_all(&self.db)
    .await?;

    Ok(categories)
  }

  async fn find_category_by_id(&self, id: Uuid) -> AppResult<Option<ProductCategory>> {
    let category = sqlx::query_as!(
      ProductCategory,
      r#"
        SELECT 
          id, code, name, description, parent_id, is_active,
          workspace_id, created_by, updated_by, created_at, updated_at
        FROM product_categories 
        WHERE id = $1
      "#,
      id
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(category)
  }

  async fn find_category_by_code(&self, code: &str) -> AppResult<Option<ProductCategory>> {
    let category = sqlx::query_as!(
      ProductCategory,
      r#"
        SELECT 
          id, code, name, description, parent_id, is_active,
          workspace_id, created_by, updated_by, created_at, updated_at
        FROM product_categories 
        WHERE code = $1
      "#,
      code
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(category)
  }

  async fn find_active_categories(&self) -> AppResult<Vec<ProductCategory>> {
    let categories = sqlx::query_as!(
      ProductCategory,
      r#"
        SELECT 
          id, code, name, description, parent_id, is_active,
          workspace_id, created_by, updated_by, created_at, updated_at
        FROM product_categories 
        WHERE is_active = true
        ORDER BY name ASC
      "#
    )
    .fetch_all(&self.db)
    .await?;

    Ok(categories)
  }
}
