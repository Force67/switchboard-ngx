-- Fix the chats table to support the new multi-user schema
-- This migration makes user_id nullable and ensures proper chat_members entries

-- First, let's create a backup of current chats data
CREATE TABLE IF NOT EXISTS chats_backup AS SELECT * FROM chats;

-- Add chat_type column if it doesn't exist (this should have been added in migration 0005)
-- SQLite doesn't support IF NOT EXISTS for columns, so we need to be careful
-- We'll assume it might already exist from migration 0005

-- Update any existing chats that don't have chat_type set
UPDATE chats SET chat_type = 'direct' WHERE chat_type IS NULL;

-- For existing group chats, set chat_type based on is_group
UPDATE chats SET chat_type = 'group' WHERE is_group = 1 AND (chat_type IS NULL OR chat_type = 'direct');

-- Make user_id nullable by recreating the table
CREATE TABLE IF NOT EXISTS chats_temp (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id TEXT NOT NULL UNIQUE,
    user_id INTEGER,  -- Make nullable
    folder_id INTEGER,
    title TEXT NOT NULL,
    is_group BOOLEAN NOT NULL DEFAULT FALSE,  -- Keep for backwards compatibility
    chat_type TEXT NOT NULL DEFAULT 'direct' CHECK (chat_type IN ('direct', 'group', 'system')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
);

-- Copy data from chats to chats_temp
INSERT INTO chats_temp (
    id, public_id, user_id, folder_id, title, is_group,
    chat_type, created_at, updated_at
)
SELECT
    id, public_id, user_id, folder_id, title, is_group,
    COALESCE(chat_type, CASE WHEN is_group = 1 THEN 'group' ELSE 'direct' END),
    created_at, updated_at
FROM chats;

-- Drop the old chats table
DROP TABLE chats;

-- Rename chats_temp to chats
ALTER TABLE chats_temp RENAME TO chats;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_chats_user_id ON chats (user_id);
CREATE INDEX IF NOT EXISTS idx_chats_folder_id ON chats (folder_id);
CREATE INDEX IF NOT EXISTS idx_chats_created_at ON chats (created_at);
CREATE INDEX IF NOT EXISTS idx_chats_updated_at ON chats (updated_at);
CREATE INDEX IF NOT EXISTS idx_chats_chat_type ON chats (chat_type);

-- Ensure all existing chats have proper chat_members entries
INSERT OR IGNORE INTO chat_members (chat_id, user_id, role, joined_at)
SELECT c.id, c.user_id, 'owner', c.created_at
FROM chats c
WHERE c.user_id IS NOT NULL
AND NOT EXISTS (
    SELECT 1 FROM chat_members cm
    WHERE cm.chat_id = c.id AND cm.user_id = c.user_id AND cm.role = 'owner'
);

-- Clean up backup table
DROP TABLE IF EXISTS chats_backup;