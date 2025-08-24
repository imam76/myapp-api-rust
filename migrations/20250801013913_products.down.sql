-- Down migration: products and product_categories

-- Remove foreign key constraint from products
ALTER TABLE products DROP CONSTRAINT IF EXISTS fk_products_category;

-- Drop triggers
DROP TRIGGER IF EXISTS update_products_updated_at ON products;
DROP TRIGGER IF EXISTS update_product_categories_updated_at ON product_categories;

-- Drop product indexes
DROP INDEX IF EXISTS idx_products_created_at;
DROP INDEX IF EXISTS idx_products_is_active;
DROP INDEX IF EXISTS idx_products_supplier_id;
DROP INDEX IF EXISTS idx_products_category_id;
DROP INDEX IF EXISTS idx_products_barcode;
DROP INDEX IF EXISTS idx_products_sku;
DROP INDEX IF EXISTS idx_products_name;
DROP INDEX IF EXISTS idx_products_code;

-- Drop product_categories indexes
DROP INDEX IF EXISTS idx_product_categories_is_active;
DROP INDEX IF EXISTS idx_product_categories_parent_id;
DROP INDEX IF EXISTS idx_product_categories_name;
DROP INDEX IF EXISTS idx_product_categories_code;

-- Remove workspace_id columns if present
ALTER TABLE products DROP COLUMN IF EXISTS workspace_id;
ALTER TABLE product_categories DROP COLUMN IF EXISTS workspace_id;

-- Drop tables
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS product_categories;
