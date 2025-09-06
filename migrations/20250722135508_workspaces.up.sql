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

-- Automatically make the workspace creator an 'admin' of that workspace.
CREATE OR REPLACE FUNCTION add_creator_to_workspace_members()
RETURNS TRIGGER AS $$
BEGIN
    -- The user who creates the workspace (NEW.created_by) is made an admin.
    INSERT INTO workspace_users (workspace_id, user_id, role)
    VALUES (NEW.id, NEW.created_by, 'admin');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Drop trigger if it exists to ensure it's idempotent
DROP TRIGGER IF EXISTS trigger_add_creator_as_admin ON workspaces;

CREATE TRIGGER trigger_add_creator_as_admin
AFTER INSERT ON workspaces
FOR EACH ROW
EXECUTE FUNCTION add_creator_to_workspace_members();


-- RLS POLICIES
-- First, drop the old generic policies
ALTER TABLE workspaces ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS workspaces_policy ON workspaces;

ALTER TABLE workspace_users ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS workspace_users_access_policy ON workspace_users;


-- === New, Granular Policies for 'workspaces' table ===

-- This allows authenticated users to create workspaces

CREATE POLICY workspaces_insert_policy ON workspaces
    FOR INSERT
    WITH CHECK (
        -- Allow any authenticated user to create a workspace
        -- The creator will be set as the owner and admin via the trigger
        current_setting('app.current_user_id', true)::UUID IS NOT NULL
        AND
        -- Ensure the owner_id matches the current user
        owner_id = current_setting('app.current_user_id', true)::UUID
    );

-- SELECT: Any user who is a member of the workspace can see it.
CREATE POLICY workspaces_select_policy ON workspaces
    FOR SELECT
    USING (id IN (
        SELECT workspace_id FROM workspace_users WHERE user_id = current_setting('app.current_user_id', true)::UUID
    ));

-- UPDATE: Only 'admin' members can update workspace details.
CREATE POLICY workspaces_update_policy ON workspaces
    FOR UPDATE
    USING ( has_workspace_access(id, ARRAY['admin']) );

-- DELETE: Only the original owner can delete the workspace (strongest protection).
CREATE POLICY workspaces_delete_policy ON workspaces
    FOR DELETE
    USING ( owner_id = current_setting('app.current_user_id', true)::UUID );


-- === New, Granular Policies for 'workspace_users' (managing members) ===

-- SELECT: An admin can see all members in their workspace, others can only see their own membership.
CREATE POLICY workspace_users_select_policy ON workspace_users
    FOR SELECT
    USING (
        -- An admin of the workspace can see all members
        has_workspace_access(workspace_id, ARRAY['admin'])
        OR
        -- A user can see their own membership record
        user_id = current_setting('app.current_user_id', true)::UUID
    );

-- INSERT/UPDATE/DELETE: Only admins can add, modify, or remove members.
CREATE POLICY workspace_users_modify_policy ON workspace_users
    FOR ALL -- Covers INSERT, UPDATE, DELETE for member management
    USING ( has_workspace_access(workspace_id, ARRAY['admin']) )
    WITH CHECK ( has_workspace_access(workspace_id, ARRAY['admin']) );