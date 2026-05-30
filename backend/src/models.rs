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
