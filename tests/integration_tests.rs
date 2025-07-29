use axum::{
  body::Body,
  http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use myapp_api_rust::{app, setup_state};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

async fn setup_test_db() -> PgPool {
  dotenvy::dotenv().ok();
  let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
  let pool = PgPool::connect(&db_url).await.expect("Failed to connect to test database");

  // Run migrations to ensure all tables exist
  let output = std::process::Command::new("sqlx")
    .args(&["migrate", "run", "--database-url", &db_url])
    .output();

  match output {
    Ok(output) if output.status.success() => {
      eprintln!("Migrations applied successfully");
    }
    Ok(output) => {
      eprintln!("Migration failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Err(e) => {
      eprintln!("Failed to run migrations: {}. Continuing anyway...", e);
    }
  }

  pool
}

/// Helper to clean up test user data
async fn cleanup_test_user(pool: &PgPool, email: &str) {
  // Clean up contacts first (foreign key constraint)
  sqlx::query("DELETE FROM contacts WHERE created_by IN (SELECT id FROM users WHERE email = $1)")
    .bind(email)
    .execute(pool)
    .await
    .ok();

  // Clean up user
  sqlx::query("DELETE FROM users WHERE email = $1").bind(email).execute(pool).await.ok();
}

async fn clean_up_contact(pool: &PgPool, code: &str) {
  sqlx::query("DELETE FROM contacts WHERE code = $1")
    .bind(code)
    .execute(pool)
    .await
    .expect("Failed to clean up contact");
}

/// Helper to setup the Axum app with test state
async fn setup_app() -> axum::Router {
  let state = setup_state().await;
  app(state)
}

/// Helper to register a test user and return the response
async fn register_test_user(app: &axum::Router, username: &str, email: &str, password: &str) -> (StatusCode, Value) {
  let payload = json!({
      "username": username,
      "email": email,
      "password": password
  });

  let request = Request::builder()
    .method(http::Method::POST)
    .uri("/api/v1/auth/register")
    .header(http::header::CONTENT_TYPE, "application/json")
    .body(Body::from(serde_json::to_string(&payload).unwrap()))
    .unwrap();

  let response = app.clone().oneshot(request).await.unwrap();
  let status = response.status();
  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: Value = serde_json::from_slice(&body).unwrap();

  (status, json)
}

/// Helper to login a test user and return the JWT token
async fn login_test_user(app: &axum::Router, email: &str, password: &str) -> String {
  let payload = json!({
      "email": email,
      "password": password
  });

  let request = Request::builder()
    .method(http::Method::POST)
    .uri("/api/v1/auth/login")
    .header(http::header::CONTENT_TYPE, "application/json")
    .body(Body::from(serde_json::to_string(&payload).unwrap()))
    .unwrap();

  let response = app.clone().oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: Value = serde_json::from_slice(&body).unwrap();

  json["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_create_contact_success() {
  let pool = setup_test_db().await;
  let app = setup_app().await;

  let test_email = "test_contact_creation@example.com";
  let test_username = "testuser_contact";

  // Cleanup before test
  cleanup_test_user(&pool, test_email).await;

  // Register and login user to get auth token
  let (status, _) = register_test_user(&app, test_username, test_email, "password123").await;
  assert_eq!(status, StatusCode::CREATED);

  let token = login_test_user(&app, test_email, "password123").await;

  let test_code = "TEST_SUCCESS";
  let payload = json!({
      "code": test_code,
      "name": "Test User",
      "email": "test@example.com",
      "position": "Tester",
      "contact_type": "employee"
  });

  let request = Request::builder()
    .method(http::Method::POST)
    .uri("/api/v1/contacts")
    .header(http::header::CONTENT_TYPE, "application/json")
    .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::CREATED);

  let body = response.into_body().collect().await.unwrap().to_bytes();
  let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

  assert_eq!(body["status"], "success");
  assert_eq!(body["data"]["code"], test_code);

  // Clean up the created contact and user
  clean_up_contact(&pool, &test_code).await;
  cleanup_test_user(&pool, test_email).await;
}

#[tokio::test]
async fn test_create_contact_validation_error() {
  let pool = setup_test_db().await;
  let app = setup_app().await;

  let test_email = "test_validation@example.com";
  let test_username = "testuser_validation";

  // Cleanup before test
  cleanup_test_user(&pool, test_email).await;

  // Register and login user to get auth token
  let (status, _) = register_test_user(&app, test_username, test_email, "password123").await;
  assert_eq!(status, StatusCode::CREATED);

  let token = login_test_user(&app, test_email, "password123").await;

  // Payload with missing name and invalid email
  let payload = json!({
      "code": "TEST_VALIDATION",
      "name": "",
      "email": "not-an-email",
      "position": "Tester",
      "contact_type": "Test"
  });

  let request = Request::builder()
    .method(http::Method::POST)
    .uri("/api/v1/contacts")
    .header(http::header::CONTENT_TYPE, "application/json")
    .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

  let body = response.into_body().collect().await.unwrap().to_bytes();
  let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

  assert_eq!(body["error"], "VALIDATION_FAILED");
  // Check the actual response structure
  assert!(body["details"]["name"].is_array());
  assert!(body["details"]["email"].is_array());

  // Cleanup after test
  cleanup_test_user(&pool, test_email).await;
}

#[tokio::test]
async fn test_create_contact_duplicate_code() {
  let pool = setup_test_db().await;
  let app = setup_app().await;

  let test_email = "test_duplicate@example.com";
  let test_username = "testuser_duplicate";
  let test_code = "TEST_DUPLICATE";

  // Cleanup before test
  cleanup_test_user(&pool, test_email).await;
  clean_up_contact(&pool, test_code).await;

  // Register and login user to get auth token
  let (status, _) = register_test_user(&app, test_username, test_email, "password123").await;
  assert_eq!(status, StatusCode::CREATED);

  let token = login_test_user(&app, test_email, "password123").await;

  let payload = json!({
      "code": test_code,
      "name": "Original User",
      "email": "original@example.com",
      "position": "Original",
      "contact_type": "employee"
  });

  // 1. First request should succeed
  let request1 = Request::builder()
    .method(http::Method::POST)
    .uri("/api/v1/contacts")
    .header(http::header::CONTENT_TYPE, "application/json")
    .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
    .unwrap();

  let response1 = app.clone().oneshot(request1).await.unwrap();
  assert_eq!(response1.status(), StatusCode::CREATED, "First request should succeed");

  // 2. Second request with the same code should fail
  let request2 = Request::builder()
    .method(http::Method::POST)
    .uri("/api/v1/contacts")
    .header(http::header::CONTENT_TYPE, "application/json")
    .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
    .unwrap();

  let response2 = app.oneshot(request2).await.unwrap();
  assert_eq!(
    response2.status(),
    StatusCode::UNPROCESSABLE_ENTITY,
    "Second request should fail with duplicate code"
  );

  let body = response2.into_body().collect().await.unwrap().to_bytes();
  let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

  assert_eq!(body["error"], "VALIDATION_FAILED");
  // Check the actual response structure for duplicate code
  assert_eq!(body["details"]["code"][0]["message"], "Contact code already exists");
  assert_eq!(body["details"]["code"][0]["code"], "DUPLICATE_CODE");

  // Clean up the created contact and user
  clean_up_contact(&pool, test_code).await;
  cleanup_test_user(&pool, test_email).await;
}
