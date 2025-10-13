-- Add role and model metadata to chat messages for proper persistence
ALTER TABLE messages
    ADD COLUMN role TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('user', 'assistant', 'system'));

ALTER TABLE messages
    ADD COLUMN model TEXT;
