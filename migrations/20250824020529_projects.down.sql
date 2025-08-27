-- Down migration: projects

-- Drop trigger
DROP TRIGGER IF EXISTS update_projects_updated_at ON projects;

-- Drop indexes
DROP INDEX IF EXISTS idx_projects_end_date;
DROP INDEX IF EXISTS idx_projects_start_date;
DROP INDEX IF EXISTS idx_projects_status;
DROP INDEX IF EXISTS idx_projects_client_contact_id;
DROP INDEX IF EXISTS idx_projects_manager_id;
DROP INDEX IF EXISTS idx_projects_department_id;
DROP INDEX IF EXISTS idx_projects_user_id;
DROP INDEX IF EXISTS idx_projects_workspace_id;
DROP INDEX IF EXISTS idx_projects_name;
DROP INDEX IF EXISTS idx_projects_code;

-- Drop policy first
DROP POLICY IF EXISTS projects_policy ON projects;

-- Drop table
DROP TABLE IF EXISTS projects;

-- Drop enum
DROP TYPE IF EXISTS project_status;
