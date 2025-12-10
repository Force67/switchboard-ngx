-- Placeholder migration file
CREATE TABLE IF NOT EXISTS migrations (
    id INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);