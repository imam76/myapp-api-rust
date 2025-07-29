use axum::{
  Router,
  routing::{delete, get, post, put},
};
use std::sync::Arc;

use crate::state::AppState;

use super::workspace_handlers::{
  add_user_to_workspace, create_workspace, delete_workspace, get_user_workspaces, get_workspace, get_workspace_users, remove_user_from_workspace,
  update_user_role, update_workspace,
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
