-- Down migration: workspaces

-- Drop trigger
DROP TRIGGER IF EXISTS update_workspaces_updated_at ON workspaces;

-- Drop indexes
DROP INDEX IF EXISTS idx_contacts_workspace_id;
DROP INDEX IF EXISTS idx_workspace_users_user_id;
DROP INDEX IF EXISTS idx_workspace_users_workspace_id;
DROP INDEX IF EXISTS idx_workspaces_owner_id;

-- Remove workspace_id column from contacts/products/product_categories
ALTER TABLE contacts DROP COLUMN IF EXISTS workspace_id;
ALTER TABLE products DROP COLUMN IF EXISTS workspace_id;
ALTER TABLE product_categories DROP COLUMN IF EXISTS workspace_id;

-- Drop workspace tables
DROP TABLE IF EXISTS workspace_users;
DROP TABLE IF EXISTS workspaces;

-- Drop workspace role enum
DROP TYPE IF EXISTS workspace_role;
