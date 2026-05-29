//! Append-only audit trail for credential-handling code paths (AC4.3:
//! "자격증명을 다루는 코드 경로는 감사 가능하도록 기록된다"). Details are
//! intentionally non-sensitive — provider names, fingerprints, ids — never
//! key material.

use crate::db::{now, Db};

pub async fn record(db: &Db, user_id: Option<&str>, action: &str, detail: &str) {
    let id = uuid::Uuid::new_v4().to_string();
    let res = sqlx::query(
        "INSERT INTO audit_log (id, user_id, action, detail, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(user_id)
    .bind(action)
    .bind(detail)
    .bind(now())
    .execute(db)
    .await;
    if let Err(e) = res {
        // Auditing must never break the request; log and move on.
        tracing::error!(error = %e, action, "failed to write audit log");
    }
}
