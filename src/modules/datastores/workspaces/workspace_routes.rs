use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;

use crate::state::AppState;

use super::workspace_handlers::{
    create_workspace, get_workspace, update_workspace, delete_workspace,
    get_user_workspaces, get_workspace_users, add_user_to_workspace,
    remove_user_from_workspace, update_user_role,
};

pub fn workspace_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Workspace CRUD
        .route("/workspaces", post(create_workspace))
        .route("/workspaces", get(get_user_workspaces))
        .route("/workspaces/:workspace_id", get(get_workspace))
        .route("/workspaces/:workspace_id", put(update_workspace))
        .route("/workspaces/:workspace_id", delete(delete_workspace))
        
        // Workspace user management
        .route("/workspaces/:workspace_id/users", get(get_workspace_users))
        .route("/workspaces/:workspace_id/users", post(add_user_to_workspace))
        .route("/workspaces/:workspace_id/users/:user_id", delete(remove_user_from_workspace))
        .route("/workspaces/:workspace_id/users/:user_id/role", put(update_user_role))
}
