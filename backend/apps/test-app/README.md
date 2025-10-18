# Switchboard API Test App

A comprehensive Rust application for testing the Switchboard REST API functionality.

## Features

- **User Management**: Create test users via dev-token authentication
- **Chat Management**: Create, list, and fetch chats
- **Message Handling**: Send and retrieve messages in chats
- **Complete Workflow Testing**: End-to-end API validation
- **Clean Output**: Informative console output
- **Error Handling**: Comprehensive error reporting

## Prerequisites

- Rust 1.70+ installed
- Switchboard backend API running (default: `http://localhost:3030`)

## Installation

1. Navigate to the test app directory:
   ```bash
   cd backend/apps/test-app
   ```

2. Build the test app:
   ```bash
   cargo build --release
   ```

## Usage

### Basic Commands

1. **Test User Creation** - Create multiple test users:
   ```bash
   cargo run -- test-users --count 5
   ```

2. **Test Chat Management** - Create chats for an existing user:
   ```bash
   cargo run -- test-chats --user-token YOUR_TOKEN --count 3
   ```

3. **Complete Workflow Test** - End-to-end testing:
   ```bash
   cargo run -- test-workflow --user-count 3 --chats-per-user 2 --messages-per-chat 4
   ```

4. **Run All Tests** - Execute all test scenarios:
   ```bash
   cargo run -- run-all
   ```

### Custom API URL

To test against a different API endpoint:

```bash
cargo run -- --api-url http://localhost:8080 run-all
```

## Test Scenarios

### 1. User Creation Tests
- Creates multiple users with unique email addresses
- Tests dev-token authentication endpoint
- Validates user creation response format
- Reports success/failure for each user

### 2. Chat Management Tests
- Creates new chats with different titles
- Validates chat CRUD operations
- Reports success/failure for each chat

### 3. Complete Workflow Tests
- **Step 1**: Creates test users
- **Step 2**: Creates chats for each user
- **Step 3**: Sends messages to each chat
- **Step 4**: Validates message creation

## Example Output

```
Switchboard API Test App
========================

Testing User Creation
=======================
Creating user: Test User 1 (testuser1@example.com)
User created successfully: abc123-def456
Creating user: Test User 2 (testuser2@example.com)
User created successfully: ghi789-jkl012

Testing Chat Management
==========================
Creating chat: Test Chat 1
Chat created: chat-uuid-1
Creating chat: Test Chat 2
Chat created: chat-uuid-2

Testing Complete Workflow
===========================
Creating user: Workflow User (workflowuser@example.com)
User created successfully: user-uuid-123
Created chat: Workflow Chat 1
Chat created: chat-uuid-456
Sent message 1 to chat 1
Sent message 2 to chat 1
Workflow test completed!

All tests completed successfully!
```

## API Endpoints Tested

### Authentication
- `POST /api/auth/dev-token` - Create user and get session token

### Chats
- `POST /api/chats` - Create new chat

### Messages
- `POST /api/chats/{id}/messages` - Send message

## Error Handling

The app includes comprehensive error handling:
- HTTP status code validation
- JSON parsing error reporting
- Network error handling
- Clear error messages with context

## Development

### Running Tests

```bash
# Run specific test
cargo run -- test-users --count 2

# Run all tests
cargo run -- run-all

# Build for production
cargo build --release
```

### Adding New Tests

1. Add new commands to the `Commands` enum in `main.rs`
2. Implement the test function
3. Add API client methods if needed
4. Update documentation

## Configuration

The app accepts the following configuration:
- `--api-url`: Base URL for the API (default: `http://localhost:3030`)
- Command-specific parameters for each test scenario

## Troubleshooting

### Common Issues

1. **Connection Refused**: Ensure the Switchboard API is running
2. **Authentication Failures**: Check if dev-token endpoint is enabled
3. **JSON Parsing Errors**: Verify API response formats match expected structures

### Debug Mode

For detailed error information, the app provides:
- Full error messages
- HTTP status codes
- Response body content
- Request context

## License

This test app follows the same license as the Switchboard project.