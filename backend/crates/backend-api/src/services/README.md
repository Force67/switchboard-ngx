# Service Layer Architecture

This directory contains the refactored business logic layer that separates core functionality from HTTP transport concerns.

## Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   REST API      │    │   WebSocket      │    │   Future APIs   │
│   (rest.rs)     │    │   (websocket.rs) │    │   (e.g., gRPC)  │
└─────────┬───────┘    └─────────┬────────┘    └─────────┬───────┘
          │                      │                       │
          ▼                      ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                        SERVICE LAYER                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐ │
│  │    auth     │ │    chat     │ │   message   │ │   invite  │ │
│  │  service.rs │ │  service.rs │ │ service.rs  │ │service.rs │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘ │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │   member    │ │   error.rs  │ │    mod.rs   │               │
│  │ service.rs  │ │             │ │             │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
└─────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                          ┌─────────────────┐
                          │   DATABASE      │
                          │   (SQLite)      │
                          └─────────────────┘
```

## Key Benefits

1. **Separation of Concerns**: Business logic is completely separate from HTTP concerns
2. **Reusability**: Same service functions can be used by REST, WebSocket, and future gateways
3. **Testability**: Services can be unit tested without HTTP dependencies
4. **Maintainability**: Changes to business logic don't require route handler updates
5. **Consistency**: Same validation and authorization logic applied across all transport layers

## Usage Examples

### REST Handler Example

```rust
// routes/rest.rs
pub async fn create_message(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<CreateMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    // 1. HTTP-specific: Extract auth token
    let token = require_bearer(&headers)?;
    let (user, _) = state.authenticate(&token).await?;

    // 2. Call service layer (pure business logic)
    let (message, member_ids) = message::create_message(
        state.db_pool(),
        &chat_id,
        user.id,
        req,
    ).await?;

    // 3. HTTP-specific: Broadcast events
    let event = ServerEvent::Message { /* ... */ };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    // 4. HTTP-specific: Return response
    Ok(Json(MessageResponse { message }))
}
```

### WebSocket Handler Example

```rust
// routes/websocket.rs
pub async fn handle_create_message_ws(
    state: &AppState,
    user_id: i64,
    chat_id: String,
    content: String,
) -> Result<Message, ServiceError> {
    // 1. WebSocket-specific: Create request from parsed message
    let create_req = CreateMessageRequest {
        content,
        role: "user".to_string(),
        model: None,
        message_type: Some("text".to_string()),
        thread_id: None,
        reply_to_id: None,
    };

    // 2. Call SAME service layer function
    let (message, member_ids) = message::create_message(
        state.db_pool(),
        &chat_id,
        user_id,
        create_req,
    ).await?;

    // 3. WebSocket-specific: Broadcast events
    let event = ServerEvent::Message { /* ... */ };
    state.broadcast_to_chat(&chat_id, &event).await;
    state.broadcast_to_users(member_ids, &event).await;

    // 4. WebSocket-specific: Return result for WebSocket response
    Ok(message)
}
```

### Testing Service Functions

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_create_message() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        // Setup test data...

        let result = message::create_message(
            &pool,
            "test-chat-id",
            1,
            CreateMessageRequest {
                content: "Hello, world!".to_string(),
                role: "user".to_string(),
                model: None,
                message_type: Some("text".to_string()),
                thread_id: None,
                reply_to_id: None,
            },
        ).await;

        assert!(result.is_ok());
        let (message, member_ids) = result.unwrap();
        assert_eq!(message.content, "Hello, world!");
    }
}
```

## Error Handling

The service layer uses a unified `ServiceError` enum that can be converted to transport-specific error types:

```rust
// service/error.rs
#[derive(Debug)]
pub enum ServiceError {
    NotFound,
    Forbidden,
    BadRequest(String),
    Database(sqlx::Error),
    Auth(switchboard_auth::AuthError),
    Config(String),
    Internal(String),
}

// Automatic conversion to API errors for REST
impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound => ApiError::not_found("Resource not found"),
            ServiceError::Forbidden => ApiError::forbidden("Access denied"),
            // ... other conversions
        }
    }
}
```

## Service Functions Pattern

Each service function follows this pattern:

1. **Input validation** - Validate parameters and business rules
2. **Authorization checks** - Verify user permissions
3. **Database operations** - Perform core business logic
4. **Return result** - Return business data and affected user IDs for broadcasting

Example signature:
```rust
pub async fn create_message(
    pool: &SqlitePool,           // Database connection
    chat_id: &str,               // Input parameters
    user_id: i64,
    req: CreateMessageRequest,
) -> Result<(Message, Vec<i64>), ServiceError>  // (Result, AffectedUserIds)
```

This architecture enables the same business logic to be consumed by any transport layer while maintaining clean separation of concerns.