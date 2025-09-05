-- Up migration: contacts
CREATE TABLE IF NOT EXISTS contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    position VARCHAR(100),
    type VARCHAR(50) NOT NULL CHECK (type IN ('salesman', 'employee', 'supplier', 'customer')),
    address TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_contacts_code ON contacts(code);
CREATE INDEX IF NOT EXISTS idx_contacts_email ON contacts(email);
CREATE INDEX IF NOT EXISTS idx_contacts_type ON contacts(type);
CREATE INDEX IF NOT EXISTS idx_contacts_is_active ON contacts(is_active);
CREATE INDEX IF NOT EXISTS idx_contacts_created_at ON contacts(created_at);
CREATE INDEX IF NOT EXISTS idx_contacts_workspace_id ON contacts(workspace_id);

-- Trigger function to update the updated_at column
CREATE TRIGGER update_contacts_updated_at
BEFORE UPDATE ON contacts
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Enable Row Level Security
ALTER TABLE contacts ENABLE ROW LEVEL SECURITY;

-- Define new, cleaner policies using the optimized helper function
CREATE POLICY contacts_select_policy ON contacts
    FOR SELECT
    USING ( has_workspace_access(workspace_id, ARRAY['admin', 'member', 'viewer']) );

CREATE POLICY contacts_insert_policy ON contacts
    FOR INSERT
    WITH CHECK ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) );

CREATE POLICY contacts_update_policy ON contacts
    FOR UPDATE
    USING ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) )
    WITH CHECK ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) );

CREATE POLICY contacts_delete_policy ON contacts
    FOR DELETE
    USING ( has_workspace_access(workspace_id, ARRAY['admin']) );