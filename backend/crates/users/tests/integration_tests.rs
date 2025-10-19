//! Integration tests for the users crate with real database

use switchboard_users::{UserService, CreateUserRequest, UpdateUserRequest, UserRole, UserStatus};
use tempfile::TempDir;
use sqlx::SqlitePool;

/// Database configuration for testing
#[derive(Clone)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
}

/// Helper function to create a test database
async fn create_test_database() -> (SqlitePool, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_users.db");
    let db_url = format!("sqlite:{}", db_path.display());

    // Create a simple SQLite pool for testing
    let pool = SqlitePool::connect(&db_url).await.expect("Failed to create test database");

    // Create basic users table schema
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_id TEXT NOT NULL UNIQUE,
            email TEXT UNIQUE,
            username TEXT UNIQUE,
            display_name TEXT,
            avatar_url TEXT,
            bio TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            role TEXT NOT NULL DEFAULT 'user',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_login_at TEXT,
            email_verified BOOLEAN NOT NULL DEFAULT false,
            is_active BOOLEAN NOT NULL DEFAULT true,
            password_hash TEXT
        )
        "#
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    (pool, temp_dir)
}

/// Helper function to create a valid user request
fn create_test_user_request() -> CreateUserRequest {
    CreateUserRequest {
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        display_name: "Test User".to_string(),
        password: "password123".to_string(),
        avatar_url: Some("https://example.com/avatar.jpg".to_string()),
        bio: Some("Test bio".to_string()),
    }
}

#[tokio::test]
async fn test_user_crud_operations_integration() {
    let (pool, _temp_dir) = create_test_database().await;
    let service = UserService::new(pool);

    // Test CREATE
    let create_request = create_test_user_request();
    let email = create_request.email.clone();
    let created_user = service.create_user(create_request).await.unwrap();

    assert!(created_user.id > 0);
    assert_eq!(created_user.email, Some(email.clone()));
    assert_eq!(created_user.username, Some("testuser".to_string()));
    assert_eq!(created_user.display_name, Some("Test User".to_string()));
    assert_eq!(created_user.role, UserRole::User);
    assert_eq!(created_user.status, UserStatus::Active);
    assert!(created_user.is_active);
    assert!(!created_user.email_verified);

    // Test READ by ID
    let found_user = service.get_user(created_user.id).await.unwrap();
    assert_eq!(found_user.id, created_user.id);
    assert_eq!(found_user.email, created_user.email);

    // Test READ by public ID
    let found_by_public_id = service.get_user_by_public_id(&created_user.public_id).await.unwrap();
    assert_eq!(found_by_public_id.id, created_user.id);
    assert_eq!(found_by_public_id.public_id, created_user.public_id);

    // Test READ by email
    let found_by_email = service.get_user_by_email(&created_user.email.clone().unwrap()).await.unwrap();
    assert!(found_by_email.is_some());
    assert_eq!(found_by_email.unwrap().id, created_user.id);

    // Test UPDATE
    let update_request = UpdateUserRequest {
        display_name: Some("Updated Test User".to_string()),
        avatar_url: Some("https://example.com/new_avatar.jpg".to_string()),
        bio: Some("Updated bio".to_string()),
        role: Some(UserRole::Admin),
    };
    let updated_user = service.update_user(created_user.id, update_request).await.unwrap();

    assert_eq!(updated_user.id, created_user.id);
    assert_eq!(updated_user.display_name, Some("Updated Test User".to_string()));
    assert_eq!(updated_user.avatar_url, Some("https://example.com/new_avatar.jpg".to_string()));
    assert_eq!(updated_user.bio, Some("Updated bio".to_string()));
    assert_eq!(updated_user.role, UserRole::Admin);

    // Test UPDATE LAST LOGIN
    service.update_last_login(created_user.id).await.unwrap();
    let user_with_login = service.get_user(created_user.id).await.unwrap();
    assert!(user_with_login.last_login_at.is_some());

    // Test SEARCH
    let search_results = service.search_users("Updated", 10).await.unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].id, created_user.id);

    // Test EMAIL AVAILABILITY
    assert!(!service.is_email_available(&email).await.unwrap());
    assert!(service.is_email_available("newemail@example.com").await.unwrap());

    // Test DELETE
    service.delete_user(created_user.id).await.unwrap();

    // Verify deletion
    let delete_result = service.get_user(created_user.id).await;
    assert!(delete_result.is_err());

    let email_result = service.get_user_by_email(&email).await.unwrap();
    assert!(email_result.is_none());
}

#[tokio::test]
async fn test_multiple_user_operations_integration() {
    let (pool, _temp_dir) = create_test_database().await;
    let service = UserService::new(pool);

    // Create multiple users
    let users_data = vec![
        ("alice@example.com", "alice", "Alice Smith"),
        ("bob@example.com", "bob", "Bob Johnson"),
        ("charlie@example.com", "charlie", "Charlie Brown"),
    ];

    let mut created_user_ids = Vec::new();

    for (email, username, display_name) in users_data {
        let request = CreateUserRequest {
            email: email.to_string(),
            username: username.to_string(),
            display_name: display_name.to_string(),
            password: "password123".to_string(),
            avatar_url: None,
            bio: None,
        };

        let user = service.create_user(request).await.unwrap();
        created_user_ids.push(user.id);
    }

    // Test that all users were created
    assert_eq!(created_user_ids.len(), 3);

    // Test search with partial match
    let search_results = service.search_users("Alice", 10).await.unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].display_name, Some("Alice Smith".to_string()));

    // Test search with multiple results
    let search_results = service.search_users("a", 10).await.unwrap();
    assert_eq!(search_results.len(), 2); // Alice and Charlie

    // Test pagination limit
    let search_results = service.search_users("", 100).await.unwrap();
    assert_eq!(search_results.len(), 3);

    // Delete users in reverse order
    for &user_id in created_user_ids.iter().rev() {
        service.delete_user(user_id).await.unwrap();
    }

    // Verify all users are deleted
    for &user_id in &created_user_ids {
        let result = service.get_user(user_id).await;
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_user_validation_integration() {
    let (pool, _temp_dir) = create_test_database().await;
    let service = UserService::new(pool);

    // Test invalid email
    let invalid_email_request = CreateUserRequest {
        email: "invalid-email".to_string(),
        username: "testuser".to_string(),
        display_name: "Test User".to_string(),
        password: "password123".to_string(),
        avatar_url: None,
        bio: None,
    };

    let result = service.create_user(invalid_email_request).await;
    assert!(result.is_err());

    // Test empty email
    let empty_email_request = CreateUserRequest {
        email: "".to_string(),
        username: "testuser".to_string(),
        display_name: "Test User".to_string(),
        password: "password123".to_string(),
        avatar_url: None,
        bio: None,
    };

    let result = service.create_user(empty_email_request).await;
    assert!(result.is_err());

    // Test duplicate email
    let request = create_test_user_request();
    let user1 = service.create_user(request.clone()).await.unwrap();

    let result = service.create_user(request).await;
    assert!(result.is_err());

    // Clean up
    service.delete_user(user1.id).await.unwrap();
}

#[tokio::test]
async fn test_user_edge_cases_integration() {
    let (pool, _temp_dir) = create_test_database().await;
    let service = UserService::new(pool);

    // Test user with very long valid data
    let long_request = CreateUserRequest {
        email: "verylongemail@example.com".to_string(),
        username: "verylongusername".to_string(),
        display_name: "A".repeat(255), // Maximum length
        password: "password123".to_string(),
        avatar_url: Some("https://example.com/very_long_url.jpg".to_string()),
        bio: Some("A".repeat(1000)), // Long bio
    };

    let user = service.create_user(long_request).await.unwrap();
    let original_display_name = user.display_name.clone();
    assert_eq!(user.display_name.clone().unwrap().len(), 255);
    assert_eq!(user.bio.clone().unwrap().len(), 1000);

    // Test update with no changes
    let empty_update = UpdateUserRequest {
        display_name: None,
        avatar_url: None,
        bio: None,
        role: None,
    };
    let unchanged_user = service.update_user(user.id, empty_update).await.unwrap();
    assert_eq!(unchanged_user.display_name, original_display_name);

    // Clean up
    service.delete_user(user.id).await.unwrap();
}

#[tokio::test]
async fn test_user_concurrent_operations_integration() {
    let (pool, _temp_dir) = create_test_database().await;
    let service = std::sync::Arc::new(UserService::new(pool));

    // Create multiple users concurrently
    let mut handles = Vec::new();

    for i in 0..10 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let request = CreateUserRequest {
                email: format!("user{}@example.com", i),
                username: format!("user{}", i),
                display_name: format!("User {}", i),
                password: "password123".to_string(),
                avatar_url: None,
                bio: None,
            };

            service_clone.create_user(request).await
        });
        handles.push(handle);
    }

    // Wait for all creations to complete
    let mut created_users = Vec::new();
    for handle in handles {
        let user = handle.await.unwrap().unwrap();
        created_users.push(user);
    }

    // Verify all users were created with unique IDs
    let mut user_ids = std::collections::HashSet::new();
    for user in &created_users {
        assert!(!user_ids.contains(&user.id), "Duplicate user ID found: {}", user.id);
        user_ids.insert(user.id);
    }

    assert_eq!(user_ids.len(), 10);

    // Clean up
    for user in created_users {
        service.delete_user(user.id).await.unwrap();
    }
}

#[tokio::test]
async fn test_user_search_pagination_integration() {
    let (pool, _temp_dir) = create_test_database().await;
    let service = UserService::new(pool);

    // Create users with similar names
    for i in 0..20 {
        let request = CreateUserRequest {
            email: format!("user{}@example.com", i),
            username: format!("user{}", i),
            display_name: format!("Test User {}", i),
            password: "password123".to_string(),
            avatar_url: None,
            bio: None,
        };

        service.create_user(request).await.unwrap();
    }

    // Test search with limit
    let results = service.search_users("Test User", 5).await.unwrap();
    assert_eq!(results.len(), 5);

    // Test search with higher limit
    let results = service.search_users("Test User", 25).await.unwrap();
    assert_eq!(results.len(), 20);

    // Test search with limit of 100 (maximum allowed)
    let results = service.search_users("Test", 100).await.unwrap();
    assert_eq!(results.len(), 20);
}


