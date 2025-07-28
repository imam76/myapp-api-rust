-- Drop trigger first
DROP TRIGGER IF EXISTS update_contacts_updated_at ON contacts;

-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop indexes
DROP INDEX IF EXISTS idx_contacts_code;
DROP INDEX IF EXISTS idx_contacts_email;
DROP INDEX IF EXISTS idx_contacts_type;
DROP INDEX IF EXISTS idx_contacts_is_active;
DROP INDEX IF EXISTS idx_contacts_created_at;

-- Drop contacts table
DROP TABLE IF EXISTS contacts;
