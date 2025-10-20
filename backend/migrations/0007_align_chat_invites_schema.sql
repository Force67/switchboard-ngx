-- Align chat_invites table with the current application model.
-- The existing schema used legacy columns (invited_by_user_id, invited_email, expires_at, etc.)
-- which no longer match the API expectations. Rebuild the table with the new shape.

PRAGMA foreign_keys = OFF;

CREATE TABLE IF NOT EXISTS chat_invites_tmp (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    inviter_id INTEGER NOT NULL,
    invitee_email TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'accepted', 'rejected', 'expired')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
    FOREIGN KEY (inviter_id) REFERENCES users(id) ON DELETE CASCADE
);

INSERT INTO chat_invites_tmp (
    id,
    chat_id,
    inviter_id,
    invitee_email,
    status,
    created_at,
    updated_at
)
SELECT
    id,
    chat_id,
    invited_by_user_id,
    COALESCE(
        invited_email,
        (
            SELECT email
            FROM users
            WHERE users.id = chat_invites.invited_user_id
        ),
        'unknown@example.com'
    ) AS invitee_email,
    COALESCE(
        CASE status
            WHEN 'declined' THEN 'rejected'
            ELSE status
        END,
        'pending'
    ) AS status,
    COALESCE(created_at, datetime('now')),
    COALESCE(created_at, datetime('now'))
FROM chat_invites;

DROP TABLE chat_invites;
ALTER TABLE chat_invites_tmp RENAME TO chat_invites;

CREATE INDEX IF NOT EXISTS idx_chat_invites_chat_id ON chat_invites (chat_id);
CREATE INDEX IF NOT EXISTS idx_chat_invites_status ON chat_invites (status);

PRAGMA foreign_keys = ON;
