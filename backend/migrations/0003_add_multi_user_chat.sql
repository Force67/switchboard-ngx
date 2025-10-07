-- Multi-user chat tables migration
-- Adds support for group chats, individual messages, reactions, and invites

-- Modify existing chats table to support multi-user
ALTER TABLE chats ADD COLUMN is_group BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE chats DROP COLUMN messages; -- Messages will be in separate table

-- Table for chat membership with roles
CREATE TABLE IF NOT EXISTS chat_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
    joined_at TEXT NOT NULL,
    FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(chat_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_chat_members_chat_id ON chat_members (chat_id);
CREATE INDEX IF NOT EXISTS idx_chat_members_user_id ON chat_members (user_id);

-- Table for individual messages
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id TEXT NOT NULL UNIQUE,
    chat_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    message_type TEXT NOT NULL DEFAULT 'text' CHECK (message_type IN ('text', 'system', 'file')),
    reply_to_id INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (reply_to_id) REFERENCES messages(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages (chat_id);
CREATE INDEX IF NOT EXISTS idx_messages_user_id ON messages (user_id);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages (created_at);
CREATE INDEX IF NOT EXISTS idx_messages_reply_to_id ON messages (reply_to_id);

-- Table for message reactions
CREATE TABLE IF NOT EXISTS reactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    emoji TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(message_id, user_id, emoji)
);

CREATE INDEX IF NOT EXISTS idx_reactions_message_id ON reactions (message_id);
CREATE INDEX IF NOT EXISTS idx_reactions_user_id ON reactions (user_id);

-- Table for chat invites
CREATE TABLE IF NOT EXISTS chat_invites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id TEXT NOT NULL UNIQUE,
    chat_id INTEGER NOT NULL,
    invited_by_user_id INTEGER NOT NULL,
    invited_user_id INTEGER,
    invited_email TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'declined', 'expired')),
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by_user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_user_id) REFERENCES users(id) ON DELETE SET NULL,
    CHECK ((invited_user_id IS NOT NULL AND invited_email IS NULL) OR (invited_user_id IS NULL AND invited_email IS NOT NULL))
);

CREATE INDEX IF NOT EXISTS idx_chat_invites_chat_id ON chat_invites (chat_id);
CREATE INDEX IF NOT EXISTS idx_chat_invites_invited_user_id ON chat_invites (invited_user_id);
CREATE INDEX IF NOT EXISTS idx_chat_invites_status ON chat_invites (status);
CREATE INDEX IF NOT EXISTS idx_chat_invites_expires_at ON chat_invites (expires_at);