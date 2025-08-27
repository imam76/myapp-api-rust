-- Up migration: workspaces
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'workspace_role') THEN
        CREATE TYPE workspace_role AS ENUM ('admin', 'member', 'viewer');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS workspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for workspaces
CREATE INDEX IF NOT EXISTS idx_workspaces_created_at ON workspaces(created_at);
CREATE INDEX IF NOT EXISTS idx_workspaces_updated_at ON workspaces(updated_at);
CREATE INDEX IF NOT EXISTS idx_workspaces_name ON workspaces(name);
CREATE INDEX IF NOT EXISTS idx_workspaces_owner_id ON workspaces(owner_id);

-- Trigger function to update the updated_at column
CREATE TRIGGER update_workspaces_updated_at
BEFORE UPDATE ON workspaces
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Create workspace_users table
CREATE TABLE IF NOT EXISTS workspace_users (
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role workspace_role NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, user_id)
);

-- Indexes for workspace_users
CREATE INDEX IF NOT EXISTS idx_workspace_users_workspace_id ON workspace_users(workspace_id);
CREATE INDEX IF NOT EXISTS idx_workspace_users_user_id ON workspace_users(user_id);

-- Trigger for workspace_users
CREATE TRIGGER update_workspaces_users_updated_at
    BEFORE UPDATE ON workspace_users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- NOW ENABLE RLS after both tables exist
-- Enable Row Level Security for workspaces
ALTER TABLE workspaces ENABLE ROW LEVEL SECURITY;
CREATE POLICY workspaces_policy ON workspaces
    FOR ALL
    USING (id IN (
        SELECT workspace_id 
        FROM workspace_users 
        WHERE user_id = (SELECT current_setting('app.current_user_id', true)::UUID)
    ))
    WITH CHECK (id IN (
        SELECT workspace_id 
        FROM workspace_users 
        WHERE user_id = (SELECT current_setting('app.current_user_id', true)::UUID)
    ));

-- Policy for workspace_users
ALTER TABLE workspace_users ENABLE ROW LEVEL SECURITY;
CREATE POLICY workspace_users_access_policy ON workspace_users
    FOR ALL
    USING (
        -- User dapat melihat keanggotaan mereka sendiri
        user_id = (SELECT current_setting('app.current_user_id', true)::UUID)
        OR
        -- Admin workspace dapat melihat semua anggota
        workspace_id IN (
            SELECT workspace_id
            FROM workspace_users
            WHERE user_id = (SELECT current_setting('app.current_user_id', true)::UUID)
            AND role = 'admin'
        )
    )
    WITH CHECK (
        -- Hanya admin workspace yang dapat menambah/mengubah keanggotaan
        workspace_id IN (
            SELECT workspace_id
            FROM workspace_users
            WHERE user_id = (SELECT current_setting('app.current_user_id', true)::UUID)
            AND role = 'admin'
        )
    );