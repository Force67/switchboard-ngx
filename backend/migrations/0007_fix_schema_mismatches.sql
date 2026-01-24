-- Migration 0007: Fix schema mismatches between repositories and database
-- This migration adds missing columns that the repository code expects

-- ============================================================================
-- FIX CHATS TABLE
-- Missing: description, avatar_url, status, created_by
-- Also fix chat_type constraint to include 'channel'
-- ============================================================================

-- SQLite doesn't support modifying CHECK constraints, so we recreate the table
-- First, create the new table with correct schema
CREATE TABLE chats_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id TEXT NOT NULL UNIQUE,
    user_id INTEGER,
    folder_id INTEGER,
    title TEXT NOT NULL,
    is_group BOOLEAN NOT NULL DEFAULT FALSE,
    chat_type TEXT NOT NULL DEFAULT 'direct' CHECK (chat_type IN ('direct', 'group', 'channel', 'system')),
    description TEXT,
    avatar_url TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived', 'deleted')),
    created_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
);

-- Copy data from old table, converting user_id to created_by as TEXT
INSERT INTO chats_new (id, public_id, user_id, folder_id, title, is_group, chat_type, created_at, updated_at, created_by)
SELECT id, public_id, user_id, folder_id, title, is_group,
       CASE WHEN chat_type IN ('direct', 'group', 'channel', 'system') THEN chat_type ELSE 'direct' END,
       created_at, updated_at, CAST(user_id AS TEXT)
FROM chats;

-- Drop old table and rename new one
DROP TABLE chats;
ALTER TABLE chats_new RENAME TO chats;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_chats_user_id ON chats (user_id);
CREATE INDEX IF NOT EXISTS idx_chats_folder_id ON chats (folder_id);
CREATE INDEX IF NOT EXISTS idx_chats_created_at ON chats (created_at);
CREATE INDEX IF NOT EXISTS idx_chats_updated_at ON chats (updated_at);
CREATE INDEX IF NOT EXISTS idx_chats_chat_type ON chats (chat_type);
CREATE INDEX IF NOT EXISTS idx_chats_status ON chats (status);
CREATE INDEX IF NOT EXISTS idx_chats_created_by ON chats (created_by);

-- Create index for status filtering
CREATE INDEX IF NOT EXISTS idx_chats_status ON chats (status);
CREATE INDEX IF NOT EXISTS idx_chats_created_by ON chats (created_by);

-- ============================================================================
-- FIX MESSAGES TABLE
-- Missing: sender_id (have user_id), status, deleted_at, reply_to_public_id, thread_public_id
-- ============================================================================

-- Add missing columns to messages table
ALTER TABLE messages ADD COLUMN sender_id INTEGER;
ALTER TABLE messages ADD COLUMN status TEXT NOT NULL DEFAULT 'sent' CHECK (status IN ('sent', 'delivered', 'read', 'deleted'));
ALTER TABLE messages ADD COLUMN deleted_at TEXT;
ALTER TABLE messages ADD COLUMN reply_to_public_id TEXT;
ALTER TABLE messages ADD COLUMN thread_public_id TEXT;

-- Migrate user_id to sender_id for existing records
UPDATE messages SET sender_id = user_id WHERE sender_id IS NULL;

-- Create indexes for new columns
CREATE INDEX IF NOT EXISTS idx_messages_sender_id ON messages (sender_id);
CREATE INDEX IF NOT EXISTS idx_messages_status ON messages (status);
CREATE INDEX IF NOT EXISTS idx_messages_deleted_at ON messages (deleted_at);

-- ============================================================================
-- FIX MESSAGE_ATTACHMENTS TABLE
-- Missing: public_id, uploader_id
-- Column name mismatch: file_size_bytes should be file_size
-- ============================================================================

-- Add missing columns
ALTER TABLE message_attachments ADD COLUMN public_id TEXT;
ALTER TABLE message_attachments ADD COLUMN uploader_id INTEGER;
ALTER TABLE message_attachments ADD COLUMN file_size INTEGER;

-- Generate public_ids for existing attachments (using hex of id as placeholder)
UPDATE message_attachments SET public_id = 'att_' || printf('%08x', id) WHERE public_id IS NULL;

-- Migrate file_size_bytes to file_size
UPDATE message_attachments SET file_size = file_size_bytes WHERE file_size IS NULL;

-- Create unique index on public_id
CREATE UNIQUE INDEX IF NOT EXISTS idx_message_attachments_public_id ON message_attachments (public_id);
CREATE INDEX IF NOT EXISTS idx_message_attachments_uploader_id ON message_attachments (uploader_id);

-- ============================================================================
-- ADD FOREIGN KEY RELATIONSHIPS (as indexes since SQLite doesn't enforce FKs by default)
-- ============================================================================

-- Foreign key indexes for referential integrity checks
CREATE INDEX IF NOT EXISTS idx_messages_reply_to_id ON messages (reply_to_id);
CREATE INDEX IF NOT EXISTS idx_chats_folder_id ON chats (folder_id);
