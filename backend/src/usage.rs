//! Measured LLM usage and what it cost (AC4.6).
//!
//! Every call that a user's money paid for already leaves a row: the pipeline's
//! stage documents (`analysis_documents`), the per-feature dependency traces
//! (`feature_dependency_requests`), the assisted edits (`feature_doc_edits`), and
//! the feature drafts (`feature_additions`). Each of those rows carries the model
//! and the token counts the provider reported. What was missing was a place that
//! adds them up and an address that answers with the sum — this module is both.
//!
//! The sum is deliberately *derived* rather than kept in a counter column. A
//! counter has to be right at every write site and stays wrong once it drifts;
//! a sum over the rows that already exist cannot disagree with them. It also
//! means the figures are retroactive: analyses that ran before this slice report
//! their real spend, not zero.
//!
//! **Estimate, not an invoice.** Providers bill on their own schedule and on rates
//! that differ per model and change over time. The tokens here are measured, the
//! money is inferred from them at the rates below — which is exactly what AC4.6
//! asks for ("추정 비용") and what the pre-flight estimate on Connect Repository
//! never was: that one is a guess from repository size, made before any call.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use sqlx::SqlitePool;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::state::AppState;

/// Cents per one million tokens. One pair of rates for every model rather than a
/// table keyed by model name: a per-model table would have to be kept in step with
/// providers' price changes to stay true, and a stale table reads as fact while a
/// single documented rate reads as the estimate it is. The pair is the order of
/// magnitude of a mid-tier frontier model, output priced above input as providers
/// price it.
const INPUT_CENTS_PER_MTOK: i64 = 300;
const OUTPUT_CENTS_PER_MTOK: i64 = 1_500;

/// What has actually been spent, measured. `cost_cents` rounds *up*, so any usage
/// at all shows as at least one cent — a screen that reports `$0.00` next to a
/// non-zero call count would read as "free", which is the one thing this number
/// must never say. No usage still costs nothing.
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

/// Ceiling division of the rate product by a million — see [`Spend`].
fn cost_cents(input_tokens: i64, output_tokens: i64) -> i64 {
    let micros = input_tokens.max(0) * INPUT_CENTS_PER_MTOK
        + output_tokens.max(0) * OUTPUT_CENTS_PER_MTOK;
    if micros == 0 {
        return 0;
    }
    (micros + 999_999) / 1_000_000
}

/// Every row that represents LLM calls, flattened to (analysis, calls, tokens).
///
/// `analysis_documents` carries its own call count because one stage merges two
/// provider calls into one row (migration `0015`). The other three are one call per
/// row, and a NULL `model` is a row no call has landed in yet — a queued dependency
/// trace, a directly-edited scenario — so it contributes nothing.
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

/// One analysis's measured spend. Zero for an analysis that has not called anyone
/// yet, which is a real answer rather than a missing one.
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

/// `작업별` and `전체별` in one answer (test/04 시나리오 9): the per-analysis rows and
/// the total across them, for the signed-in user and no one else.
///
/// This is also the operator's address. There is no separate operator role in this
/// product — the same figures, per analysis and per stage, are reachable here and on
/// `/api/analyses/{id}`, and the per-stage breakdown behind them is in the log line
/// `llm usage recorded` (`worker_api::submit_document`). A second, privileged copy of
/// the same numbers would be a second thing that can disagree with the rows.
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
    // The total is costed from the summed tokens, not from the sum of the rows'
    // costs: rounding each row up first and adding would charge the ceiling once
    // per analysis. 전체별 is one figure about one pile of tokens.
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
        // Two analyses that each round up would report 2 cents between them; the
        // total is costed once, from the tokens.
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
