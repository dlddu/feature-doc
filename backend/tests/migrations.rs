//! Verifies that `db::connect` applies the migrations and creates the schema —
//! and that the migrations that have already been applied somewhere are never
//! edited again.

use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

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

/// The SHA-256 of every migration that has been applied to a running deployment.
///
/// `sqlx` records a checksum of each migration **file** when it applies it, and
/// refuses to migrate — refuses to *start* — if the file it finds later differs:
///
/// ```text
/// migration 4 was previously applied but has been modified
/// ```
///
/// The checksum covers the whole file, not the SQL, so a comment or a stray
/// whitespace change is enough. On 2026-09-02 exactly that happened to
/// `0004_analysis_stages.sql` and the API could not start for 6h41m; the fix was to
/// restore the file byte for byte, which is why that file still carries a comment
/// pointing at a mockup that no longer exists. It is a deliberate revert, not a
/// loose end — see `migrations/README.md`.
///
/// Without this test the violation is invisible until the image pin rolls out and a
/// new pod tries to start, i.e. until it is already an outage. Here it is a red PR.
///
/// Adding a migration means adding a line: `sha256sum backend/migrations/00NN_*.sql`.
const APPLIED: [(&str, &str); 7] = [
    ("0001_init.sql", "ef20c326ad47e185d9a15b76a9a004daee6ce3eb2d54707afe3dff9446d6e95e"),
    ("0002_github_tokens.sql", "a62a0ecb0a7cdd303a1e06bc2420d7ab9a853836af36db9aabc6a35caa05514b"),
    ("0003_analyses.sql", "7ef7cce91e984e53ed4be299fe84e6c90c4169cbd38940a50bb1c743fd424969"),
    ("0004_analysis_stages.sql", "476e3e735173ed6354c88cca4af733dad728af02eb80e5f4bf738d4a17c2aa0e"),
    ("0005_analysis_documents.sql", "77f2787ee57cf7e9b23e5a7c05589e2523edea1ae176702dadc6e4457a691fd2"),
    ("0006_discovery_strategies.sql", "14d91b58df7d3e2f881b9048e2f50f805c402621e75be0b45d192ac625c4925d"),
    ("0007_feature_candidates.sql", "024e2b6a65ea0299ab6482f1a933889b9592475643e7c4b213a0775a73c07ff4"),
];

fn migrations_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
}

#[test]
fn applied_migrations_are_never_edited() {
    for (name, expected) in APPLIED {
        let path = migrations_dir().join(name);
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("{name} is missing ({e}) — an applied migration cannot be deleted either"));
        let actual = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(
            actual, expected,
            "\n{name} has been modified after it was applied.\n\
             sqlx will refuse to start against any database that already ran it \
             (\"migration N was previously applied but has been modified\").\n\
             Restore the file byte for byte and put the change in a new migration \
             — see backend/migrations/README.md."
        );
    }
}

/// The pin above is only as good as its coverage: a new migration that nobody adds
/// to `APPLIED` is unprotected from the moment it ships.
#[test]
fn every_migration_file_is_pinned() {
    let mut found: Vec<String> = std::fs::read_dir(migrations_dir())
        .expect("migrations directory")
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().to_string_lossy().into_owned();
            name.ends_with(".sql").then_some(name)
        })
        .collect();
    found.sort();
    let pinned: Vec<String> = APPLIED.iter().map(|(name, _)| (*name).to_string()).collect();
    assert_eq!(
        found, pinned,
        "\nevery migration must be listed in APPLIED (backend/tests/migrations.rs).\n\
         Add the new file with `sha256sum backend/migrations/00NN_*.sql`."
    );
}
