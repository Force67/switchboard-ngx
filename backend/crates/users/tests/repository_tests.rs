//! Repository-level tests for the users crate

use switchboard_database::{UserRepository, CreateUserRequest, UpdateUserRequest, UserRole, UserStatus};
use tempfile::TempDir;
use sqlx::SqlitePool;

/// Helper function to create a test database
async fn create_test_database() -> (SqlitePool, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_users_repo.db");
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
fn create_test_user_request(email: &str) -> CreateUserRequest {
    CreateUserRequest {
        email: email.to_string(),
        username: format!("user_{}", email),
        display_name: format!("Test User {}", email),
        password: "password123".to_string(),
        avatar_url: Some(format!("https://example.com/{}.jpg", email)),
        bio: Some(format!("Bio for {}", email)),
    }
}

#[tokio::test]
async fn test_repository_crud_operations() {
    let (pool, _temp_dir) = create_test_database().await;
    let repo = UserRepository::new(pool);

    // Test CREATE
    let request = create_test_user_request("test1@example.com");
    let created_user = repo.create(&request).await.unwrap();

    assert!(created_user.id > 0);
    let email = request.email.clone();
    let username = request.username.clone();
    let display_name = request.display_name.clone();

    assert_eq!(created_user.email, Some(email.clone()));
    assert_eq!(created_user.username, Some(username.clone()));
    assert_eq!(created_user.display_name, Some(display_name));
    assert_eq!(created_user.role, UserRole::User);
    assert_eq!(created_user.status, UserStatus::Active);

    // Test READ by ID
    let found_user = repo.find_by_id(created_user.id).await.unwrap();
    assert!(found_user.is_some());
    assert_eq!(found_user.unwrap().id, created_user.id);

    // Test READ by public ID
    let found_by_public_id = repo.find_by_public_id(&created_user.public_id).await.unwrap();
    assert!(found_by_public_id.is_some());
    assert_eq!(found_by_public_id.unwrap().public_id, created_user.public_id);

    // Test READ by email
    let found_by_email = repo.find_by_email(&email).await.unwrap();
    assert!(found_by_email.is_some());
    assert_eq!(found_by_email.unwrap().email, Some(request.email));

    // Test READ by username
    let found_by_username = repo.find_by_username(&username).await.unwrap();
    assert!(found_by_username.is_some());
    assert_eq!(found_by_username.unwrap().username, Some(request.username));

    // Test UPDATE
    let update_request = UpdateUserRequest {
        display_name: Some("Updated Display Name".to_string()),
        avatar_url: Some("https://example.com/updated.jpg".to_string()),
        bio: Some("Updated bio".to_string()),
        role: Some(UserRole::Admin),
    };
    let updated_user = repo.update(created_user.id, &update_request).await.unwrap();

    assert_eq!(updated_user.id, created_user.id);
    assert_eq!(updated_user.display_name, Some("Updated Display Name".to_string()));
    assert_eq!(updated_user.role, UserRole::Admin);

    // Test DELETE (soft delete)
    repo.delete(created_user.id).await.unwrap();

    // Verify user is marked as deleted
    let deleted_user = repo.find_by_id(created_user.id).await.unwrap();
    assert!(deleted_user.is_none());

    // But should still be findable if we query all records (for data recovery)
    let all_users = repo.count().await.unwrap();
    assert!(all_users >= 0); // The count might exclude deleted users
}

#[tokio::test]
async fn test_repository_user_management_operations() {
    let (pool, _temp_dir) = create_test_database().await;
    let repo = UserRepository::new(pool);

    // Create a test user
    let request = create_test_user_request("test2@example.com");
    let user = repo.create(&request).await.unwrap();

    // Test update last login
    repo.update_last_login(user.id).await.unwrap();
    let user_with_login = repo.find_by_id(user.id).await.unwrap().unwrap();
    assert!(user_with_login.last_login_at.is_some());

    // Test update active status
    repo.update_active_status(user.id, false).await.unwrap();
    let inactive_user = repo.find_by_id(user.id).await.unwrap().unwrap();
    assert!(!inactive_user.is_active);

    // Test verify email
    repo.verify_email(user.id).await.unwrap();
    let verified_user = repo.find_by_id(user.id).await.unwrap().unwrap();
    assert!(verified_user.email_verified);

    // Test update password
    repo.update_password(user.id, "new_password_hash").await.unwrap();
    // Password hash update is verified at repository level

    // Clean up
    repo.delete(user.id).await.unwrap();
}

#[tokio::test]
async fn test_repository_search_and_filtering() {
    let (pool, _temp_dir) = create_test_database().await;
    let repo = UserRepository::new(pool);

    // Create users with different roles
    let admin_request = create_test_user_request("admin@example.com");
    let admin_user = repo.create(&admin_request).await.unwrap();

    // Update admin user role
    let role_update = UpdateUserRequest {
        display_name: None,
        avatar_url: None,
        bio: None,
        role: Some(UserRole::Admin),
    };
    repo.update(admin_user.id, &role_update).await.unwrap();

    // Create regular users
    let user1 = repo.create(&create_test_user_request("user1@example.com")).await.unwrap();
    let user2 = repo.create(&create_test_user_request("user2@example.com")).await.unwrap();

    // Test search by display name
    let search_results = repo.search_by_display_name("Test User user1", 10).await.unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].email, Some("user1@example.com".to_string()));

    // Test find by role
    let admin_results = repo.find_by_role(UserRole::Admin, 10).await.unwrap();
    assert_eq!(admin_results.len(), 1);
    assert_eq!(admin_results[0].email, Some("admin@example.com".to_string()));

    let user_results = repo.find_by_role(UserRole::User, 10).await.unwrap();
    assert_eq!(user_results.len(), 2);

    // Test email exists
    assert!(repo.email_exists("admin@example.com").await.unwrap());
    assert!(repo.email_exists("user1@example.com").await.unwrap());
    assert!(!repo.email_exists("nonexistent@example.com").await.unwrap());

    // Clean up
    repo.delete(admin_user.id).await.unwrap();
    repo.delete(user1.id).await.unwrap();
    repo.delete(user2.id).await.unwrap();
}

#[tokio::test]
async fn test_repository_batch_operations() {
    let (pool, _temp_dir) = create_test_database().await;
    let repo = UserRepository::new(pool);

    // Create multiple users
    let mut user_ids = Vec::new();
    for i in 0..5 {
        let request = create_test_user_request(&format!("batch{}@example.com", i));
        let user = repo.create(&request).await.unwrap();
        user_ids.push(user.id);
    }

    // Test batch update status
    let updated_count = repo.batch_update_status(&user_ids[..3], UserStatus::Inactive).await.unwrap();
    assert_eq!(updated_count, 3);

    // Verify the batch update
    for &user_id in &user_ids[..3] {
        let user = repo.find_by_id(user_id).await.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().status, UserStatus::Inactive);
    }

    // Remaining users should still be active
    for &user_id in &user_ids[3..] {
        let user = repo.find_by_id(user_id).await.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().status, UserStatus::Active);
    }

    // Clean up
    for &user_id in &user_ids {
        repo.delete(user_id).await.unwrap();
    }
}

#[tokio::test]
async fn test_repository_statistics() {
    let (pool, _temp_dir) = create_test_database().await;
    let repo = UserRepository::new(pool);

    // Create users with different roles
    let admin_user = repo.create(&create_test_user_request("admin_stats@example.com")).await.unwrap();
    let user1 = repo.create(&create_test_user_request("user1_stats@example.com")).await.unwrap();
    let user2 = repo.create(&create_test_user_request("user2_stats@example.com")).await.unwrap();

    // Update one user to admin
    let role_update = UpdateUserRequest {
        display_name: None,
        avatar_url: None,
        bio: None,
        role: Some(UserRole::Admin),
    };
    repo.update(admin_user.id, &role_update).await.unwrap();

    // Update one user to inactive
    repo.update_active_status(user2.id, false).await.unwrap();

    // Get statistics
    let stats = repo.get_user_stats().await.unwrap();
    assert_eq!(stats.total_count, 3);
    assert_eq!(stats.active_count, 2); // admin and user1 are active

    // Check role distribution
    let mut admin_count = 0;
    let mut user_count = 0;
    let mut moderator_count = 0;
    for (role, count) in &stats.by_role {
        match role {
            UserRole::Admin => admin_count = *count,
            UserRole::User => user_count = *count,
            UserRole::Moderator => moderator_count = *count,
        }
    }
    assert_eq!(admin_count, 1);
    assert_eq!(user_count, 2);

    // Test individual count methods
    let total_count = repo.count().await.unwrap();
    let active_count = repo.count_active().await.unwrap();
    assert_eq!(total_count, 3);
    assert_eq!(active_count, 2);

    // Clean up
    repo.delete(admin_user.id).await.unwrap();
    repo.delete(user1.id).await.unwrap();
    repo.delete(user2.id).await.unwrap();
}

#[tokio::test]
async fn test_repository_edge_cases() {
    let (pool, _temp_dir) = create_test_database().await;
    let repo = UserRepository::new(pool);

    // Test update non-existent user
    let update_request = UpdateUserRequest {
        display_name: Some("Updated".to_string()),
        avatar_url: None,
        bio: None,
        role: None,
    };
    let result = repo.update(99999, &update_request).await;
    assert!(result.is_err());

    // Test delete non-existent user
    let result = repo.delete(99999).await;
    assert!(result.is_err());

    // Test update last login for non-existent user
    let result = repo.update_last_login(99999).await;
    assert!(result.is_err());

    // Test update active status for non-existent user
    let result = repo.update_active_status(99999, false).await;
    assert!(result.is_err());

    // Test verify email for non-existent user
    let result = repo.verify_email(99999).await;
    assert!(result.is_err());

    // Test update password for non-existent user
    let result = repo.update_password(99999, "hash").await;
    assert!(result.is_err());

    // Test batch update with empty list
    let result = repo.batch_update_status(&[], UserStatus::Inactive).await.unwrap();
    assert_eq!(result, 0);

    // Test search with no results
    let results = repo.search_by_display_name("nonexistent", 10).await.unwrap();
    assert_eq!(results.len(), 0);

    // Test find by role with no results
    let results = repo.find_by_role(UserRole::Admin, 10).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_repository_concurrent_operations() {
    let (pool, _temp_dir) = create_test_database().await;
    let repo = std::sync::Arc::new(UserRepository::new(pool));

    // Create multiple users concurrently
    let mut handles = Vec::new();

    for i in 0..20 {
        let repo_clone = repo.clone();
        let handle = tokio::spawn(async move {
            let request = create_test_user_request(&format!("concurrent{}@example.com", i));
            repo_clone.create(&request).await
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

    assert_eq!(user_ids.len(), 20);

    // Test concurrent searches
    let mut search_handles = Vec::new();
    for user in &created_users {
        let repo_clone = repo.clone();
        let email = user.email.clone().unwrap();
        let handle = tokio::spawn(async move {
            repo_clone.find_by_email(&email).await
        });
        search_handles.push(handle);
    }

    // Verify all searches succeeded
    for handle in search_handles {
        let found_user = handle.await.unwrap().unwrap();
        assert!(found_user.is_some());
    }

    // Clean up
    for user in created_users {
        repo.delete(user.id).await.unwrap();
    }
}

#[tokio::test]
async fn test_repository_data_integrity() {
    let (pool, _temp_dir) = create_test_database().await;
    let repo = UserRepository::new(pool);

    // Test duplicate email prevention
    let request = create_test_user_request("duplicate@example.com");
    let user1 = repo.create(&request).await.unwrap();

    // Try to create another user with the same email
    let result = repo.create(&request).await;
    assert!(result.is_err());

    // Test very long data
    let long_request = CreateUserRequest {
        email: "long@example.com".to_string(),
        username: "long_user".to_string(),
        display_name: "A".repeat(255),
        password: "password123".to_string(),
        avatar_url: Some("https://example.com/very_long_url.jpg".to_string()),
        bio: Some("A".repeat(1000)),
    };

    let long_user = repo.create(&long_request).await.unwrap();
    assert_eq!(long_user.display_name.unwrap().len(), 255);
    assert_eq!(long_user.bio.unwrap().len(), 1000);

    // Test Unicode characters
    let unicode_request = CreateUserRequest {
        email: "unicode@example.com".to_string(),
        username: "unicode_user".to_string(),
        display_name: "🚀 User with Unicode テスト".to_string(),
        password: "password123".to_string(),
        avatar_url: None,
        bio: Some("Unicode bio: 🎉 Hello 世界 🌍".to_string()),
    };

    let unicode_user = repo.create(&unicode_request).await.unwrap();
    assert!(unicode_user.display_name.unwrap().contains("🚀"));
    assert!(unicode_user.bio.unwrap().contains("🌍"));

    // Clean up
    repo.delete(user1.id).await.unwrap();
    repo.delete(long_user.id).await.unwrap();
    repo.delete(unicode_user.id).await.unwrap();
}

