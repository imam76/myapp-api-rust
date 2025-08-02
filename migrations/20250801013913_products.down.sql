-- Add down migration script here

-- Drop foreign key constraint first
ALTER TABLE products DROP CONSTRAINT IF EXISTS fk_products_category;

-- Drop triggers
DROP TRIGGER IF EXISTS update_products_updated_at ON products;

-- Drop the trigger function (only if no other tables are using it)
-- DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop indexes for products table
DROP INDEX IF EXISTS idx_products_created_at;
DROP INDEX IF EXISTS idx_products_is_active;
DROP INDEX IF EXISTS idx_products_supplier_id;
DROP INDEX IF EXISTS idx_products_category_id;
DROP INDEX IF EXISTS idx_products_barcode;
DROP INDEX IF EXISTS idx_products_sku;
DROP INDEX IF EXISTS idx_products_name;
DROP INDEX IF EXISTS idx_products_code;

-- Drop indexes for product_categories table
DROP INDEX IF EXISTS idx_product_categories_is_active;
DROP INDEX IF EXISTS idx_product_categories_parent_id;
DROP INDEX IF EXISTS idx_product_categories_name;
DROP INDEX IF EXISTS idx_product_categories_code;

-- Drop tables (products first due to foreign key dependency)
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS product_categories;
