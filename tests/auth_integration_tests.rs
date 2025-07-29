use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use myapp_api_rust::{app, setup_state};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

/// Helper to setup test database connection
async fn setup_test_db() -> PgPool {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to test database");
    
    // Run migrations to ensure all tables exist
    // Note: This assumes sqlx-cli is installed and migrations are available
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

/// Helper to setup the Axum app with test state
async fn setup_app() -> axum::Router {
    let state = setup_state().await;
    app(state)
}

/// Helper to generate unique test identifiers
fn generate_test_id() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
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
    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await
        .ok();
    
    // Also cleanup by username pattern (for tests)
    sqlx::query("DELETE FROM contacts WHERE created_by IN (SELECT id FROM users WHERE username LIKE 'test%')")
        .execute(pool)
        .await
        .ok();
    
    sqlx::query("DELETE FROM users WHERE username LIKE 'test%'")
        .execute(pool)
        .await
        .ok();
}

/// Helper to clean up test contact
async fn cleanup_test_contact(pool: &PgPool, code: &str) {
    sqlx::query("DELETE FROM contacts WHERE code = $1")
        .bind(code)
        .execute(pool)
        .await
        .ok();
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

/// Helper to create a test contact for a user
async fn create_test_contact(app: &axum::Router, token: &str, code: &str) -> (StatusCode, Value) {
    let payload = json!({
        "code": code,
        "name": "Test Contact",
        "email": "test.contact@example.com",
        "position": "Manager",
        "contact_type": "customer",
        "address": "123 Test St"
    });

    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/contacts")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    (status, json)
}

#[tokio::test]
async fn test_auth_user_registration_success() {
    let pool = setup_test_db().await;
    let app = setup_app().await;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let test_email = format!("test_register_{}@example.com", timestamp);
    let test_username = format!("testuser_{}", timestamp);
    
    // Cleanup before test
    cleanup_test_user(&pool, &test_email).await;
    
    let (status, response) = register_test_user(&app, &test_username, &test_email, "password123").await;
    
    // Debug output
    if status != StatusCode::CREATED {
        println!("Registration response status: {}", status);
        println!("Registration response body: {}", serde_json::to_string_pretty(&response).unwrap());
    }
    
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["status"], "success");
    assert!(response["data"]["id"].is_string());
    assert_eq!(response["data"]["email"], test_email);
    assert_eq!(response["data"]["username"], test_username);
    assert_eq!(response["data"]["is_active"], true);
    
    // Cleanup after test
    cleanup_test_user(&pool, &test_email).await;
}

#[tokio::test]
async fn test_auth_user_registration_duplicate_email() {
    let pool = setup_test_db().await;
    let app = setup_app().await;
    
    let test_email = "test_duplicate@example.com";
    
    // Cleanup before test
    cleanup_test_user(&pool, test_email).await;
    
    // Register first user
    let (status1, _) = register_test_user(&app, "testuser1", test_email, "password123").await;
    assert_eq!(status1, StatusCode::CREATED);
    
    // Try to register second user with same email
    let (status2, response2) = register_test_user(&app, "testuser2", test_email, "password456").await;
    assert_eq!(status2, StatusCode::CONFLICT);
    assert!(response2["error"].as_str().unwrap().contains("CONFLICT"));
    
    // Cleanup after test
    cleanup_test_user(&pool, test_email).await;
}

#[tokio::test]
async fn test_auth_user_login_success() {
    let pool = setup_test_db().await;
    let app = setup_app().await;
    
    let test_email = "test_login@example.com";
    
    // Cleanup before test
    cleanup_test_user(&pool, test_email).await;
    
    // Register user first
    let (status, _) = register_test_user(&app, "testuser", test_email, "password123").await;
    assert_eq!(status, StatusCode::CREATED);
    
    // Test login
    let token = login_test_user(&app, test_email, "password123").await;
    assert!(!token.is_empty());
    assert!(token.starts_with("eyJ")); // JWT tokens start with eyJ
    
    // Cleanup after test
    cleanup_test_user(&pool, test_email).await;
}

#[tokio::test]
async fn test_auth_user_login_invalid_credentials() {
    let pool = setup_test_db().await;
    let app = setup_app().await;
    
    let test_id = generate_test_id();
    let test_email = &format!("test_invalid_login_{}@example.com", test_id);
    
    // Cleanup before test
    cleanup_test_user(&pool, test_email).await;
    
    // Register user first
    let (status, _) = register_test_user(&app, &format!("testuser_{}", test_id), test_email, "password123").await;
    assert_eq!(status, StatusCode::CREATED);
    
    // Test login with wrong password
    let payload = json!({
        "email": test_email,
        "password": "wrongpassword"
    });

    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/auth/login")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    // Cleanup after test
    cleanup_test_user(&pool, test_email).await;
}

#[tokio::test]
async fn test_auth_protected_endpoint_without_token() {
    let app = setup_app().await;
    
    let request = Request::builder()
        .method(http::Method::GET)
        .uri("/api/v1/auth/me")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "TOKEN_MISSING");
}

#[tokio::test]
async fn test_auth_protected_endpoint_with_invalid_token() {
    let app = setup_app().await;
    
    let request = Request::builder()
        .method(http::Method::GET)
        .uri("/api/v1/auth/me")
        .header(http::header::AUTHORIZATION, "Bearer invalid-token")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "TOKEN_INVALID");
}

#[tokio::test]
async fn test_auth_me_endpoint_success() {
    let pool = setup_test_db().await;
    let app = setup_app().await;
    
    let test_id = generate_test_id();
    let test_email = &format!("test_me_{}@example.com", test_id);
    let test_username = &format!("testuser_{}", test_id);
    
    // Cleanup before test
    cleanup_test_user(&pool, test_email).await;
    
    // Register and login user
    let (status, _) = register_test_user(&app, test_username, test_email, "password123").await;
    assert_eq!(status, StatusCode::CREATED);
    
    let token = login_test_user(&app, test_email, "password123").await;
    
    // Test /me endpoint
    let request = Request::builder()
        .method(http::Method::GET)
        .uri("/api/v1/auth/me")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["email"], *test_email);
    assert_eq!(json["data"]["username"], *test_username);
    
    // Cleanup after test
    cleanup_test_user(&pool, test_email).await;
}

#[tokio::test]
async fn test_user_data_isolation() {
    let pool = setup_test_db().await;
    let app = setup_app().await;
    
    let test_id1 = generate_test_id();
    let test_id2 = generate_test_id();
    let user1_email = &format!("test_user1_{}@example.com", test_id1);
    let user2_email = &format!("test_user2_{}@example.com", test_id2);
    let contact_code = &format!("ISO_{}", &test_id1[0..8]); // Use only first 8 chars
    
    // Cleanup before test
    cleanup_test_user(&pool, user1_email).await;
    cleanup_test_user(&pool, user2_email).await;
    cleanup_test_contact(&pool, contact_code).await;
    
    // Register two users
    let (status1, _) = register_test_user(&app, &format!("user1_{}", test_id1), user1_email, "password123").await;
    assert_eq!(status1, StatusCode::CREATED);
    
    let (status2, _) = register_test_user(&app, &format!("user2_{}", test_id2), user2_email, "password123").await;
    assert_eq!(status2, StatusCode::CREATED);
    
    // Login both users
    let token1 = login_test_user(&app, user1_email, "password123").await;
    let token2 = login_test_user(&app, user2_email, "password123").await;
    
    // User 1 creates a contact
    let (create_status, create_response) = create_test_contact(&app, &token1, contact_code).await;
    assert_eq!(create_status, StatusCode::CREATED);
    let contact_id = create_response["data"]["id"].as_str().unwrap();
    
    // User 1 can see their contacts
    let request = Request::builder()
        .method(http::Method::GET)
        .uri("/api/v1/contacts")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token1))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["pagination"]["total"], 1);
    
    // User 2 cannot see User 1's contacts
    let request = Request::builder()
        .method(http::Method::GET)
        .uri("/api/v1/contacts")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token2))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["pagination"]["total"], 0);
    
    // User 2 cannot access User 1's contact by ID
    let request = Request::builder()
        .method(http::Method::GET)
        .uri(&format!("/api/v1/contacts/{}", contact_id))
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token2))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    // Cleanup after test
    cleanup_test_contact(&pool, contact_code).await;
    cleanup_test_user(&pool, user1_email).await;
    cleanup_test_user(&pool, user2_email).await;
}

#[tokio::test]
async fn test_contact_audit_trail() {
    let pool = setup_test_db().await;
    let app = setup_app().await;
    
    let test_id = generate_test_id();
    let test_email = &format!("test_audit_{}@example.com", test_id);
    let contact_code = &format!("AUDIT_{}", &test_id[0..8]); // Use only first 8 chars of timestamp
    
    // Cleanup before test
    cleanup_test_user(&pool, test_email).await;
    cleanup_test_contact(&pool, contact_code).await;
    
    // Register and login user
    let (status, register_response) = register_test_user(&app, &format!("testuser_{}", test_id), test_email, "password123").await;
    assert_eq!(status, StatusCode::CREATED);
    let user_id = register_response["data"]["id"].as_str().unwrap();
    
    let token = login_test_user(&app, test_email, "password123").await;
    
    // Create contact
    let (create_status, create_response) = create_test_contact(&app, &token, contact_code).await;
    assert_eq!(create_status, StatusCode::CREATED);
    
    let contact_data = &create_response["data"];
    assert_eq!(contact_data["created_by"], user_id);
    assert!(contact_data["updated_by"].is_null());
    
    let contact_id = contact_data["id"].as_str().unwrap();
    
    // Update contact
    let update_payload = json!({
        "name": "Updated Contact Name",
        "position": "Senior Manager"
    });

    let request = Request::builder()
        .method(http::Method::PUT)
        .uri(&format!("/api/v1/contacts/{}", contact_id))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&update_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    let updated_data = &json["data"];
    assert_eq!(updated_data["name"], "Updated Contact Name");
    assert_eq!(updated_data["position"], "Senior Manager");
    assert_eq!(updated_data["created_by"], user_id);
    assert_eq!(updated_data["updated_by"], user_id); // Should be set after update
    
    // Cleanup after test
    cleanup_test_contact(&pool, contact_code).await;
    cleanup_test_user(&pool, test_email).await;
}

#[tokio::test]
async fn test_contact_access_control() {
    let pool = setup_test_db().await;
    let app = setup_app().await;
    
    let test_id1 = generate_test_id();
    let test_id2 = generate_test_id();
    let user1_email = &format!("test_access1_{}@example.com", test_id1);
    let user2_email = &format!("test_access2_{}@example.com", test_id2);
    let contact_code = &format!("ACC_{}", &test_id1[0..8]); // Use only first 8 chars
    
    // Cleanup before test
    cleanup_test_user(&pool, user1_email).await;
    cleanup_test_user(&pool, user2_email).await;
    cleanup_test_contact(&pool, contact_code).await;
    
    // Register two users
    let (status1, _) = register_test_user(&app, &format!("user1_{}", test_id1), user1_email, "password123").await;
    assert_eq!(status1, StatusCode::CREATED);
    
    let (status2, _) = register_test_user(&app, &format!("user2_{}", test_id2), user2_email, "password123").await;
    assert_eq!(status2, StatusCode::CREATED);
    
    // Login both users
    let token1 = login_test_user(&app, user1_email, "password123").await;
    let token2 = login_test_user(&app, user2_email, "password123").await;
    
    // User 1 creates a contact
    let (create_status, create_response) = create_test_contact(&app, &token1, contact_code).await;
    assert_eq!(create_status, StatusCode::CREATED);
    let contact_id = create_response["data"]["id"].as_str().unwrap();
    
    // User 2 tries to update User 1's contact (should fail)
    let update_payload = json!({"name": "Unauthorized Update"});

    let request = Request::builder()
        .method(http::Method::PUT)
        .uri(&format!("/api/v1/contacts/{}", contact_id))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token2))
        .body(Body::from(serde_json::to_string(&update_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    // User 2 tries to delete User 1's contact (should fail)
    let request = Request::builder()
        .method(http::Method::DELETE)
        .uri(&format!("/api/v1/contacts/{}", contact_id))
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token2))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    // User 1 can still access their contact
    let request = Request::builder()
        .method(http::Method::GET)
        .uri(&format!("/api/v1/contacts/{}", contact_id))
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token1))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Cleanup after test
    cleanup_test_contact(&pool, contact_code).await;
    cleanup_test_user(&pool, user1_email).await;
    cleanup_test_user(&pool, user2_email).await;
}
