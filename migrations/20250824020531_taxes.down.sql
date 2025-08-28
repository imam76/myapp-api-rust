-- Down migration: taxes

-- Drop trigger
DROP TRIGGER IF EXISTS update_taxes_updated_at ON taxes;

-- Drop indexes
DROP INDEX IF EXISTS idx_taxes_is_active;
DROP INDEX IF EXISTS idx_taxes_user_id;
DROP INDEX IF EXISTS idx_taxes_workspace_id;
DROP INDEX IF EXISTS idx_taxes_name;
DROP INDEX IF EXISTS idx_taxes_code;

-- Drop policy first
DROP POLICY IF EXISTS taxes_policy ON taxes;

-- Drop table first before dropping the enum
DROP TABLE IF EXISTS taxes;
