-- Major schema update to match the new ERD
-- This migration updates the existing schema and adds new tables for:
-- - Audit logs (message_edits, message_deletions)
-- - File attachments (message_attachments)
-- - User notifications
-- - Granular permissions system

-- Add chat_type column to chats table (safe to add)
ALTER TABLE chats ADD COLUMN chat_type TEXT NOT NULL DEFAULT 'direct' CHECK (chat_type IN ('direct', 'group', 'system'));

-- Update existing group chats to set the correct chat_type
UPDATE chats SET chat_type = 'group' WHERE is_group = TRUE;

-- Note: We can't DROP is_group in SQLite, but we can stop using it
-- The is_group column will remain but be ignored in future code

-- Add thread_id to messages table (new column, should be safe)
ALTER TABLE messages ADD COLUMN thread_id INTEGER;

-- Create index for thread_id
CREATE INDEX IF NOT EXISTS idx_messages_thread_id ON messages (thread_id);

-- Add credential_encrypted column to user_identities if it doesn't exist
-- Note: SQLite doesn't support "IF NOT EXISTS" for columns, but we can use a pragma check
ALTER TABLE user_identities ADD COLUMN credential_encrypted TEXT;

-- Migrate existing secret data to credential_encrypted if credential_encrypted is NULL
UPDATE user_identities SET credential_encrypted = secret WHERE credential_encrypted IS NULL AND secret IS NOT NULL;

-- Now create the new tables for audit logs, attachments, notifications, and permissions

-- Message edits audit table
CREATE TABLE IF NOT EXISTS message_edits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    edited_by_user_id INTEGER NOT NULL,
    old_content TEXT NOT NULL,
    new_content TEXT NOT NULL,
    edited_at TEXT NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (edited_by_user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_message_edits_message_id ON message_edits (message_id);
CREATE INDEX IF NOT EXISTS idx_message_edits_edited_by ON message_edits (edited_by_user_id);
CREATE INDEX IF NOT EXISTS idx_message_edits_edited_at ON message_edits (edited_at);
CREATE INDEX IF NOT EXISTS idx_message_edits_message_timeline ON message_edits (message_id, edited_at);

-- Message deletions audit table
CREATE TABLE IF NOT EXISTS message_deletions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    deleted_by_user_id INTEGER NOT NULL,
    reason TEXT,
    deleted_at TEXT NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (deleted_by_user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_message_deletions_message_id ON message_deletions (message_id);
CREATE INDEX IF NOT EXISTS idx_message_deletions_deleted_by ON message_deletions (deleted_by_user_id);
CREATE INDEX IF NOT EXISTS idx_message_deletions_deleted_at ON message_deletions (deleted_at);

-- Message attachments table
CREATE TABLE IF NOT EXISTS message_attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    file_name TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_url TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_message_attachments_message_id ON message_attachments (message_id);
CREATE INDEX IF NOT EXISTS idx_message_attachments_created_at ON message_attachments (created_at);

-- Notifications table
CREATE TABLE IF NOT EXISTS notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications (user_id);
CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications (read);
CREATE INDEX IF NOT EXISTS idx_notifications_user_unread ON notifications (user_id, read);
CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications (created_at);

-- Permissions table for granular access control
CREATE TABLE IF NOT EXISTS permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id INTEGER NOT NULL,
    permission_level TEXT NOT NULL CHECK (permission_level IN ('read', 'write', 'admin')),
    granted_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, resource_type, resource_id)
);

CREATE INDEX IF NOT EXISTS idx_permissions_user_id ON permissions (user_id);
CREATE INDEX IF NOT EXISTS idx_permissions_resource ON permissions (resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_permissions_level ON permissions (permission_level);
CREATE INDEX IF NOT EXISTS idx_permissions_granted_at ON permissions (granted_at);

-- Add additional indexes for performance
CREATE INDEX IF NOT EXISTS idx_chats_chat_type ON chats (chat_type);
CREATE INDEX IF NOT EXISTS idx_chats_updated_at ON chats (updated_at);
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users (created_at);