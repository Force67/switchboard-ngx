CREATE TABLE IF NOT EXISTS folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    color TEXT,
    parent_id INTEGER,
    collapsed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_folders_user_id
    ON folders (user_id);

CREATE INDEX IF NOT EXISTS idx_folders_parent_id
    ON folders (parent_id);

CREATE TABLE IF NOT EXISTS chats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    folder_id INTEGER,
    title TEXT NOT NULL,
    messages TEXT NOT NULL, -- JSON array of messages
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_chats_user_id
    ON chats (user_id);

CREATE INDEX IF NOT EXISTS idx_chats_folder_id
    ON chats (folder_id);

CREATE INDEX IF NOT EXISTS idx_chats_created_at
    ON chats (created_at);