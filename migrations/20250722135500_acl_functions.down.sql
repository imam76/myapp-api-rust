-- This file should undo the changes made in the corresponding .up.sql file.
DROP FUNCTION IF EXISTS has_workspace_access(UUID, TEXT[]);
DROP FUNCTION IF EXISTS get_current_role();