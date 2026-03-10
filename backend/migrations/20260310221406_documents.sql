-- Add migration script here
-- Enable foreign keys (important for SQLite)
PRAGMA
    foreign_keys = ON;

CREATE TABLE IF NOT EXISTS moties
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    external_id TEXT NOT NULL UNIQUE,
    title       TEXT NOT NULL,
    description TEXT NOT NULL,
    result      TEXT NOT NULL,
    timestamp   TEXT NOT NULL
);

-- Each stemming (vote event)
CREATE TABLE IF NOT EXISTS stemmingen
(
    id            TEXT PRIMARY KEY,
    motie_id      INTEGER NOT NULL,
    besluit_id    TEXT,
    soort         TEXT    NOT NULL,
    status        TEXT,
    actor_naam    TEXT,
    actor_fractie TEXT,

    FOREIGN KEY (motie_id)
        REFERENCES moties (id)
        ON DELETE CASCADE
);

-- Optional simpler party vote table (if still needed)
CREATE TABLE IF NOT EXISTS party_votes
(
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    motie_id INTEGER NOT NULL,
    party    TEXT    NOT NULL,
    vote     TEXT    NOT NULL,

    FOREIGN KEY (motie_id)
        REFERENCES moties (id)
        ON DELETE CASCADE
);

-- User votes (your app logic)
CREATE TABLE IF NOT EXISTS user_votes
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    TEXT    NOT NULL,
    motie_id   INTEGER NOT NULL,
    vote       TEXT    NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (user_id, motie_id),
    FOREIGN KEY (motie_id)
        REFERENCES moties (id)
        ON DELETE CASCADE
);

CREATE TABLE motie_documents
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    motie_id    INTEGER NOT NULL,
    document_id TEXT    NOT NULL,
    FOREIGN KEY (motie_id) REFERENCES moties (id)
);

-- Helpful indexes
CREATE INDEX IF NOT EXISTS idx_stemmingen_motie_id ON stemmingen (motie_id);
CREATE INDEX IF NOT EXISTS idx_user_votes_user_id ON user_votes (user_id);
CREATE INDEX IF NOT EXISTS idx_user_votes_motie_id ON user_votes (motie_id);
