-- Down migration: workspaces

-- Drop trigger
DROP TRIGGER IF EXISTS update_workspaces_updated_at ON workspaces;
DROP TRIGGER IF EXISTS update_workspaces_users_updated_at ON workspace_users;

-- Drop indexes
DROP INDEX IF EXISTS idx_workspace_users_user_id;
DROP INDEX IF EXISTS idx_workspace_users_workspace_id;
DROP INDEX IF EXISTS idx_workspaces_owner_id;
DROP INDEX IF EXISTS idx_workspaces_name;
DROP INDEX IF EXISTS idx_workspaces_created_at;
DROP INDEX IF EXISTS idx_workspaces_updated_at;

-- Drop RLS policies
DROP POLICY IF EXISTS workspaces_policy ON workspaces;
DROP POLICY IF EXISTS workspace_users_access_policy ON workspace_users;

-- Drop workspace tables
DROP TABLE IF EXISTS workspace_users;
DROP TABLE IF EXISTS workspaces;

-- Drop workspace role enum
DROP TYPE IF EXISTS workspace_role;