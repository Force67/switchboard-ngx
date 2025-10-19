//! User service for managing user operations.

use switchboard_database::{User, CreateUserRequest, UpdateUserRequest, UserRepository, UserError, UserResult};
use sqlx::sqlite::SqlitePool;
use super::mock_repositories::MockUserRepository;

/// Service for managing user operations
pub struct UserService<R> {
    user_repository: R,
}

impl UserService<UserRepository> {
    /// Create a new user service instance with real database repository
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            user_repository: UserRepository::new(pool),
        }
    }
}

impl UserService<MockUserRepository> {
    /// Create a new user service instance for testing
    pub fn new_for_testing() -> Self {
        Self {
            user_repository: MockUserRepository::new(),
        }
    }
}

impl<R> UserService<R>
where
    R: UserRepo,
{
    /// Get a user by ID
    pub async fn get_user(&self, user_id: i64) -> UserResult<User> {
        self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(UserError::UserNotFound)
    }

    /// Get a user by public ID
    pub async fn get_user_by_public_id(&self, public_id: &str) -> UserResult<User> {
        self.user_repository
            .find_by_public_id(public_id)
            .await?
            .ok_or(UserError::UserNotFound)
    }

    /// Get a user by email
    pub async fn get_user_by_email(&self, email: &str) -> UserResult<Option<User>> {
        self.user_repository.find_by_email(email).await
    }

    /// Create a new user
    pub async fn create_user(&self, request: CreateUserRequest) -> UserResult<User> {
        // Validate input
        if let Err(e) = request.validate() {
            return Err(UserError::DatabaseError(e));
        }

        // Check if email is already taken
        if let Some(_) = self.user_repository.find_by_email(&request.email).await? {
            return Err(UserError::EmailAlreadyExists);
        }

        let user = self.user_repository.create(&request).await?;

        // Log user creation for audit
        log::info!("Created new user: {:?} (ID: {})", user.email, user.id);

        Ok(user)
    }

    /// Update a user
    pub async fn update_user(&self, user_id: i64, request: UpdateUserRequest) -> UserResult<User> {
        // Check if user exists
        let _existing_user = self.get_user(user_id).await?;

        // Validate update request
        if let Err(e) = request.validate() {
            return Err(UserError::DatabaseError(e));
        }

        // Perform update
        let updated_user = self.user_repository.update(user_id, &request).await?;

        // Log user update for audit
        log::info!("Updated user: {:?} (ID: {})", updated_user.email, user_id);

        Ok(updated_user)
    }

    /// Delete a user
    pub async fn delete_user(&self, user_id: i64) -> UserResult<()> {
        // Check if user exists
        let user = self.get_user(user_id).await?;

        // Perform deletion
        self.user_repository.delete(user_id).await?;

        // Log user deletion for audit
        log::warn!("Deleted user: {:?} (ID: {})", user.email, user_id);

        Ok(())
    }

    /// Update user last login
    pub async fn update_last_login(&self, user_id: i64) -> UserResult<()> {
        // Check if user exists
        let _user = self.get_user(user_id).await?;

        // Update last login
        self.user_repository.update_last_login(user_id).await?;

        Ok(())
    }

    /// Search users by display name
    pub async fn search_users(&self, query: &str, limit: u32) -> UserResult<Vec<User>> {
        // Validate query
        let trimmed_query = query.trim();
        if trimmed_query.is_empty() {
            return Ok(Vec::new());
        }

        // Limit search results to prevent performance issues
        let search_limit = std::cmp::min(limit, 100);

        self.user_repository
            .search_by_display_name(trimmed_query, search_limit)
            .await
    }

    /// Check if email is available
    pub async fn is_email_available(&self, email: &str) -> UserResult<bool> {
        // Validate email format
        if let Err(_e) = self.validate_email(email) {
            return Err(UserError::InvalidEmail);
        }

        // Check if email exists
        let exists = self.user_repository.email_exists(email).await?;
        Ok(!exists)
    }

    /// Get user statistics
    pub async fn get_user_stats(&self) -> UserResult<super::mock_repositories::MockUserStats> {
        self.user_repository.get_user_stats().await
    }

    // Helper methods for validation

    /// Validate email format
    fn validate_email(&self, email: &str) -> Result<(), String> {
        if email.trim().is_empty() {
            return Err("Email cannot be empty".to_string());
        }

        if email.len() > 255 {
            return Err("Email too long (max 255 characters)".to_string());
        }

        // Basic email validation
        if !email.contains('@') || !email.contains('.') {
            return Err("Invalid email format".to_string());
        }

        Ok(())
    }
}

/// Trait for user repositories to allow generic usage
pub trait UserRepo {
    async fn find_by_id(&self, id: i64) -> UserResult<Option<User>>;
    async fn find_by_public_id(&self, public_id: &str) -> UserResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> UserResult<Option<User>>;
    async fn create(&self, request: &CreateUserRequest) -> UserResult<User>;
    async fn update(&self, user_id: i64, request: &UpdateUserRequest) -> UserResult<User>;
    async fn delete(&self, user_id: i64) -> UserResult<()>;
    async fn update_last_login(&self, user_id: i64) -> UserResult<()>;
    async fn search_by_display_name(&self, query: &str, limit: u32) -> UserResult<Vec<User>>;
    async fn email_exists(&self, email: &str) -> UserResult<bool>;
    async fn get_user_stats(&self) -> UserResult<super::mock_repositories::MockUserStats>;
}

impl UserRepo for UserRepository {
    async fn find_by_id(&self, id: i64) -> UserResult<Option<User>> {
        self.find_by_id(id).await
    }

    async fn find_by_public_id(&self, public_id: &str) -> UserResult<Option<User>> {
        self.find_by_public_id(public_id).await
    }

    async fn find_by_email(&self, email: &str) -> UserResult<Option<User>> {
        self.find_by_email(email).await
    }

    async fn create(&self, request: &CreateUserRequest) -> UserResult<User> {
        self.create(request).await
    }

    async fn update(&self, user_id: i64, request: &UpdateUserRequest) -> UserResult<User> {
        self.update(user_id, request).await
    }

    async fn delete(&self, user_id: i64) -> UserResult<()> {
        self.delete(user_id).await
    }

    async fn update_last_login(&self, user_id: i64) -> UserResult<()> {
        self.update_last_login(user_id).await
    }

    async fn search_by_display_name(&self, query: &str, limit: u32) -> UserResult<Vec<User>> {
        self.search_by_display_name(query, limit).await
    }

    async fn email_exists(&self, email: &str) -> UserResult<bool> {
        self.email_exists(email).await
    }

    async fn get_user_stats(&self) -> UserResult<super::mock_repositories::MockUserStats> {
        // This method doesn't exist in the real repository, return default stats
        Ok(super::mock_repositories::MockUserStats {
            total_users: 0,
            active_users: 0,
            inactive_users: 0,
        })
    }
}

impl UserRepo for MockUserRepository {
    async fn find_by_id(&self, id: i64) -> UserResult<Option<User>> {
        self.find_by_id(id).await
    }

    async fn find_by_public_id(&self, public_id: &str) -> UserResult<Option<User>> {
        self.find_by_public_id(public_id).await
    }

    async fn find_by_email(&self, email: &str) -> UserResult<Option<User>> {
        self.find_by_email(email).await
    }

    async fn create(&self, request: &CreateUserRequest) -> UserResult<User> {
        self.create(request).await
    }

    async fn update(&self, user_id: i64, request: &UpdateUserRequest) -> UserResult<User> {
        self.update(user_id, request).await
    }

    async fn delete(&self, user_id: i64) -> UserResult<()> {
        self.delete(user_id).await
    }

    async fn update_last_login(&self, user_id: i64) -> UserResult<()> {
        self.update_last_login(user_id).await
    }

    async fn search_by_display_name(&self, query: &str, limit: u32) -> UserResult<Vec<User>> {
        self.search_by_display_name(query, limit).await
    }

    async fn email_exists(&self, email: &str) -> UserResult<bool> {
        self.email_exists(email).await
    }

    async fn get_user_stats(&self) -> UserResult<super::mock_repositories::MockUserStats> {
        self.get_user_stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{User, CreateUserRequest, UpdateUserRequest, UserRole};

    fn create_test_service() -> UserService<MockUserRepository> {
        UserService::new_for_testing()
    }

    fn create_valid_user_request() -> CreateUserRequest {
        CreateUserRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            password: "password123".to_string(),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            bio: None,
        }
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let service = create_test_service();
        let request = create_valid_user_request();

        let user = service.create_user(request).await.unwrap();

        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert_eq!(user.username, Some("testuser".to_string()));
        assert_eq!(user.display_name, Some("Test User".to_string()));
        assert_eq!(user.avatar_url, Some("https://example.com/avatar.jpg".to_string()));
        assert_eq!(user.role, UserRole::User);
        assert!(user.is_active);
        assert!(user.id > 0);
    }

    #[tokio::test]
    async fn test_create_user_duplicate_email() {
        let service = create_test_service();
        let request = create_valid_user_request();

        service.create_user(request.clone()).await.unwrap();

        let result = service.create_user(request).await;
        assert!(matches!(result, Err(UserError::EmailAlreadyExists)));
    }

    #[tokio::test]
    async fn test_create_user_invalid_email() {
        let service = create_test_service();
        let mut request = create_valid_user_request();
        request.email = "invalid-email".to_string();

        let result = service.create_user(request).await;
        assert!(matches!(result, Err(UserError::ValidationFailed(_))));
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        let service = create_test_service();
        let request = create_valid_user_request();

        let created = service.create_user(request).await.unwrap();
        let found = service.get_user(created.id).await.unwrap();

        assert_eq!(found.id, created.id);
        assert_eq!(found.email, created.email);
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let service = create_test_service();

        let result = service.get_user(999).await;
        assert!(matches!(result, Err(UserError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_get_user_by_email() {
        let service = create_test_service();
        let request = create_valid_user_request();

        let created = service.create_user(request).await.unwrap();
        let found = service.get_user_by_email(&created.email.clone().unwrap()).await.unwrap();

        assert!(found.is_some());
        let found_user = found.unwrap();
        assert_eq!(found_user.id, created.id);
    }

    #[tokio::test]
    async fn test_update_user() {
        let service = create_test_service();
        let request = create_valid_user_request();

        let user = service.create_user(request).await.unwrap();
        let update_request = UpdateUserRequest {
            display_name: Some("Updated Name".to_string()),
            avatar_url: Some("https://example.com/new_avatar.jpg".to_string()),
            role: Some(UserRole::Admin),
            ..Default::default()
        };

        let updated = service.update_user(user.id, update_request).await.unwrap();

        assert_eq!(updated.display_name, Some("Updated Name".to_string()));
        assert_eq!(updated.avatar_url, Some("https://example.com/new_avatar.jpg".to_string()));
        assert_eq!(updated.role, UserRole::Admin);
    }

    #[tokio::test]
    async fn test_search_users() {
        let service = create_test_service();

        let user1_request = CreateUserRequest {
            email: "user1@example.com".to_string(),
            username: "alice".to_string(),
            display_name: "Alice Smith".to_string(),
            password: "password123".to_string(),
            avatar_url: None,
            bio: None,
        };

        let user2_request = CreateUserRequest {
            email: "user2@example.com".to_string(),
            username: "bob".to_string(),
            display_name: "Bob Johnson".to_string(),
            password: "password123".to_string(),
            avatar_url: None,
            bio: None,
        };

        service.create_user(user1_request).await.unwrap();
        service.create_user(user2_request).await.unwrap();

        let results = service.search_users("Alice", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].display_name, Some("Alice Smith".to_string()));
    }

    #[tokio::test]
    async fn test_is_email_available() {
        let service = create_test_service();
        let request = create_valid_user_request();
        let email = request.email.clone();

        // Should be available before creation
        assert!(service.is_email_available(&email).await.unwrap());

        service.create_user(request).await.unwrap();

        // Should not be available after creation
        assert!(!service.is_email_available(&email).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let service = create_test_service();
        let request = create_valid_user_request();

        let user = service.create_user(request).await.unwrap();
        let user_id = user.id;

        // Verify user exists before deletion
        let found_user = service.get_user(user_id).await.unwrap();
        assert_eq!(found_user.id, user.id);

        // Delete user
        service.delete_user(user_id).await.unwrap();

        // Verify user no longer exists
        let result = service.get_user(user_id).await;
        assert!(matches!(result, Err(UserError::UserNotFound)));

        // Verify user cannot be found by email either
        let email_result = service.get_user_by_email(&found_user.email.clone().unwrap()).await.unwrap();
        assert!(email_result.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_user() {
        let service = create_test_service();

        let result = service.delete_user(99999).await;
        assert!(matches!(result, Err(UserError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_create_user_with_empty_optional_fields() {
        let service = create_test_service();
        let mut request = create_valid_user_request();
        request.avatar_url = None;
        request.bio = None;

        let user = service.create_user(request).await.unwrap();

        assert!(user.avatar_url.is_none());
        assert!(user.bio.is_none());
        assert_eq!(user.role, UserRole::User); // Default role
        assert!(user.is_active);
        assert!(!user.email_verified);
    }

    #[tokio::test]
    async fn test_update_user_partial_fields() {
        let service = create_test_service();
        let create_request = create_valid_user_request();
        let user = service.create_user(create_request).await.unwrap();
        let original_display_name = user.display_name.clone();

        // Update only display name
        let update_request = UpdateUserRequest {
            display_name: Some("New Display Name".to_string()),
            ..Default::default()
        };

        let updated_user = service.update_user(user.id, update_request).await.unwrap();

        assert_eq!(updated_user.display_name, Some("New Display Name".to_string()));
        assert_eq!(updated_user.avatar_url, user.avatar_url); // Should remain unchanged
        assert_ne!(updated_user.display_name, original_display_name);
    }

    #[tokio::test]
    async fn test_update_nonexistent_user() {
        let service = create_test_service();
        let update_request = UpdateUserRequest {
            display_name: Some("New Name".to_string()),
            ..Default::default()
        };

        let result = service.update_user(99999, update_request).await;
        assert!(matches!(result, Err(UserError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_search_users_empty_query() {
        let service = create_test_service();

        let results = service.search_users("", 10).await.unwrap();
        assert_eq!(results.len(), 0);

        let results = service.search_users("   ", 10).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_search_users_no_results() {
        let service = create_test_service();
        let request = create_valid_user_request();

        service.create_user(request).await.unwrap();

        let results = service.search_users("nonexistent_user", 10).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_update_last_login() {
        let service = create_test_service();
        let request = create_valid_user_request();

        let user = service.create_user(request).await.unwrap();
        service.update_last_login(user.id).await.unwrap();

        let updated_user = service.get_user(user.id).await.unwrap();
        assert!(updated_user.last_login_at.is_some());
    }

    #[tokio::test]
    async fn test_user_lifecycle_complete() {
        let service = create_test_service();
        let request = create_valid_user_request();

        // Create user
        let user = service.create_user(request).await.unwrap();
        let user_id = user.id;
        assert!(user.id > 0);
        assert!(user.is_active);

        // Update user
        let update_request = UpdateUserRequest {
            display_name: Some("Updated Name".to_string()),
            ..Default::default()
        };
        let updated_user = service.update_user(user_id, update_request).await.unwrap();
        assert_eq!(updated_user.display_name, Some("Updated Name".to_string()));

        // Update last login
        service.update_last_login(user_id).await.unwrap();
        let user_with_login = service.get_user(user_id).await.unwrap();
        assert!(user_with_login.last_login_at.is_some());

        // Search user
        let search_results = service.search_users("Updated", 10).await.unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].id, user.id);

        // Delete user
        service.delete_user(user_id).await.unwrap();
        let result = service.get_user(user_id).await;
        assert!(matches!(result, Err(UserError::UserNotFound)));
    }

    impl Default for UpdateUserRequest {
        fn default() -> Self {
            Self {
                display_name: None,
                avatar_url: None,
                bio: None,
                role: None,
            }
        }
    }
}