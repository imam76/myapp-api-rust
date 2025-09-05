-- Up migration: projects
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'project_status') THEN
        CREATE TYPE project_status AS ENUM ('planning', 'active', 'on_hold', 'completed', 'cancelled', 'archived', 'inactive');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    department_id UUID REFERENCES departments(id) ON DELETE SET NULL,
    manager_id UUID REFERENCES users(id) ON DELETE SET NULL,
    client_contact_id UUID REFERENCES contacts(id) ON DELETE SET NULL,
    start_date DATE,
    end_date DATE,
    estimated_hours NUMERIC(10,2) DEFAULT 0.00,
    actual_hours NUMERIC(10,2) DEFAULT 0.00,
    budget NUMERIC(15,2) DEFAULT 0.00,
    actual_cost NUMERIC(15,2) DEFAULT 0.00,
    status project_status NOT NULL DEFAULT 'planning',
    priority VARCHAR(20) DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name);
CREATE INDEX IF NOT EXISTS idx_projects_workspace_id ON projects(workspace_id);
CREATE INDEX IF NOT EXISTS idx_projects_department_id ON projects(department_id);
CREATE INDEX IF NOT EXISTS idx_projects_manager_id ON projects(manager_id);
CREATE INDEX IF NOT EXISTS idx_projects_client_contact_id ON projects(client_contact_id);
CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status);
CREATE INDEX IF NOT EXISTS idx_projects_start_date ON projects(start_date);
CREATE INDEX IF NOT EXISTS idx_projects_end_date ON projects(end_date);

CREATE TRIGGER update_projects_updated_at
    BEFORE UPDATE ON projects
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- enable Row Level Security
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;

-- Define new, cleaner policies using the optimized helper function
CREATE POLICY projects_select_policy ON projects
    FOR SELECT
    USING ( has_workspace_access(workspace_id, ARRAY['admin', 'member', 'viewer']) );

CREATE POLICY projects_insert_policy ON projects
    FOR INSERT
    WITH CHECK ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) );

CREATE POLICY projects_update_policy ON projects
    FOR UPDATE
    USING ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) )
    WITH CHECK ( has_workspace_access(workspace_id, ARRAY['admin', 'member']) );

CREATE POLICY projects_delete_policy ON projects
    FOR DELETE
    USING ( has_workspace_access(workspace_id, ARRAY['admin']) );

