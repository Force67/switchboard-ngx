# Switchboard API Test App

A comprehensive Rust application for testing the Switchboard REST API functionality.

## Features

- **User Management**: Create test users via dev-token authentication
- **Chat Management**: Create, list, and fetch chats
- **Message Handling**: Send and retrieve messages in chats
- **Provider Configuration**: Test AI provider setup
- **Complete Workflow Testing**: End-to-end API validation
- **Colored Output**: Beautiful, informative console output
- **Error Handling**: Comprehensive error reporting

## Prerequisites

- Rust 1.70+ installed
- Switchboard backend API running (default: `http://localhost:3030`)

## Installation

1. Navigate to the backend directory:
   ```bash
   cd /path/to/switchboard/backend
   ```

2. Build the test app:
   ```bash
   cargo build -p api-test-app
   ```

## Usage

### Basic Commands

1. **Test User Creation** - Create multiple test users:
   ```bash
   cargo run -p api-test-app -- test-users --count 5
   ```

2. **Test Chat Management** - Create chats for an existing user:
   ```bash
   cargo run -p api-test-app -- test-chats --user-token YOUR_TOKEN --count 3
   ```

3. **Complete Workflow Test** - End-to-end testing:
   ```bash
   cargo run -p api-test-app -- test-workflow --user-count 3 --chats-per-user 2 --messages-per-chat 4
   ```

4. **Run All Tests** - Execute all test scenarios:
   ```bash
   cargo run -p api-test-app -- run-all
   ```

### Custom API URL

To test against a different API endpoint:

```bash
cargo run -p api-test-app -- --api-url http://localhost:8080 run-all
```

## Test Scenarios

### 1. User Creation Tests
- Creates multiple users with unique email addresses
- Tests dev-token authentication endpoint
- Validates user creation response format
- Reports success/failure for each user

### 2. Chat Management Tests
- Lists existing chats for a user
- Creates new chats with different titles
- Fetches individual chat details
- Validates chat CRUD operations

### 3. Complete Workflow Tests
- **Step 1**: Creates multiple test users
- **Step 2**: Creates chats for each user
- **Step 3**: Sends messages to each chat
- **Step 4**: Verifies message creation and retrieval
- **Step 5**: Tests provider configuration

### 4. Provider Configuration Tests
- Tests AI provider setup (may fail if endpoint doesn't exist)
- Configures sample providers (OpenAI, Claude)
- Validates provider configuration flow

## Example Output

```
🚀 Switchboard API Test App
========================

🧪 Testing User Creation
=======================
🔵 Creating user: Test User 1 (testuser1@example.com)
✅ User created successfully: abc123-def456
🔵 Creating user: Test User 2 (testuser2@example.com)
✅ User created successfully: ghi789-jkl012
✅ Successfully created 2/2 users

🧪 Testing Chat Management
==========================
📋 Fetching chats for user
📊 Found 0 chats
🆕 Creating chat: Test Chat 1
✅ Chat created: chat-uuid-1
🆕 Creating chat: Test Chat 2
✅ Chat created: chat-uuid-2
✅ Successfully created 2 chats

📊 Workflow Summary
===================
✅ Users created: 2/2
✅ Chats created: 4
✅ Messages sent: 8
✅ Messages verified: 8

✅ All tests completed successfully!
```

## API Endpoints Tested

### Authentication
- `POST /api/auth/dev-token` - Create user and get session token

### Chats
- `GET /api/chats` - List user chats
- `POST /api/chats` - Create new chat
- `GET /api/chats/{id}` - Get chat details

### Messages
- `POST /api/chats/{id}/messages` - Send message
- `GET /api/chats/{id}/messages` - Get chat messages

### Providers (Experimental)
- `POST /api/providers` - Configure AI provider

## Error Handling

The app includes comprehensive error handling:
- HTTP status code validation
- JSON parsing error reporting
- Network error handling
- Clear error messages with context

## Development

### Adding New Tests

1. Add new commands to the `Commands` enum in `main.rs`
2. Implement the test function
3. Add API client methods if needed
4. Update documentation

### API Client Structure

The `ApiClient` struct provides methods for:
- HTTP request handling
- Response parsing
- Error handling
- Authentication token management

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

## Contributing

When adding new test scenarios:
1. Follow the existing code structure
2. Add comprehensive error handling
3. Include informative console output
4. Update this README

## License

This test app follows the same license as the Switchboard project.