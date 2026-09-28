use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::state::AppState;

const WINDOW: usize = 18;
const MAX_BYTES: usize = 64 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/analyses/{id}/features/{key}/evidence", get(read))
}

#[derive(Deserialize)]
struct Ask {
    path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExcerptView {
    path: String,
    symbol: Option<String>,
    start_line: usize,
    end_line: usize,
    of_lines: usize,
    lines: Vec<String>,
    truncated: bool,
}

async fn read(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, key)): Path<(String, String)>,
    Query(ask): Query<Ask>,
) -> Result<Json<ExcerptView>, AppError> {
    let (owner, name, branch, _) = crate::analysis::owned_analysis(&state, &user.id, &id).await?;

    let content = sqlx::query_scalar::<_, String>(
        "SELECT content FROM analysis_documents WHERE analysis_id = ? AND kind = ?",
    )
    .bind(&id)
    .bind(crate::pipeline::ACCEPTANCE_DEPENDENCIES)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let doc: Value = serde_json::from_str(&content)
        .map_err(|_| AppError::internal("stored document is unreadable"))?;

    let symbol = cited(&doc, &key, &ask.path).ok_or(AppError::NotFound)?;

    let token = match crate::installations::get_for_user(&state.db, &user.id).await? {
        Some(inst) => crate::github_app::mint_installation_token(&state, inst.installation_id)
            .await
            .ok()
            .map(|minted| minted.token),
        None => None,
    };

    let mut files = crate::repo_scan::read_files(
        &state.http,
        state.config.doubles.repo_scan,
        &state.config.github.api_base,
        &owner,
        &name,
        &branch,
        token.as_deref(),
        std::slice::from_ref(&ask.path),
        MAX_BYTES,
    )
    .await
    .map_err(AppError::internal)?;

    let file = files.pop().ok_or(AppError::NotFound)?;
    let (start_line, end_line, of_lines, lines) = narrow(&file.body, symbol.as_deref(), WINDOW);
    if lines.is_empty() {
        return Err(AppError::NotFound);
    }

    Ok(Json(ExcerptView {
        path: file.path,
        symbol,
        start_line,
        end_line,
        of_lines,
        lines,
        truncated: file.truncated,
    }))
}

fn cited(doc: &Value, key: &str, path: &str) -> Option<Option<String>> {
    doc.get("features")
        .and_then(Value::as_array)?
        .iter()
        .find(|feature| feature.get("key").and_then(Value::as_str) == Some(key))?
        .get("scenarios")
        .and_then(Value::as_array)?
        .iter()
        .find(|scenario| scenario.get("evidence").and_then(Value::as_str) == Some(path))
        .map(|scenario| {
            scenario
                .get("symbol")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

fn narrow(
    body: &str,
    symbol: Option<&str>,
    window: usize,
) -> (usize, usize, usize, Vec<String>) {
    let lines: Vec<&str> = body.lines().collect();
    let of_lines = lines.len();
    let at = symbol
        .and_then(|name| lines.iter().position(|line| line.contains(name)))
        .unwrap_or(0);
    let end = (at + window).min(of_lines);
    (
        at + 1,
        end,
        of_lines,
        lines[at..end].iter().map(|line| line.to_string()).collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc() -> Value {
        json!({ "features": [
            { "key": "card", "scenarios": [
                { "evidence": "billing/card.ts", "symbol": "registerCard" },
                { "evidence": "billing/save.ts", "symbol": null },
            ]},
        ]})
    }

    #[test]
    fn an_uncited_path_does_not_resolve() {
        assert!(cited(&doc(), "card", "billing/secrets.ts").is_none());
        assert!(cited(&doc(), "other", "billing/card.ts").is_none());
        assert_eq!(
            cited(&doc(), "card", "billing/card.ts"),
            Some(Some("registerCard".to_string()))
        );
        assert_eq!(cited(&doc(), "card", "billing/save.ts"), Some(None));
    }

    #[test]
    fn the_symbol_anchors_the_window() {
        let body = "a\nb\nfn registerCard() {\nc\nd\n";
        let (start, end, of_lines, lines) = narrow(body, Some("registerCard"), 2);
        assert_eq!((start, end, of_lines), (3, 4, 5));
        assert_eq!(lines, vec!["fn registerCard() {", "c"]);
    }

    #[test]
    fn a_symbol_the_file_does_not_hold_falls_back_to_the_head() {
        let body = "a\nb\nc\n";
        let (start, end, _, lines) = narrow(body, Some("elsewhere"), 2);
        assert_eq!((start, end), (1, 2));
        assert_eq!(lines, vec!["a", "b"]);
    }

    #[test]
    fn the_window_never_runs_past_the_end() {
        let (start, end, of_lines, lines) = narrow("only\n", None, 18);
        assert_eq!((start, end, of_lines), (1, 1, 1));
        assert_eq!(lines, vec!["only"]);
    }

    #[test]
    fn an_empty_file_yields_no_lines() {
        let (_, _, of_lines, lines) = narrow("", Some("x"), 18);
        assert_eq!(of_lines, 0);
        assert!(lines.is_empty());
    }
}
