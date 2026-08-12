//! Verifies that `db::connect` applies the migrations and creates the schema.

use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_url() -> (String, std::path::PathBuf) {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("featuredoc-test-{}-{}.db", std::process::id(), nanos));
    let url = format!("sqlite://{}?mode=rwc", path.display());
    (url, path)
}

#[tokio::test]
async fn migrations_create_expected_tables() {
    let (url, path) = temp_db_url();
    let pool = featuredoc::db::connect(&url).await.expect("connect + migrate");

    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .fetch_all(&pool)
            .await
            .expect("query tables");
    let names: Vec<String> = rows.into_iter().map(|r| r.0).collect();

    for expected in [
        "users",
        "sessions",
        "installations",
        "llm_keys",
        "audit_log",
        "github_tokens",
        "analyses",
        "analysis_stages",
    ] {
        assert!(
            names.contains(&expected.to_string()),
            "expected table `{expected}` to exist, got {names:?}"
        );
    }

    // The worker's lease columns are what make a claim reclaimable (AC4.5); an
    // `ALTER TABLE` that silently went missing would only surface at runtime.
    let columns: Vec<(String,)> = sqlx::query_as("SELECT name FROM pragma_table_info('analyses')")
        .fetch_all(&pool)
        .await
        .expect("query analyses columns");
    let columns: Vec<String> = columns.into_iter().map(|r| r.0).collect();
    for expected in ["claimed_by", "claimed_at", "lease_expires_at", "started_at", "finished_at"] {
        assert!(
            columns.contains(&expected.to_string()),
            "expected `analyses.{expected}` to exist, got {columns:?}"
        );
    }

    pool.close().await;
    let _ = std::fs::remove_file(&path);
}
