-- Down migration: departments

-- Drop trigger
DROP TRIGGER IF EXISTS update_departments_updated_at ON departments;

-- Drop indexes
DROP INDEX IF EXISTS idx_departments_status;
DROP INDEX IF EXISTS idx_departments_manager_id;
DROP INDEX IF EXISTS idx_departments_parent_department_id;
DROP INDEX IF EXISTS idx_departments_user_id;
DROP INDEX IF EXISTS idx_departments_workspace_id;
DROP INDEX IF EXISTS idx_departments_name;
DROP INDEX IF EXISTS idx_departments_code;

-- Drop policy first
DROP POLICY IF EXISTS departments_policy ON departments;

-- Drop table first before dropping the enum
DROP TABLE IF EXISTS departments;

-- Drop enum after table is dropped
DROP TYPE IF EXISTS department_status;
