-- Down migration: contacts

-- Drop trigger
DROP TRIGGER IF EXISTS update_contacts_updated_at ON contacts;

-- Drop indexes
DROP INDEX IF EXISTS idx_contacts_code;
DROP INDEX IF EXISTS idx_contacts_email;
DROP INDEX IF EXISTS idx_contacts_type;
DROP INDEX IF EXISTS idx_contacts_is_active;
DROP INDEX IF EXISTS idx_contacts_created_at;
DROP INDEX IF EXISTS idx_contacts_workspace_id;

-- Remove workspace_id column if present
ALTER TABLE contacts DROP COLUMN IF EXISTS workspace_id;

-- Drop table
DROP TABLE IF EXISTS contacts;
