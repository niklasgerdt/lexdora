use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use DORAEU::api::build_router;
use sqlx::PgPool;
use serde_json::{json, Value};
use uuid::Uuid;

async fn get_test_pool() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");
    PgPool::connect(&db_url).await.expect("Failed to connect to database")
}

#[tokio::test]
async fn test_healthz() {
    let pool = get_test_pool().await;
    let app = build_router(pool);

    let response = app
        .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body, json!({"status": "ok"}));
}

#[tokio::test]
async fn test_org_crud() {
    let pool = get_test_pool().await;
    let app = build_router(pool);

    // 1. Create Org
    let org_name = format!("Test Org {}", Uuid::new_v4());
    let create_req = json!({
        "name": org_name,
        "legal_entity_id": "LEI123"
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/orgs")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&create_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let res_data: Value = serde_json::from_slice(&body).unwrap();
    let org_id = res_data["id"].as_str().unwrap();

    // 2. Get Org
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/{}", org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let org_data: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(org_data["name"], org_name);

    // 3. List Orgs
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri("/orgs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let orgs_list: Value = serde_json::from_slice(&body).unwrap();
    assert!(orgs_list.as_array().unwrap().len() > 0);

    // 4. Update Org
    let new_name = format!("Updated {}", org_name);
    let update_req = json!({
        "name": new_name
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/orgs/{}", org_id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&update_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify update
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/{}", org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let org_data: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(org_data["name"], new_name);

    // 5. Delete Org
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/orgs/{}", org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify deletion
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/orgs/{}", org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
#[tokio::test]
async fn test_role_creation() {
    let pool = get_test_pool().await;
    let app = build_router(pool);
    // 1. Create Org first
    let org_name = format!("Role Test Org {}", Uuid::new_v4());
    let create_org_req = json!({
        "name": org_name,
    });
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/orgs")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&create_org_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let res_data: Value = serde_json::from_slice(&body).unwrap();
    let org_id = res_data["id"].as_str().unwrap();
    // 2. Create Role (The part that supposedly fails with 422)
    let role_name = format!("Role {}", Uuid::new_v4());
    let create_role_req = json!({
        "organization_id": org_id,
        "name": role_name,
        "description": "Full access"
    });
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/roles")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&create_role_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    // If it's 422, this assertion will fail
    assert_eq!(response.status(), StatusCode::CREATED, "Role creation failed with status: {}", response.status());

    // 3. Attempt to create Role WITHOUT organization_id (Reproduction of the 422 error)
    let bad_role_req = json!({
        "name": "Bad Role"
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/roles")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&bad_role_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY, "Expected 422 for missing organization_id, got: {}", response.status());
}

#[tokio::test]
async fn test_user_crud() {
    let pool = get_test_pool().await;
    let app = build_router(pool);

    // 1. Create Org for the User
    let org_name = format!("Org for User {}", Uuid::new_v4());
    let create_org_req = json!({"name": org_name});
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/orgs")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&create_org_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let res_org: Value = serde_json::from_slice(&body).unwrap();
    let org_id = res_org["id"].as_str().unwrap();

    // 2. Create User
    let email = format!("user-{}@example.com", Uuid::new_v4());
    let create_user_req = json!({
        "organization_id": org_id,
        "email": email,
        "full_name": "Test User"
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&create_user_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let res_user: Value = serde_json::from_slice(&body).unwrap();
    let user_id = res_user["id"].as_str().unwrap();

    // 3. Get User
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/users/{}", user_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let user_data: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(user_data["email"], email);

    // 4. Update User
    let new_email = format!("updated-{}@example.com", Uuid::new_v4());
    let update_user_req = json!({
        "email": new_email,
        "is_active": false
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/users/{}", user_id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&update_user_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 5. List Users
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri("/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 6. Delete User
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/users/{}", user_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
