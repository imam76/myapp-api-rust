-- Up migration: discounts
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'discount_type') THEN
        CREATE TYPE discount_type AS ENUM ('percentage', 'fixed_amount');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS discounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    discount_type discount_type NOT NULL DEFAULT 'percentage',
    discount_value NUMERIC(15,2) NOT NULL DEFAULT 0.00,
    is_active BOOLEAN NOT NULL DEFAULT true,
    start_date DATE,
    end_date DATE,
    minimum_amount NUMERIC(15,2) DEFAULT 0.00,
    maximum_discount NUMERIC(15,2),
    usage_limit INTEGER,
    used_count INTEGER DEFAULT 0,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (discount_value >= 0),
    CHECK (discount_type = 'percentage' AND discount_value <= 100 OR discount_type = 'fixed_amount')
);

CREATE INDEX IF NOT EXISTS idx_discounts_name ON discounts(name);
CREATE INDEX IF NOT EXISTS idx_discounts_workspace_id ON discounts(workspace_id);
CREATE INDEX IF NOT EXISTS idx_discounts_is_active ON discounts(is_active);
CREATE INDEX IF NOT EXISTS idx_discounts_start_date ON discounts(start_date);
CREATE INDEX IF NOT EXISTS idx_discounts_end_date ON discounts(end_date);

CREATE TRIGGER update_discounts_updated_at
    BEFORE UPDATE ON discounts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- enable Row Level Security
ALTER TABLE discounts ENABLE ROW LEVEL SECURITY;

-- Define new, cleaner policies using the optimized helper function
CREATE POLICY discounts_select_policy ON discounts
    FOR SELECT
    USING ( has_workspace_access(workspace_id, ARRAY['admin', 'member', 'viewer']) );

CREATE POLICY discounts_insert_policy ON discounts
    FOR INSERT
    WITH CHECK ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) );

CREATE POLICY discounts_update_policy ON discounts
    FOR UPDATE
    USING ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) )
    WITH CHECK ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) );

CREATE POLICY discounts_delete_policy ON discounts
    FOR DELETE
    USING ( has_workspace_access(workspace_id, ARRAY['admin']) );

