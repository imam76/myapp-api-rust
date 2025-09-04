-- Down migration: products and product_categories

-- Dropping the tables will also remove their indexes, constraints, triggers, and RLS policies.
-- 'products' has a foreign key to 'product_categories', so 'products' must be dropped first.
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS product_categories;
