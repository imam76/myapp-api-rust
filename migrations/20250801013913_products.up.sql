-- Up migration: product_categories and products

-- Ensure tax_type enum exists (shared concept)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'tax_type') THEN
        CREATE TYPE tax_type AS ENUM ('percentage', 'fixed_amount');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS product_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES product_categories(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    workspace_id UUID REFERENCES workspaces(id),
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_product_categories_name ON product_categories(name);
CREATE INDEX IF NOT EXISTS idx_product_categories_parent_id ON product_categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_product_categories_is_active ON product_categories(is_active);

-- Trigger function to update the updated_at column
CREATE TRIGGER update_product_categories_updated_at
BEFORE UPDATE ON product_categories
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Enable Row Level Security
ALTER TABLE product_categories ENABLE ROW LEVEL SECURITY;

-- Define new, cleaner policies using the optimized helper function
CREATE POLICY product_categories_select_policy ON product_categories
    FOR SELECT
    USING ( has_workspace_access(workspace_id, ARRAY['admin', 'member', 'viewer']) );

CREATE POLICY product_categories_insert_policy ON product_categories
    FOR INSERT
    WITH CHECK ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) );

CREATE POLICY product_categories_update_policy ON product_categories
    FOR UPDATE
    USING ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) )
    WITH CHECK ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) );

CREATE POLICY product_categories_delete_policy ON product_categories
    FOR DELETE
    USING ( has_workspace_access(workspace_id, ARRAY['admin']) );


CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    category_id UUID REFERENCES product_categories(id),
    base_unit VARCHAR(50) NOT NULL,
    unit_on_report_preview VARCHAR(50),
    sku VARCHAR(100) UNIQUE,
    barcode VARCHAR(100) UNIQUE,
    description TEXT,
    supplier_id UUID REFERENCES contacts(id),
    track_inventory BOOLEAN NOT NULL DEFAULT false,
    minimum_stock INTEGER DEFAULT 0,
    maximum_stock INTEGER,
    reorder_level INTEGER DEFAULT 0,
    stock INTEGER DEFAULT 0,
    unit_cost NUMERIC(15,2) NOT NULL DEFAULT 0.00,
    selling_price NUMERIC(15,2) NOT NULL DEFAULT 0.00,
    tax_type tax_type DEFAULT 'percentage',
    tax_rate NUMERIC(5,2) DEFAULT 0.00,
    tax_amount NUMERIC(15,2) DEFAULT 0.00,
    is_active BOOLEAN NOT NULL DEFAULT true,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_products_name ON products(name);
CREATE INDEX IF NOT EXISTS idx_products_category_id ON products(category_id);
CREATE INDEX IF NOT EXISTS idx_products_supplier_id ON products(supplier_id);
CREATE INDEX IF NOT EXISTS idx_products_is_active ON products(is_active);
CREATE INDEX IF NOT EXISTS idx_products_created_at ON products(created_at);

-- Trigger function to update the updated_at column
CREATE TRIGGER update_products_updated_at
BEFORE UPDATE ON products
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Enable Row Level Security
ALTER TABLE products ENABLE ROW LEVEL SECURITY;

-- Define new, cleaner policies using the optimized helper function
CREATE POLICY products_select_policy ON products
    FOR SELECT
    USING ( has_workspace_access(workspace_id, ARRAY['admin', 'member', 'viewer']) );

CREATE POLICY products_insert_policy ON products
    FOR INSERT
    WITH CHECK ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) );

CREATE POLICY products_update_policy ON products
    FOR UPDATE
    USING ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) )
    WITH CHECK ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) );

CREATE POLICY products_delete_policy ON products
    FOR DELETE
    USING ( has_workspace_access(workspace_id, ARRAY['admin']) );