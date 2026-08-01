//! SQLite connection pool + migrations.

use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;

/// Opens (creating if absent) the SQLite database at `database_url`, enables
/// foreign-key enforcement, and applies all pending migrations.
///
/// `journal_mode` is pinned to `DELETE` on purpose. The database file lives on
/// an EFS volume, and WAL requires a `-shm` mapping shared by every process
/// that opens the file — something a network filesystem does not guarantee.
/// sqlx leaves `journal_mode` untouched by default as of 0.8 (it defaulted to
/// WAL through 0.7), so relying on that default would silently reintroduce WAL
/// on a version bump. The Deployment runs `replicas: 1` with strategy
/// `Recreate`, so there is only ever one writer.
///
/// `synchronous` is pinned to `FULL` for the same reason: on the rollback
/// journal, `NORMAL` does not guarantee that committed transactions survive a
/// power loss. sqlx 0.8 does not emit the pragma unless asked (falling back to
/// SQLite's compiled-in default of FULL), and journal_mode has already shown
/// that such defaults can shift between releases, so we state it explicitly.
pub async fn connect(database_url: &str) -> anyhow::Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Delete)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
