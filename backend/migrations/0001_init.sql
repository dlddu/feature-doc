-- FeatureDoc S01 (Credentials Setup) schema.
-- Multi-user: every credential row is owned by a user and queried under that scope (AC4.7).
-- Timestamps are unix epoch seconds (INTEGER). Ids are opaque TEXT (uuid v4).

CREATE TABLE users (
    id          TEXT    PRIMARY KEY,
    github_id   INTEGER NOT NULL UNIQUE,
    login       TEXT    NOT NULL,
    name        TEXT,
    avatar_url  TEXT,
    created_at  INTEGER NOT NULL
);

-- Opaque session tokens (the cookie value is the primary key); never reversible to a user secret.
CREATE TABLE sessions (
    id          TEXT    PRIMARY KEY,
    user_id     TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER NOT NULL
);
CREATE INDEX idx_sessions_user ON sessions(user_id);

-- GitHub App installations linked to a user. We persist only the installation id
-- and display metadata; short-lived installation access tokens are minted on demand
-- and never stored (AC4.1).
CREATE TABLE installations (
    id                   TEXT    PRIMARY KEY,
    user_id              TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    installation_id      INTEGER NOT NULL,
    account_login        TEXT,
    account_type         TEXT,
    repository_selection TEXT,
    created_at           INTEGER NOT NULL,
    UNIQUE(user_id, installation_id)
);
CREATE INDEX idx_installations_user ON installations(user_id);

-- LLM API keys, envelope-encrypted. The plaintext key is NEVER stored:
--   ciphertext     = AES-256-GCM(plaintext_key) under a per-record DEK
--   wrapped_dek    = AES-256-GCM(DEK) under the process KEK
-- `fingerprint` is a non-reversible identifier; `masked` is a display-only hint.
CREATE TABLE llm_keys (
    id          TEXT    PRIMARY KEY,
    user_id     TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider    TEXT    NOT NULL,
    fingerprint TEXT    NOT NULL,
    masked      TEXT    NOT NULL,
    ciphertext  BLOB    NOT NULL,
    nonce       BLOB    NOT NULL,
    wrapped_dek BLOB    NOT NULL,
    dek_nonce   BLOB    NOT NULL,
    status      TEXT    NOT NULL DEFAULT 'active',
    created_at  INTEGER NOT NULL,
    revoked_at  INTEGER
);
CREATE INDEX idx_llm_keys_user ON llm_keys(user_id);

-- Append-only audit trail for credential-touching actions (AC4.3).
CREATE TABLE audit_log (
    id          TEXT    PRIMARY KEY,
    user_id     TEXT,
    action      TEXT    NOT NULL,
    detail      TEXT,
    created_at  INTEGER NOT NULL
);
CREATE INDEX idx_audit_user ON audit_log(user_id);
