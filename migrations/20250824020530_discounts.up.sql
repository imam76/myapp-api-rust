-- Create discount type enum
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'discount_type') THEN
        CREATE TYPE discount_type AS ENUM ('percentage', 'fixed_amount');
    END IF;
END $$;

-- Create discounts table
CREATE TABLE IF NOT EXISTS discounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    discount_type discount_type NOT NULL DEFAULT 'percentage',
    discount_value NUMERIC(15, 2) NOT NULL DEFAULT 0.00,
    
    -- Applicability
    is_active BOOLEAN NOT NULL DEFAULT true,
    start_date DATE,
    end_date DATE,
    minimum_amount NUMERIC(15, 2) DEFAULT 0.00,
    maximum_discount NUMERIC(15, 2),
    
    -- Usage limits
    usage_limit INTEGER,
    used_count INTEGER DEFAULT 0,
    
    -- Workspace support
    workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    
    -- Metadata
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(code, workspace_id),
    CHECK (discount_value >= 0),
    CHECK (discount_type = 'percentage' AND discount_value <= 100 OR discount_type = 'fixed_amount')
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_discounts_code ON discounts(code);
CREATE INDEX IF NOT EXISTS idx_discounts_name ON discounts(name);
CREATE INDEX IF NOT EXISTS idx_discounts_workspace_id ON discounts(workspace_id);
CREATE INDEX IF NOT EXISTS idx_discounts_user_id ON discounts(user_id);
CREATE INDEX IF NOT EXISTS idx_discounts_is_active ON discounts(is_active);
CREATE INDEX IF NOT EXISTS idx_discounts_start_date ON discounts(start_date);
CREATE INDEX IF NOT EXISTS idx_discounts_end_date ON discounts(end_date);

-- Create trigger for updating updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_discounts_updated_at BEFORE UPDATE ON discounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
