//! Measured LLM usage and what it cost (AC4.6).
//!
//! The sum is deliberately *derived* rather than kept in a counter column. A
//! counter has to be right at every write site and stays wrong once it drifts;
//! a sum over the rows that already exist cannot disagree with them.

use std::collections::HashMap;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use sqlx::SqlitePool;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::state::AppState;

/// One pair of rates for every model rather than a table keyed by model name: a
/// per-model table would have to be kept in step with providers' price changes to
/// stay true, and a stale table reads as fact while a single documented rate reads
/// as the estimate it is.
const INPUT_CENTS_PER_MTOK: i64 = 300;
const OUTPUT_CENTS_PER_MTOK: i64 = 1_500;

/// What has actually been spent, measured.
#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Spend {
    pub llm_calls: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_cents: i64,
}

impl Spend {
    fn new(llm_calls: i64, input_tokens: i64, output_tokens: i64) -> Self {
        Self {
            llm_calls,
            input_tokens,
            output_tokens,
            cost_cents: cost_cents(input_tokens, output_tokens),
        }
    }
}

fn cost_cents(input_tokens: i64, output_tokens: i64) -> i64 {
    let micros = input_tokens.max(0) * INPUT_CENTS_PER_MTOK
        + output_tokens.max(0) * OUTPUT_CENTS_PER_MTOK;
    if micros == 0 {
        return 0;
    }
    (micros + 999_999) / 1_000_000
}

/// A NULL `model` is a row no call has landed in yet — a queued dependency trace, a
/// directly-edited scenario — so it contributes nothing.
const CALL_ROWS: &str = "SELECT analysis_id, calls, input_tokens, output_tokens \
       FROM analysis_documents \
     UNION ALL \
     SELECT analysis_id, 1, input_tokens, output_tokens \
       FROM feature_dependency_requests WHERE model IS NOT NULL \
     UNION ALL \
     SELECT analysis_id, 1, input_tokens, output_tokens \
       FROM feature_doc_edits WHERE model IS NOT NULL \
     UNION ALL \
     SELECT analysis_id, 1, input_tokens, output_tokens \
       FROM feature_additions WHERE model IS NOT NULL";

pub async fn of_analysis(db: &SqlitePool, analysis_id: &str) -> Result<Spend, sqlx::Error> {
    let row: (i64, i64, i64) = sqlx::query_as(&format!(
        "SELECT COALESCE(SUM(calls), 0), COALESCE(SUM(input_tokens), 0), \
                COALESCE(SUM(output_tokens), 0) \
           FROM ({CALL_ROWS}) WHERE analysis_id = ?"
    ))
    .bind(analysis_id)
    .fetch_one(db)
    .await?;
    Ok(Spend::new(row.0, row.1, row.2))
}

/// What each pipeline stage spent, keyed by [`crate::pipeline`] stage key (AC4.6's
/// 검증 방법: 단계별 비용).
///
/// Only `analysis_documents` is read here, and that is why the stage axis needs no
/// mapping table: `worker_api::submit_document` rejects a `kind` that is not a stage
/// key, so the column *is* that axis. The other three tables in [`CALL_ROWS`] hold
/// calls a person asked for after the pipeline ran — a dependency trace, an LLM doc
/// edit, a feature addition — and belong to no stage. Attributing them to one would
/// invent an attribution the rows do not carry.
///
/// Two consequences a caller must not paper over. On calls and tokens these buckets
/// sum to *at most* [`of_analysis`]'s total, never more, because the rows here are a
/// subset of the ones it reads. On money they can sum to **more** than it:
/// [`cost_cents`] rounds every bucket up to the cent, which is the same reason
/// [`read`] costs a summed total rather than adding up costed rows.
pub async fn by_stage(
    db: &SqlitePool,
    analysis_id: &str,
) -> Result<HashMap<String, Spend>, sqlx::Error> {
    // Grouped rather than leaning on `UNIQUE(analysis_id, kind)` (migration 0005):
    // if a second row for one stage ever lands, a sum still answers correctly where
    // building the map row by row would silently keep whichever came last.
    let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
        "SELECT kind, COALESCE(SUM(calls), 0), COALESCE(SUM(input_tokens), 0), \
                COALESCE(SUM(output_tokens), 0) \
           FROM analysis_documents WHERE analysis_id = ? GROUP BY kind",
    )
    .bind(analysis_id)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(kind, calls, input, output)| (kind, Spend::new(calls, input, output)))
        .collect())
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct AnalysisSpendView {
    analysis_id: String,
    repo_owner: String,
    repo_name: String,
    branch: String,
    status: String,
    created_at: i64,
    llm_calls: i64,
    input_tokens: i64,
    output_tokens: i64,
    /// Not summed in SQL — [`cost_cents`] owns the rates, so the money is computed
    /// in one place whichever query produced the tokens.
    #[sqlx(default)]
    cost_cents: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UsageView {
    total: Spend,
    analyses: Vec<AnalysisSpendView>,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/usage", get(read))
}

async fn read(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<UsageView>, AppError> {
    let mut analyses = sqlx::query_as::<_, AnalysisSpendView>(&format!(
        "SELECT a.id AS analysis_id, a.repo_owner, a.repo_name, a.branch, a.status, \
                a.created_at, \
                COALESCE(SUM(u.calls), 0) AS llm_calls, \
                COALESCE(SUM(u.input_tokens), 0) AS input_tokens, \
                COALESCE(SUM(u.output_tokens), 0) AS output_tokens, \
                0 AS cost_cents \
           FROM analyses a LEFT JOIN ({CALL_ROWS}) u ON u.analysis_id = a.id \
          WHERE a.user_id = ? \
          GROUP BY a.id \
          ORDER BY a.created_at DESC, a.id DESC"
    ))
    .bind(&user.id)
    .fetch_all(&state.db)
    .await?;

    let mut total = Spend::default();
    for row in &mut analyses {
        row.cost_cents = cost_cents(row.input_tokens, row.output_tokens);
        total.llm_calls += row.llm_calls;
        total.input_tokens += row.input_tokens;
        total.output_tokens += row.output_tokens;
    }
    // Costed from the summed tokens, not from the sum of the rows' costs: rounding
    // each row up first and adding would charge the ceiling once per analysis.
    total.cost_cents = cost_cents(total.input_tokens, total.output_tokens);

    Ok(Json(UsageView { total, analyses }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_usage_costs_nothing() {
        assert_eq!(cost_cents(0, 0), 0);
    }

    #[test]
    fn any_usage_costs_at_least_one_cent() {
        assert_eq!(cost_cents(1, 0), 1);
        assert_eq!(cost_cents(0, 1), 1);
    }

    #[test]
    fn output_is_priced_above_input() {
        assert!(cost_cents(0, 1_000_000) > cost_cents(1_000_000, 0));
    }

    #[test]
    fn the_rates_are_applied_per_million_tokens() {
        assert_eq!(cost_cents(1_000_000, 0), INPUT_CENTS_PER_MTOK);
        assert_eq!(cost_cents(0, 1_000_000), OUTPUT_CENTS_PER_MTOK);
        assert_eq!(
            cost_cents(2_000_000, 3_000_000),
            2 * INPUT_CENTS_PER_MTOK + 3 * OUTPUT_CENTS_PER_MTOK
        );
    }

    #[test]
    fn summing_tokens_then_costing_is_not_the_same_as_costing_each_row() {
        let each = cost_cents(1, 0) + cost_cents(1, 0);
        assert_eq!(each, 2);
        assert_eq!(cost_cents(2, 0), 1);
    }

    #[test]
    fn spend_costs_the_tokens_it_carries() {
        let s = Spend::new(3, 1_000_000, 1_000_000);
        assert_eq!(s.llm_calls, 3);
        assert_eq!(s.cost_cents, INPUT_CENTS_PER_MTOK + OUTPUT_CENTS_PER_MTOK);
    }
}
