//! Verifies that `db::connect` applies the migrations and creates the schema —
//! and that the migrations that have already been applied somewhere are never
//! edited again.

use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256, Sha384};

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
        "feature_dependency_requests",
        "feature_dependencies",
        "feature_doc_edits",
        "feature_additions",
        "feature_deletions",
        "feature_doc_conflicts",
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

// Pins the file bytes accepted by main at PR #98 — a comment-only pass whose
// production `_sqlx_migrations` repair was done by hand before the merge.
// This SHA-256 guard is not evidence of deployed DB repair; sqlx stores SHA-384.
// See migrations/README.md for provenance and the release preflight.
const APPLIED: [(&str, &str); 13] = [
    ("0001_init.sql", "d318541ba2d08dd74d917f424c42657d6859a7692294d7c9b478238dabe59d3b"),
    ("0002_github_tokens.sql", "a62a0ecb0a7cdd303a1e06bc2420d7ab9a853836af36db9aabc6a35caa05514b"),
    ("0003_analyses.sql", "097542fd1927f845c0a09d43a8666c5de1d92a39f09a8e1ecd278aa8da72856e"),
    ("0004_analysis_stages.sql", "a6c24aa861fbad9e4624ba51342d04d7f930796761de5b9efe4ecb36ad279953"),
    ("0005_analysis_documents.sql", "2cf2c26fcb7df17794d7963e8aa2d2d5707584d4c26277a6e62162ff1dbade06"),
    ("0006_discovery_strategies.sql", "9b76ecb4e66bb84a58135689b8512cc1adf690371513e7f1e6e06b60573cc694"),
    ("0007_feature_candidates.sql", "ff481a86036a7d8813ba054a332549e5fea7bec70f2dc876d49728bccc52b9f4"),
    ("0008_feature_dependencies.sql", "505d83149609b64f1632ac794c3632cb802876038b82c2e87c3df33ae8dcf8f6"),
    ("0009_feature_doc_edits.sql", "9ca4cdc6d0b75bcc996581f1d3f5cd9583b4920ca14036ba0653ca7f22b4faa8"),
    ("0010_feature_additions.sql", "7252693cd3b40c7414e0e84785324194e536e09c1ebb11b56bfafe0c37dae8e5"),
    ("0011_feature_deletions.sql", "a9664b05f143d48c27bacbd8240524f20ada9f8a81dd9874df39a77b61a0d736"),
    ("0012_llm_language.sql", "5daaff648db3b542ad625fe967e2810174601de0c5d1de5ffd669bda45579739"),
    ("0013_feature_doc_conflicts.sql", "25e2ce3d0e95473cfaa20f4d805e8a0617b6815b1377a83912fefe350be5bde3"),
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

async fn migration_checksums(pool: &sqlx::SqlitePool) -> Vec<(i64, Vec<u8>)> {
    sqlx::query_as("SELECT version, checksum FROM _sqlx_migrations ORDER BY version")
        .fetch_all(pool)
        .await
        .expect("read migration history")
}

#[tokio::test]
async fn current_migration_history_survives_reconnect() {
    let (url, path) = temp_db_url();
    let pool = featuredoc::db::connect(&url).await.expect("initial migration");
    let before = migration_checksums(&pool).await;
    assert_eq!(before.len(), APPLIED.len());
    for ((version, checksum), (name, _)) in before.iter().zip(APPLIED) {
        let expected_version: i64 = name.split('_').next().unwrap().parse().unwrap();
        assert_eq!(*version, expected_version);
        let bytes = std::fs::read(migrations_dir().join(name)).expect("migration bytes");
        assert_eq!(*checksum, Sha384::digest(bytes).to_vec(), "{name}");
    }
    pool.close().await;

    let pool = featuredoc::db::connect(&url).await.expect("matching history can restart");
    assert_eq!(migration_checksums(&pool).await, before);
    pool.close().await;
    std::fs::remove_file(path).expect("remove temporary database");
}

// SHA-384 of the six files changed by PR #44, from its parent
// 636ee771dd404ad7cf383d1bfadcc5036ab7fac6. These are source fixtures, not
// observations of a production database. Migration 0002 did not change.
const PRE_CLEANUP: [(i64, &str); 6] = [
    (1, "032f48530ee375551f2f229919ca3d77b77718bc3003121623a4d85f1fcee73048bf711725af980717a56692bdc73232"),
    (3, "848f967794d8e9e7965d454e52b4067c6c688b1db332d1e40a9a292606b8f5282d8596518b5fc64305e1b4611c73cf3b"),
    (4, "5af1095a2b17034bcaed34e42037a9d0d83e69d98bcc1067e0a9d0b40f8599f6eda14b109ed78c5421c0bb6efe8e5d1e"),
    (5, "43e1c1fe38d37bae84b4edd579818667c854b79ac46abc56a5be774dfc8c4662cb7e93f8659260166741de884fa3d620"),
    (6, "a4dec3f3e3c97bb3b872bb4797895783b71583dc2b396da080f0c1058d12ef801ed4dcdef99eaf4782c16ac81841263e"),
    (7, "5341110d7e138be0c74c046e514041ef60b45a210b426b80e401861bb845a2d22df0171d8fdc2283f326f9b7d63b2b26"),
];

#[tokio::test]
async fn pre_cleanup_checksums_are_rejected_without_repair() {
    for (version, old_hex) in PRE_CLEANUP {
        let (url, path) = temp_db_url();
        let pool = featuredoc::db::connect(&url).await.expect("initial migration");
        let current = migration_checksums(&pool).await;
        let old_checksum = hex::decode(old_hex).expect("historical SHA-384");
        assert_eq!(old_checksum.len(), 48);
        assert_ne!(
            &current.iter().find(|(v, _)| *v == version).unwrap().1,
            &old_checksum,
            "fixture must differ from the current file for migration {version}"
        );
        let changed = sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
            .bind(old_checksum)
            .bind(version)
            .execute(&pool)
            .await
            .expect("inject historical checksum into temporary database");
        assert_eq!(changed.rows_affected(), 1);
        let before = migration_checksums(&pool).await;
        pool.close().await;

        let error = featuredoc::db::connect(&url)
            .await
            .expect_err("historical mismatch must prevent startup");
        assert!(
            matches!(
                error.downcast_ref::<sqlx::migrate::MigrateError>(),
                Some(sqlx::migrate::MigrateError::VersionMismatch(actual)) if *actual == version
            ),
            "expected VersionMismatch({version}), got {error:#}"
        );

        let pool = sqlx::SqlitePool::connect(&url).await.expect("inspect without migrating");
        assert_eq!(migration_checksums(&pool).await, before, "no automatic repair");
        pool.close().await;
        std::fs::remove_file(path).expect("remove temporary database");
    }
}
