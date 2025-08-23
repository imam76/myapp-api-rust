-- Create tax type enum (if not exists from products table)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'tax_type') THEN
        CREATE TYPE tax_type AS ENUM ('percentage', 'fixed_amount');
    END IF;
END $$;

-- Create taxes table
CREATE TABLE IF NOT EXISTS taxes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    tax_type tax_type NOT NULL DEFAULT 'percentage',
    tax_rate NUMERIC(5, 2) NOT NULL DEFAULT 0.00,
    tax_amount NUMERIC(15, 2) NOT NULL DEFAULT 0.00,
    
    -- Tax details
    is_compound BOOLEAN NOT NULL DEFAULT false,
    is_inclusive BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    
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
    CHECK (tax_rate >= 0),
    CHECK (tax_amount >= 0)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_taxes_code ON taxes(code);
CREATE INDEX IF NOT EXISTS idx_taxes_name ON taxes(name);
CREATE INDEX IF NOT EXISTS idx_taxes_workspace_id ON taxes(workspace_id);
CREATE INDEX IF NOT EXISTS idx_taxes_user_id ON taxes(user_id);
CREATE INDEX IF NOT EXISTS idx_taxes_is_active ON taxes(is_active);

-- Create trigger for updating updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_taxes_updated_at BEFORE UPDATE ON taxes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
