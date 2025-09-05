-- Helper function to get current role safely. Returns 'none' if not set.
-- STABLE indicates the function's result is consistent within a single transaction.
CREATE OR REPLACE FUNCTION get_current_role()
RETURNS TEXT
LANGUAGE plpgsql STABLE AS $$
BEGIN
    RETURN COALESCE(current_setting('app.current_user_role', true), 'none');
END;
$$;

-- The primary, optimized function for checking access.
-- It verifies both the workspace and the user's role against an allowed list.
CREATE OR REPLACE FUNCTION has_workspace_access(
    record_workspace_id UUID,
    allowed_roles TEXT[]
)
RETURNS BOOLEAN
LANGUAGE plpgsql STABLE AS $$
DECLARE
    session_workspace_id UUID;
BEGIN
    -- Get the workspace_id from the current session
    BEGIN
        session_workspace_id := current_setting('app.current_workspace_id', true)::UUID;
    EXCEPTION
        WHEN OTHERS THEN
            RETURN FALSE; -- If session variable is not set or invalid, deny access.
    END;

    -- Check if the record's workspace matches the session's and if the user's role is in the allowed list.
    RETURN record_workspace_id = session_workspace_id AND get_current_role() = ANY(allowed_roles);
END;
$$;