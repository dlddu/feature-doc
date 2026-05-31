-- Per-user GitHub user-authorization OAuth token, envelope-encrypted (same scheme
-- as llm_keys). Used to verify installation ownership at setup, because the Setup
-- URL's installation_id is spoofable (GitHub docs). One row per user, replaced on
-- each login; never stored or returned in plaintext.
CREATE TABLE github_tokens (
    user_id     TEXT    PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    ciphertext  BLOB    NOT NULL,
    nonce       BLOB    NOT NULL,
    wrapped_dek BLOB    NOT NULL,
    dek_nonce   BLOB    NOT NULL,
    updated_at  INTEGER NOT NULL
);
