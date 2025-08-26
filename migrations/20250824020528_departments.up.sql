-- Up migration: departments
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'department_status') THEN
        CREATE TYPE department_status AS ENUM ('active', 'inactive', 'archived');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    parent_department_id UUID REFERENCES departments(id) ON DELETE SET NULL,
    manager_id UUID REFERENCES users(id) ON DELETE SET NULL,
    budget NUMERIC(15,2) DEFAULT 0.00,
    status department_status NOT NULL DEFAULT 'active',
    workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE,
    created_by UUID REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_departments_name ON departments(name);
CREATE INDEX IF NOT EXISTS idx_departments_workspace_id ON departments(workspace_id);
CREATE INDEX IF NOT EXISTS idx_departments_parent_department_id ON departments(parent_department_id);
CREATE INDEX IF NOT EXISTS idx_departments_manager_id ON departments(manager_id);
CREATE INDEX IF NOT EXISTS idx_departments_status ON departments(status);

CREATE TRIGGER update_departments_updated_at
    BEFORE UPDATE ON departments
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- enable Row Level Security
ALTER TABLE departments ENABLE ROW LEVEL SECURITY;
CREATE POLICY departments_policy ON departments
    FOR ALL
    USING (workspace_id = current_setting('app.current_workspace_id', true)::UUID)
    WITH CHECK (workspace_id = current_setting('app.current_workspace_id', true)::UUID);
