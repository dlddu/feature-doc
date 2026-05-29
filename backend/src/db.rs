//! SQLite pool plus idempotent schema setup. Runs on startup against the PVC
//! file in k8s, or `sqlite::memory:` in tests.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

pub type Db = SqlitePool;

pub async fn connect(database_url: &str) -> Result<Db, sqlx::Error> {
    let opts = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;
    migrate(&pool).await?;
    Ok(pool)
}

/// In-memory pool for tests. A single connection so the schema persists for
/// the life of the pool.
pub async fn connect_memory() -> Db {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("open in-memory sqlite");
    migrate(&pool).await.expect("migrate");
    pool
}

/// Create the credential-domain schema. `IF NOT EXISTS` keeps this safe to run
/// on every boot (walking-skeleton migration strategy).
pub async fn migrate(pool: &Db) -> Result<(), sqlx::Error> {
    let stmts = [
        r#"CREATE TABLE IF NOT EXISTS users (
            id            TEXT PRIMARY KEY,
            github_id     INTEGER NOT NULL UNIQUE,
            login         TEXT NOT NULL,
            avatar_url    TEXT,
            created_at    INTEGER NOT NULL
        )"#,
        r#"CREATE TABLE IF NOT EXISTS sessions (
            id            TEXT PRIMARY KEY,
            token_hash    TEXT NOT NULL UNIQUE,
            user_id       TEXT NOT NULL REFERENCES users(id),
            created_at    INTEGER NOT NULL,
            expires_at    INTEGER NOT NULL
        )"#,
        r#"CREATE TABLE IF NOT EXISTS installations (
            id              TEXT PRIMARY KEY,
            user_id         TEXT NOT NULL REFERENCES users(id),
            installation_id INTEGER NOT NULL,
            account_login   TEXT NOT NULL,
            repo_count      INTEGER NOT NULL,
            permissions     TEXT NOT NULL,
            created_at      INTEGER NOT NULL,
            UNIQUE(user_id, installation_id)
        )"#,
        // No plaintext column exists, by construction (AC4.3a).
        r#"CREATE TABLE IF NOT EXISTS llm_keys (
            id                TEXT PRIMARY KEY,
            user_id           TEXT NOT NULL REFERENCES users(id),
            provider          TEXT NOT NULL,
            fingerprint       TEXT NOT NULL,
            masked            TEXT NOT NULL,
            dek_wrapped       BLOB NOT NULL,
            dek_nonce         BLOB NOT NULL,
            ciphertext        BLOB NOT NULL,
            ciphertext_nonce  BLOB NOT NULL,
            status            TEXT NOT NULL,
            created_at        INTEGER NOT NULL,
            revoked_at        INTEGER
        )"#,
        r#"CREATE TABLE IF NOT EXISTS audit_log (
            id          TEXT PRIMARY KEY,
            user_id     TEXT,
            action      TEXT NOT NULL,
            detail      TEXT,
            created_at  INTEGER NOT NULL
        )"#,
    ];
    for stmt in stmts {
        sqlx::query(stmt).execute(pool).await?;
    }
    Ok(())
}

/// Seconds since the Unix epoch.
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
