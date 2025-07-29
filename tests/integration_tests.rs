use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use myapp_api_rust::{app, state::AppState};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_test_db() -> PgPool {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to test database");
    pool
}

async fn clean_up_contact(pool: &PgPool, code: &str) {
    sqlx::query("DELETE FROM contacts WHERE code = $1")
        .bind(code)
        .execute(pool)
        .await
        .expect("Failed to clean up contact");
}

fn setup_app(pool: PgPool) -> axum::Router {
    let state = AppState {
        db: pool.clone(),
        contact_repository: Arc::new(
            myapp_api_rust::modules::datastores::contacts::contact_repository::SqlxContactRepository::new(pool),
        ),
    };
    app(state)
}

#[tokio::test]
async fn test_create_contact_success() {
    let pool = setup_test_db().await;
    let app = setup_app(pool.clone());

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
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["status"], "success");
    assert_eq!(body["data"]["code"], test_code);

    // Clean up the created contact
    clean_up_contact(&pool, &test_code).await;
}

#[tokio::test]
async fn test_create_contact_validation_error() {
    let pool = setup_test_db().await;
    let app = setup_app(pool);

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
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["error"], "VALIDATION_FAILED");
    assert!(body["details"]["errors"]["name"].is_array());
    assert!(body["details"]["errors"]["email"].is_array());
}

#[tokio::test]
async fn test_create_contact_duplicate_code() {
    let pool = setup_test_db().await;
    let app = setup_app(pool.clone());

    let test_code = "TEST_DUPLICATE";
    // Ensure the contact doesn't exist before the test
    clean_up_contact(&pool, test_code).await;

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
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK, "First request should succeed");

    // 2. Second request with the same code should fail
    let request2 = Request::builder()
        .method(http::Method::POST)
        .uri("/api/v1/contacts")
        .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response2 = app.oneshot(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::UNPROCESSABLE_ENTITY, "Second request should fail with duplicate code");

    let body = response2.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(body["error"], "VALIDATION_FAILED");
    assert_eq!(body["details"]["message"], "Contact code already exists");
    assert_eq!(body["details"]["code"], "DUPLICATE_CODE");

    // Clean up the created contact
    clean_up_contact(&pool, test_code).await;
}

