REST endpoints currently live in `switchboard_gateway` (Axum). Routes are registered at the root (e.g. `/chats`); OpenAPI annotations still refer to `/api/...` but there is no automatic prefixing.

## Active routes
### Auth
- `GET /github/login` – returns GitHub OAuth authorize URL (placeholder, 503 when unconfigured).
- `POST /github/callback` – exchange GitHub code for a session (not yet implemented).
- `POST /logout` – placeholder logout endpoint.
- `GET /me` – fetch current user profile from token.
- `GET /dev/token` – debug-only helper to mint a session token.

### Chats
- `GET /chats?folder_id&limit&offset` – list chats the caller belongs to.
- `POST /chats` – create a chat (uses placeholder user id until auth is wired).
- `GET /chats/:chat_id` – fetch chat details if the caller is a member.
- `PUT /chats/:chat_id` – update chat metadata (admin/owner; placeholder auth).
- `DELETE /chats/:chat_id` – delete a chat (owner only).

### Messages
- `GET /chats/:chat_id/messages?limit&offset&before&after&thread_id` – list messages in a chat.
- `POST /chats/:chat_id/messages` – create a message (auth placeholder, membership required).
- `GET /chats/:chat_id/messages/:message_id` – fetch a single message.
- `PUT /chats/:chat_id/messages/:message_id` – edit content (sender or chat admin).
- `DELETE /chats/:chat_id/messages/:message_id` – delete message (sender or chat admin).

### Invites
- `GET /chats/:chat_id/invites?status&limit&offset` – list invites for a chat (admin/owner).
- `GET /invites?status&limit&offset` – list invites addressed to the caller.
- `POST /chats/:chat_id/invites` – create an invite (admin/owner; optional role, expiry hours).
- `GET /invites/:invite_id` – fetch invite details (inviter only in current code).
- `POST /invites/:invite_id/respond` – accept or reject an invite (auth placeholder).
- `DELETE /invites/:invite_id` – delete an invite (inviter or chat admin/owner).

### Members
- `GET /chats/:chat_id/members?role&limit&offset` – list members in a chat.
- `GET /chats/:chat_id/members/:member_id` – fetch a member record.
- `PUT /chats/:chat_id/members/:member_id/role` – change a member’s role (admin/owner; placeholder auth).
- `DELETE /chats/:chat_id/members/:member_id` – remove a member (admins; blocks removing last owner).
- `POST /chats/:chat_id/leave` – leave a chat unless you are the last owner.

### Attachments
- `GET /chats/:chat_id/attachments?message_id&file_type&limit&offset` – list attachments in a chat.
- `GET /chats/:chat_id/messages/:message_id/attachments` – list attachments for a message.
- `POST /chats/:chat_id/messages/:message_id/attachments` – upload base64 file data (50MB limit, placeholder auth).
- `GET /attachments/:attachment_id` – fetch attachment metadata.
- `GET /attachments/:attachment_id/download` – download placeholder response.
- `DELETE /attachments/:attachment_id` – delete an attachment (uploader or chat admin/owner).

## Legacy/unused definitions
Not mounted by `create_rest_routes` but present in the codebase for future/older flows:
- `GET /health` (simple health check).
- `POST /api/chat` (LLM chat completion, multipart).
- `GET /api/models` (list model metadata).
- Folder CRUD under `/api/folders` and `/api/folders/:folder_id`.
- Notification routes under `/api/notifications` (list, unread count, mark read, delete).
- Permission routes under `/api/permissions/...` and `/api/users/:user_id/permissions`.
