/// Macro to generate next_code handler for any module
/// This reduces boilerplate code while maintaining flexibility per module
#[macro_export]
macro_rules! impl_next_code_handler {
  (
        $handler_name:ident,
        $module_name:literal,
        $config:expr
    ) => {
    /// Get the next available code for this module based on name
    #[axum::debug_handler]
    pub async fn $handler_name(
      State(state): State<Arc<AppState>>,
      current_user: CurrentUser,
      WorkspaceContext(workspace_id): WorkspaceContext,
      Query(params): Query<NextCodeQuery>,
    ) -> AppResult<Json<ApiResponse<String>>> {
      use crate::utils::code_generator::CodeGenerator;

      tracing::debug!(
        "Getting next available {} code for name: '{}' in workspace: {}",
        $module_name,
        params.name,
        workspace_id
      );

      // Validate workspace access
      let workspace_repository = &state.workspace_repository;
      if !check_workspace_permission(workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
        return Err(AppError::Authorization(format!(
          "You don't have permission to access {} in this workspace",
          $module_name
        )));
      }

      // Generate next code using the shared utility
      // Access the database pool directly from AppState
      let code_generator = CodeGenerator::new(state.db.clone());
      let next_code = code_generator.get_next_available_code(&$config, &params.name, Some(workspace_id)).await?;

      tracing::debug!("Next available {} code: {} for name: '{}'", $module_name, next_code, params.name);

      let response = ApiResponse::success(next_code, &format!("Next {} code retrieved successfully", $module_name));
      Ok(Json(response))
    }
  };
}

/// Query parameters for next code request
#[derive(Debug, serde::Deserialize)]
pub struct NextCodeQuery {
  pub name: String,
}
