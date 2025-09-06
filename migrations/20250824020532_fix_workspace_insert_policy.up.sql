-- Fix missing INSERT policy for workspaces table
-- This allows authenticated users to create workspaces

CREATE POLICY workspaces_insert_policy ON workspaces
    FOR INSERT
    WITH CHECK (
        -- Allow any authenticated user to create a workspace
        -- The creator will be set as the owner and admin via the trigger
        current_setting('app.current_user_id', true)::UUID IS NOT NULL
        AND
        -- Ensure the owner_id matches the current user
        owner_id = current_setting('app.current_user_id', true)::UUID
    );
