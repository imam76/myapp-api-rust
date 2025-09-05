-- Down migration: workspaces

-- Drop RLS policies first
DROP POLICY IF EXISTS workspace_users_modify_policy ON workspace_users;
DROP POLICY IF EXISTS workspace_users_select_policy ON workspace_users;
DROP POLICY IF EXISTS workspaces_delete_policy ON workspaces;
DROP POLICY IF EXISTS workspaces_update_policy ON workspaces;
DROP POLICY IF EXISTS workspaces_select_policy ON workspaces;

-- Disable RLS
ALTER TABLE workspace_users DISABLE ROW LEVEL SECURITY;
ALTER TABLE workspaces DISABLE ROW LEVEL SECURITY;

-- Drop triggers
DROP TRIGGER IF EXISTS trigger_add_creator_as_admin ON workspaces;
DROP TRIGGER IF EXISTS update_workspaces_users_updated_at ON workspace_users;
DROP TRIGGER IF EXISTS update_workspaces_updated_at ON workspaces;

-- Drop function
DROP FUNCTION IF EXISTS add_creator_to_workspace_members();

-- Drop indexes for workspace_users
DROP INDEX IF EXISTS idx_workspace_users_user_id;
DROP INDEX IF EXISTS idx_workspace_users_workspace_id;

-- Drop indexes for workspaces
DROP INDEX IF EXISTS idx_workspaces_owner_id;
DROP INDEX IF EXISTS idx_workspaces_name;
DROP INDEX IF EXISTS idx_workspaces_updated_at;
DROP INDEX IF EXISTS idx_workspaces_created_at;

-- Drop tables (workspace_users first due to foreign key dependency)
DROP TABLE IF EXISTS workspace_users;
DROP TABLE IF EXISTS workspaces;

-- Drop the enum type
DROP TYPE IF EXISTS workspace_role;
