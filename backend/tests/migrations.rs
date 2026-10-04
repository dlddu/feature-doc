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

const APPLIED: [(&str, &str); 18] = [
    ("0001_init.sql", "d318541ba2d08dd74d917f424c42657d6859a7692294d7c9b478238dabe59d3b"),
    ("0002_github_tokens.sql", "a62a0ecb0a7cdd303a1e06bc2420d7ab9a853836af36db9aabc6a35caa05514b"),
    ("0003_analyses.sql", "0e340a2bd6eda0ee014101230cb5bb7ff1a3b3b97e6d26e0eb46f38a101348b3"),
    ("0004_analysis_stages.sql", "2d7dbf96b079f097d35aa913dee58344b09382f8a876fb1a5cd67b106e556b37"),
    ("0005_analysis_documents.sql", "17c95ba9425f43350f0dfa2b32b8f47626130531b29a1bc93bf546b3ac80696a"),
    ("0006_discovery_strategies.sql", "07d98b1b88826e0ef165052f3cbf24c36a11de940a173e39484bea2ffa4be40f"),
    ("0007_feature_candidates.sql", "51a08cb041cc70b61eb273679b10cf9d5303271273c9f08508918ad256c3907d"),
    ("0008_feature_dependencies.sql", "971467e2967d810bcd6a212f46304a478baa386fcb8231fde4737fbebbd9b4d6"),
    ("0009_feature_doc_edits.sql", "8e0aed489f9dde994969469daa96d626fefb44bf81baff9c5f84d2d49897497f"),
    ("0010_feature_additions.sql", "bb60bdc808ed77266e39f24f27d6d17e8797a40ee2b559197692dec892b499df"),
    ("0011_feature_deletions.sql", "b842981ddf8bc45561d32c706564daf7bd55704fdbe0db859cecf5f91cad71d8"),
    ("0012_llm_language.sql", "dcd99f910326d43e8173653ec002c8d1dbcae20d6034c6e80e15c3f1c6f2a972"),
    ("0013_feature_doc_conflicts.sql", "d61a0e62c6f857e5e3c9a79b85c8b08ec6751dfaab8ab0bea22182da2edda466"),
    ("0014_feature_doc_restores.sql", "4ef81e08d8cd9a05bcc80baaa6f35d6d97c4b12f9b2e3fa664db6ba44093d2ea"),
    ("0015_llm_call_usage.sql", "fe83da97a819586bd1f33316e710ca6de9a680dde6b4080f4c22eeb6833bf8d3"),
    ("0016_access_requests.sql", "dfc096da2e550303a2e43dc51f09cbda41ab1d358b98859332b3cc46791b086f"),
    ("0017_push_subscriptions.sql", "506c45ad71e2918d28a5973006e3681e1874eec2b3fa70b35ed2995cdc00e057"),
    ("0018_analysis_public_repo.sql", "eae91c19499fba3ac327d81dc676d242bd65ceae35c1062d3ea6d86c164d330e"),
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
