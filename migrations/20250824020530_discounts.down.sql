-- Drop trigger
DROP TRIGGER IF EXISTS update_discounts_updated_at ON discounts;

-- Drop indexes
DROP INDEX IF EXISTS idx_discounts_end_date;
DROP INDEX IF EXISTS idx_discounts_start_date;
DROP INDEX IF EXISTS idx_discounts_is_active;
DROP INDEX IF EXISTS idx_discounts_user_id;
DROP INDEX IF EXISTS idx_discounts_workspace_id;
DROP INDEX IF EXISTS idx_discounts_name;
DROP INDEX IF EXISTS idx_discounts_code;

-- Drop table
DROP TABLE IF EXISTS discounts;

-- Drop enum
DROP TYPE IF EXISTS discount_type;
