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
