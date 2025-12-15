# Switchboard NGX API Documentation

## Base URL
```
http://localhost:7070/api/v1
```

## Authentication

The API uses JWT token-based authentication:
- Include token in Authorization header: `Authorization: Bearer <token>`
- Or as query parameter for WebSocket connections: `?token=<token>`
- Default token TTL: 24 hours

### Public Endpoints (No Authentication Required)
- `GET /api/v1/health` - Health check
- `GET /api/v1/auth/github/login` - Get GitHub OAuth URL
- `POST /api/v1/auth/github/callback` - Exchange OAuth code for session
- `GET /api/v1/auth/dev/token` - Development endpoint (when `SWITCHBOARD_DEV_AUTH_FALLBACK` is enabled)

## Response Formats

### Success Response
```json
{
  "data": { ... } // Response data
}
```

### Error Response
```json
{
  "error": "ERROR_CODE",
  "message": "Human readable error message"
}
```

## Active Routes

### Authentication
- `GET /api/v1/auth/github/login` - Returns GitHub OAuth authorize URL
- `POST /api/v1/auth/github/callback` - Exchange GitHub code for a session token
- `POST /api/v1/auth/logout` - Invalidate current session
- `GET /api/v1/auth/me` - Fetch current user profile
- `GET /api/v1/auth/dev/token` - Development endpoint to mint a session token

### Models
- `GET /api/v1/models` - List available LLM models from OpenRouter

### Chat Completion
- `POST /api/v1/chat` - Send chat completion request (multipart/form-data)
  - Parameters:
    - `prompt` (string, required): Chat prompt
    - `model` (string, optional): Model to use
    - `images` (file, optional): Image attachments

### Chats
- `GET /api/v1/chats?folder_id&limit&offset` - List user's chats
- `POST /api/v1/chats` - Create new chat
  - Body: `{"name": "Chat Name", "folder_id": "optional"}`
- `GET /api/v1/chats/:chat_id` - Fetch chat details
- `PUT /api/v1/chats/:chat_id` - Update chat metadata
- `DELETE /api/v1/chats/:chat_id` - Delete chat (owner only)

### Messages
- `GET /api/v1/chats/:chat_id/messages?limit&offset&before&after&thread_id` - List messages
- `POST /api/v1/chats/:chat_id/messages` - Create new message
  - Body: `{"content": "Message content", "thread_id": "optional"}`
- `GET /api/v1/chats/:chat_id/messages/:message_id` - Fetch single message
- `PUT /api/v1/chats/:chat_id/messages/:message_id` - Edit message content
- `DELETE /api/v1/chats/:chat_id/messages/:message_id` - Delete message

### Folders
- `GET /api/v1/folders` - List user's folders
- `POST /api/v1/folders` - Create new folder
  - Body: `{"name": "Folder Name"}`
- `GET /api/v1/folders/:folder_id` - Fetch folder details
- `PUT /api/v1/folders/:folder_id` - Update folder
- `DELETE /api/v1/folders/:folder_id` - Delete folder

### Members
- `GET /api/v1/chats/:chat_id/members?role&limit&offset` - List chat members
- `GET /api/v1/members/:member_id` - Fetch member record
- `PUT /api/v1/members/:member_id/role` - Change member role
  - Body: `{"role": "admin|member"}`
- `DELETE /api/v1/members/:member_id` - Remove member
- `POST /api/v1/members/leave/:chat_id` - Leave chat

### Invites
- `GET /api/v1/invites?status&limit&offset` - List user's invites
- `GET /api/v1/invites/:invite_id` - Fetch invite details
- `POST /api/v1/invites/:invite_id/respond` - Accept or reject invite
  - Body: `{"accept": true|false}`
- `DELETE /api/v1/invites/:invite_id` - Delete invite
- `GET /api/v1/chats/:chat_id/invites` - List chat invites
- `POST /api/v1/chats/:chat_id/invites` - Create invite
  - Body: `{"role": "optional", "expires_in_hours": 168}`

### Attachments
- `GET /api/v1/attachments?chat_id&message_id&file_type&limit&offset` - List attachments
- `POST /api/v1/attachments` - Upload attachment (base64, max 50MB)
- `GET /api/v1/attachments/:attachment_id` - Fetch attachment metadata
- `GET /api/v1/attachments/:attachment_id/download` - Download attachment
- `DELETE /api/v1/attachments/:attachment_id` - Delete attachment

## WebSocket Endpoints

### User WebSocket
- `GET /ws/user?token=<token>` - User-specific updates

### Chat WebSocket
- `GET /ws/chat?token=<token>&chat_id=<id>` - Chat-specific updates

### WebSocket Events
**Client -> Server:**
- `ping` - Keep-alive
- `subscribe`/`unsubscribe` - Manage chat subscriptions
- `send_message` - Send message via WebSocket
- `update_message`/`delete_message` - Message operations
- `typing` - Typing indicator

**Server -> Client:**
- `message_updated` - Message changes
- `member_updated` - Member status changes
- `typing` - Typing notifications
- `invite_updated` - Invite status changes

## Rate Limits
- Rate limiting middleware is implemented but currently placeholder
- No specific limits defined yet

## CORS
- Configurable allowed origins via `SWITCHBOARD__CORS__ALLOWED_ORIGINS`
- Defaults to `http://localhost:3000` for development

## Versioning Strategy
- Current API version: v1
- Version included in path: `/api/v1/`
- Breaking changes will result in a new version (e.g., `/api/v2/`)
- Backward-compatible changes will not increment version
- Deprecation notices will be provided for deprecated endpoints

## Migration from Unversioned API
If you were using the API before versioning was introduced:
- Replace `/api/` with `/api/v1/` in all endpoint URLs
- All existing functionality remains the same
- WebSocket endpoints are not versioned (remain at `/ws/`)

## OpenAPI/Swagger Documentation
Interactive API documentation is available when running in debug mode:
- Swagger UI: `http://localhost:7070/swagger-ui`
- OpenAPI JSON: `http://localhost:7070/api-docs/openapi.json`