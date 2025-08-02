use crate::modules::auth::auth_repository::AuthRepository;
use crate::modules::datastores::contacts::contact_repository::ContactRepository;
use crate::modules::datastores::products::product_repository::ProductRepository;
use crate::modules::datastores::workspaces::workspace_repository::WorkspaceRepository;
use sqlx::PgPool;
use std::sync::Arc;

/// The shared application state.
///
/// This struct holds all the shared resources that are available to every request handler.
/// It is initialized once at application startup and then cloned for each incoming request.
///
/// # Fields
///
/// * `db`: A `PgPool` for asynchronous connections to the PostgreSQL database.
/// * `contact_repository`: An `Arc` wrapped trait object for the contact repository.
///   This allows for dependency injection and easy mocking in tests. `Send` and `Sync` are
///   required to share the repository safely across threads.
/// * `auth_repository`: An `Arc` wrapped trait object for the auth repository.
/// * `jwt_secret`: The secret key used for signing JWTs.
#[derive(Clone)]
pub struct AppState {
  pub db: PgPool,
  pub contact_repository: Arc<dyn ContactRepository + Send + Sync>,
  pub product_repository: Arc<dyn ProductRepository + Send + Sync>,
  pub auth_repository: Arc<dyn AuthRepository + Send + Sync>,
  pub workspace_repository: Arc<dyn WorkspaceRepository + Send + Sync>,
  pub jwt_secret: String,
}
