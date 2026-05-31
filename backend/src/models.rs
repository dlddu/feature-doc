//! Database row types.

#[derive(Clone, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub github_id: i64,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub struct Installation {
    pub id: String,
    pub user_id: String,
    pub installation_id: i64,
    pub account_login: Option<String>,
    pub account_type: Option<String>,
    pub repository_selection: Option<String>,
    pub created_at: i64,
}

/// A connected repository. Analysis-derived columns are reserved (NULL /
/// 'not_analyzed') until the analysis pipeline produces them — see
/// migration 0003.
#[derive(Clone, sqlx::FromRow)]
pub struct Repository {
    pub id: String,
    pub user_id: String,
    pub owner: String,
    pub name: String,
    pub branch: String,
    pub status: String,
    pub feature_count: Option<i64>,
    pub conflict_count: Option<i64>,
    pub spend_cents: Option<i64>,
    pub progress: Option<i64>,
    pub step: Option<String>,
    pub last_analyzed_at: Option<i64>,
    pub created_at: i64,
}
