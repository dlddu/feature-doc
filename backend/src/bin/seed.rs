//! Test-data seeding for stub mode (`cargo run --bin seed`).
//!
//! Stub mode keeps no repository fixtures in application code; instead this binary
//! injects them into the database. It produces a coherent S02 demo under the
//! default stub identity ("stub") that `/api/auth/login` resolves to:
//!   - the stub user + a linked GitHub App installation,
//!   - the installation's accessible repositories (the candidates `connect`
//!     verifies against — AC1.1), and
//!   - those repositories already connected, so the Repositories home is non-empty.
//!
//! Point it at the same database the server uses (DATABASE_URL) and run the server
//! in FEATUREDOC_MODE=stub.

use sqlx::SqlitePool;

use featuredoc::config::Config;
use featuredoc::error::AppError;
use featuredoc::github_app::RepoRef;
use featuredoc::{db, github_api, github_app, installations, repositories, users};

/// A fixed stub installation id; any value works as long as the seeded
/// installation row and its repositories share it.
const INSTALLATION_ID: i64 = 424242;

/// The handle `/api/auth/login` defaults to (see auth::login).
const STUB_HANDLE: &str = "stub";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let db = db::connect(&config.database_url).await?;
    let count = seed(&db)
        .await
        .map_err(|e| anyhow::anyhow!("seed failed: {e:?}"))?;
    println!(
        "seeded {count} repositories for stub user '{STUB_HANDLE}' (login at /api/auth/login)"
    );
    Ok(())
}

/// Inserts the demo dataset; returns how many repositories were connected.
async fn seed(db: &SqlitePool) -> Result<usize, AppError> {
    // Same identity logging in as the default stub handle would resolve to.
    let gh = github_api::stub_user_from_code(STUB_HANDLE);
    let user = users::upsert(db, &gh).await?;

    // Link an installation so connect/list have a scope to work within.
    installations::upsert(
        db,
        &user.id,
        &installations::NewInstallation {
            installation_id: INSTALLATION_ID,
            account_login: Some("stub-account"),
            account_type: Some("User"),
            repository_selection: Some("selected"),
        },
    )
    .await?;

    let repos = [
        RepoRef {
            owner: "stub-account".to_string(),
            name: "payments-api".to_string(),
            default_branch: "main".to_string(),
        },
        RepoRef {
            owner: "stub-account".to_string(),
            name: "checkout-web".to_string(),
            default_branch: "main".to_string(),
        },
        RepoRef {
            owner: "stub-account".to_string(),
            name: "notif-worker".to_string(),
            default_branch: "main".to_string(),
        },
    ];

    // The accessible candidate set (what connect verifies against).
    github_app::set_installation_repositories(db, INSTALLATION_ID, &repos).await?;

    // Connect them so the Repositories home shows data immediately.
    for repo in &repos {
        repositories::upsert(db, &user.id, &repo.owner, &repo.name, &repo.default_branch).await?;
    }

    Ok(repos.len())
}
