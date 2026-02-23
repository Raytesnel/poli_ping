-- Add migration script here
CREATE TABLE IF NOT EXISTS moties
(
    id
    INTEGER
    PRIMARY
    KEY
    AUTOINCREMENT,
    external_id
    TEXT
    UNIQUE
    NOT
    NULL,
    title
    TEXT
    NOT
    NULL,
    description
    TEXT
    NOT
    NULL,
    result
    TEXT
    NOT
    NULL,
    timestamp
    TEXT
    NOT
    NULL
);

CREATE TABLE IF NOT EXISTS party_votes
(
    id
    INTEGER
    PRIMARY
    KEY
    AUTOINCREMENT,
    motie_id
    INTEGER
    NOT
    NULL,
    party
    TEXT
    NOT
    NULL,
    vote
    TEXT
    NOT
    NULL,
    FOREIGN
    KEY
(
    motie_id
) REFERENCES moties
(
    id
)
    );

CREATE TABLE IF NOT EXISTS user_votes
(
    id
    INTEGER
    PRIMARY
    KEY
    AUTOINCREMENT,
    user_id
    TEXT
    NOT
    NULL,
    motie_id
    INTEGER
    NOT
    NULL,
    vote
    TEXT
    NOT
    NULL,
    created_at
    TEXT
    DEFAULT
    CURRENT_TIMESTAMP,
    UNIQUE
(
    user_id,
    motie_id
),
    FOREIGN KEY
(
    motie_id
) REFERENCES moties
(
    id
)
    );