//! Stage 3 (`discovery_strategy`): where should feature discovery look? (AC1.3)
//!
//! AC1.3 takes the cross-cutting document stage 2 produced and asks for a strategy
//! a person can read and edit — "진입점 파일 패턴, 라우팅 정의 위치, UI 컴포넌트
//! 경로, 외부에 노출된 API 엔드포인트, CLI 명령 등". So the unit here is a
//! **pattern**, not a prose paragraph: something the next stage can scan by and a
//! reviewer can delete in one gesture.
//!
//! Two inputs, both already earned by earlier stages: the sorted, truncated path
//! list stage 1 measured (via [`crate::cross_cutting::input_paths`], the same view,
//! so the two stages cannot disagree about what the repository contains) and the
//! cross-cutting document stage 2 stored. Reusing stage 1's list rather than
//! re-deriving one is what keeps the prompt deterministic for a given tree.
//!
//! The *editing and approval* half of AC1.3 does not live here — it is
//! `discovery_strategies` (migration 0006) and the routes in [`crate::analysis`].
//! This module only produces the proposal.

use serde_json::{json, Value};

use crate::llm::{self, Ask};

/// The kinds of entry point AC1.3 enumerates. Sent to the model as the closed set
/// it must classify into, so an entry's `kind` is never free-form text the screen
/// would have to guess how to render.
pub const KINDS: [&str; 5] = ["route", "ui", "api", "job", "cli"];

/// Upper bound on the proposal. AC1.3's output is something a person reviews on a
/// phone screen; a list longer than this is not a strategy, it is a file listing.
const MAX_ENTRIES: usize = 12;

const SYSTEM: &str = "\
You plan where end-user features can be discovered in a source repository.
Work only from the file paths and the cross-cutting summary you are given.
Each entry is a glob or path pattern that a later scan will follow, plus a short
rationale and at least one path from the list as evidence that the pattern matches
something real. Never propose a pattern that matches no path in the list.
Prefer a few broad entry points over many narrow ones.";

/// The JSON shape the answer is constrained to (sent to the provider verbatim).
fn schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["entries"],
        "properties": {
            "entries": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["pattern", "kind", "rationale", "evidence"],
                    "properties": {
                        "pattern": { "type": "string" },
                        "kind": { "type": "string", "enum": KINDS },
                        "rationale": { "type": "string" },
                        "evidence": { "type": "array", "items": { "type": "string" } },
                    },
                },
            },
        },
    })
}

/// Deterministic stand-in for the model's answer.
///
/// Derived from the actual tree rather than hard-coded, for the same reason stage
/// 2's stub is: the e2e can then assert AC1.3's real property — every proposed
/// pattern matches a path the analysis actually saw — instead of a fixed string.
///
/// The pattern is the directory prefix of the chosen path plus `/**`, which is a
/// pattern the path genuinely matches. A path with no directory part yields the
/// file itself.
fn stub_answer(paths: &[String]) -> Value {
    let mut entries = Vec::new();
    for (i, kind) in KINDS.iter().enumerate() {
        let Some(path) = paths.get(i * paths.len() / KINDS.len().max(1)) else {
            continue;
        };
        let pattern = match path.rsplit_once('/') {
            Some((dir, _)) => format!("{dir}/**"),
            None => path.clone(),
        };
        entries.push(json!({
            "pattern": pattern,
            "kind": kind,
            "rationale": format!("{kind} entry points (stub 제안)"),
            "evidence": [path],
        }));
    }
    json!({ "entries": entries })
}

/// A compact rendering of stage 2's document — the axes and item names, without the
/// evidence lists. The model needs to know *what kind of codebase this is*; feeding
/// it the whole document back would mostly re-send paths it already has.
fn landscape(cross_cutting: &Value) -> String {
    let mut lines = Vec::new();
    for category in cross_cutting
        .get("categories")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let axis = category.get("axis").and_then(Value::as_str).unwrap_or("?");
        let names: Vec<&str> = category
            .get("items")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .filter_map(|item| item.get("name").and_then(Value::as_str))
            .collect();
        lines.push(format!("- {axis}: {}", names.join(", ")));
    }
    lines.join("\n")
}

fn prompt(owner: &str, name: &str, branch: &str, paths: &[String], cross_cutting: &Value) -> String {
    format!(
        "Repository: {owner}/{name}@{branch}\n\
         Cross-cutting concerns already extracted:\n{landscape}\n\n\
         Files ({shown}):\n{listed}\n\n\
         Propose at most {max} discovery entry points, one of these kinds: {kinds}.",
        landscape = landscape(cross_cutting),
        shown = paths.len(),
        listed = paths.join("\n"),
        max = MAX_ENTRIES,
        kinds = KINDS.join(", "),
    )
}

/// Runs stage 3 and returns the document to persist.
pub async fn propose(
    http: &reqwest::Client,
    mode: crate::config::Mode,
    provider: llm::Provider,
    key: Option<&str>,
    owner: &str,
    name: &str,
    branch: &str,
    paths: &[String],
    cross_cutting: &Value,
) -> Result<llm::Answer, String> {
    let paths = crate::cross_cutting::input_paths(paths);
    if paths.is_empty() {
        return Err("repository tree is empty; nothing to plan a search over".to_string());
    }
    let mut answer = llm::ask(
        http,
        mode,
        provider,
        key,
        Ask {
            system: SYSTEM,
            user: prompt(owner, name, branch, &paths, cross_cutting),
            schema: schema(),
            stub: stub_answer(&paths),
        },
    )
    .await?;

    // Cap here rather than trusting the instruction: a model that ignores "at most
    // N" would otherwise hand the reviewer an unreviewable list, and AC1.3's whole
    // point is that a person reviews this.
    if let Some(entries) = answer.content.get_mut("entries").and_then(Value::as_array_mut) {
        entries.truncate(MAX_ENTRIES);
        if entries.is_empty() {
            return Err("the model proposed no discovery entry points".to_string());
        }
    } else {
        return Err("the strategy document has no entries".to_string());
    }
    Ok(answer)
}

/// The patterns, in document order — what seeds the reviewable strategy.
pub fn patterns(doc: &Value) -> Vec<String> {
    doc.get("entries")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|e| e.get("pattern").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
}

/// The one-liner Analysis Progress renders under the step, mirroring stage 2's "5 categories · 3 items".
pub fn detail(doc: &Value) -> String {
    format!("{} entry points · awaiting approval", patterns(doc).len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Vec<String> {
        vec![
            "src/routes/auth.ts".to_string(),
            "src/pages/Home.tsx".to_string(),
            "openapi.yaml".to_string(),
            "src/jobs/mail.worker.ts".to_string(),
            "cmd/admin/main.go".to_string(),
        ]
    }

    fn landscape_doc() -> Value {
        json!({ "categories": [
            { "axis": "framework", "items": [{ "name": "Express 4", "evidence": ["package.json"] }] },
        ]})
    }

    async fn run() -> llm::Answer {
        propose(
            &reqwest::Client::new(),
            crate::config::Mode::Stub,
            llm::Provider::OpenAI,
            None,
            "acme",
            "widgets",
            "main",
            &tree(),
            &landscape_doc(),
        )
        .await
        .unwrap()
    }

    /// AC1.3's proposal is only reviewable if it points at things that exist: every
    /// entry cites a path the analysis actually saw.
    #[tokio::test]
    async fn every_entry_cites_a_path_from_the_tree() {
        let known = crate::cross_cutting::input_paths(&tree());
        let answer = run().await;
        let entries = answer.content["entries"].as_array().unwrap();
        assert!(!entries.is_empty());
        for entry in entries {
            assert!(KINDS.contains(&entry["kind"].as_str().unwrap()));
            assert!(!entry["pattern"].as_str().unwrap().is_empty());
            let evidence = entry["evidence"].as_array().unwrap();
            assert!(!evidence.is_empty(), "entry has no evidence");
            for e in evidence {
                let path = e.as_str().unwrap().to_string();
                assert!(known.contains(&path), "cited a path not in the tree: {path}");
            }
        }
    }

    /// The same tree and the same landscape must reproduce exactly — the stage
    /// inherits AC1.2's determinism posture because its input is the same fixed list.
    #[tokio::test]
    async fn the_same_input_reproduces_the_same_document() {
        assert_eq!(run().await.content, run().await.content);
    }

    #[tokio::test]
    async fn an_empty_tree_fails_rather_than_inventing_a_strategy() {
        let err = propose(
            &reqwest::Client::new(),
            crate::config::Mode::Stub,
            llm::Provider::OpenAI,
            None,
            "acme",
            "widgets",
            "main",
            &[],
            &landscape_doc(),
        )
        .await
        .unwrap_err();
        assert!(err.contains("empty"), "{err}");
    }

    /// The reviewer, not the model, decides how long the list is allowed to be.
    #[test]
    fn the_proposal_is_capped_for_a_human_reviewer() {
        assert!(MAX_ENTRIES <= 12);
    }

    #[test]
    fn the_schema_is_only_a_schema() {
        let s = schema();
        assert!(s.get("stub_answer").is_none());
        assert_eq!(s["additionalProperties"], false);
        assert_eq!(s["required"][0], "entries");
    }

    #[test]
    fn patterns_and_detail_read_off_the_document() {
        let doc = json!({ "entries": [
            { "pattern": "src/routes/**", "kind": "route", "rationale": "r", "evidence": ["a"] },
            { "pattern": "src/pages/**", "kind": "ui", "rationale": "r", "evidence": ["b"] },
        ]});
        assert_eq!(patterns(&doc), vec!["src/routes/**", "src/pages/**"]);
        assert_eq!(detail(&doc), "2 entry points · awaiting approval");
    }

    #[test]
    fn the_landscape_summary_carries_axis_names_without_evidence() {
        let text = landscape(&landscape_doc());
        assert!(text.contains("framework"));
        assert!(text.contains("Express 4"));
        assert!(!text.contains("package.json"), "evidence should not be re-sent");
    }
}
