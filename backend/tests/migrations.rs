//! Migration application, schema shape, and applied-migration immutability.

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
        "feature_doc_restores",
        "push_subscriptions",
    ] {
        assert!(
            names.contains(&expected.to_string()),
            "expected table `{expected}` to exist, got {names:?}"
        );
    }

    // An `ALTER TABLE` that silently went missing would only surface at runtime.
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

const APPLIED: [(&str, &str); 17] = [
    ("0001_init.sql", "d318541ba2d08dd74d917f424c42657d6859a7692294d7c9b478238dabe59d3b"),
    ("0002_github_tokens.sql", "a62a0ecb0a7cdd303a1e06bc2420d7ab9a853836af36db9aabc6a35caa05514b"),
    ("0003_analyses.sql", "097542fd1927f845c0a09d43a8666c5de1d92a39f09a8e1ecd278aa8da72856e"),
    ("0004_analysis_stages.sql", "36c48cd0fda863ce651c132df52db602daefcd93f052a45dfa7d40e97bd0291d"),
    ("0005_analysis_documents.sql", "6e3ecfe9205ebdc9d18b07659e5fe34952996e3cb0360639516e61c0365f9cfd"),
    ("0006_discovery_strategies.sql", "07d98b1b88826e0ef165052f3cbf24c36a11de940a173e39484bea2ffa4be40f"),
    ("0007_feature_candidates.sql", "80005da5f654bd3406a34d3233b518946f0d568eb0dc457e289696751bf6ce07"),
    ("0008_feature_dependencies.sql", "4c5054205e41c4a7de3b8d44b92c1bd5b10c23bbc79a5aad811baa0a7c1a286c"),
    ("0009_feature_doc_edits.sql", "9ca4cdc6d0b75bcc996581f1d3f5cd9583b4920ca14036ba0653ca7f22b4faa8"),
    ("0010_feature_additions.sql", "7252693cd3b40c7414e0e84785324194e536e09c1ebb11b56bfafe0c37dae8e5"),
    ("0011_feature_deletions.sql", "a9664b05f143d48c27bacbd8240524f20ada9f8a81dd9874df39a77b61a0d736"),
    ("0012_llm_language.sql", "5daaff648db3b542ad625fe967e2810174601de0c5d1de5ffd669bda45579739"),
    ("0013_feature_doc_conflicts.sql", "25e2ce3d0e95473cfaa20f4d805e8a0617b6815b1377a83912fefe350be5bde3"),
    ("0014_feature_doc_restores.sql", "2d0a65161a5370d4f9c77fe6fddb1a82f0c454a7e7e19abf44a48c4691448b20"),
    ("0015_llm_call_usage.sql", "fe83da97a819586bd1f33316e710ca6de9a680dde6b4080f4c22eeb6833bf8d3"),
    ("0016_access_requests.sql", "d332e2812f8550b62c58082abc6bd17544605471de365911d6043cf5f0aabedc"),
    ("0017_push_subscriptions.sql", "506c45ad71e2918d28a5973006e3681e1874eec2b3fa70b35ed2995cdc00e057"),
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
