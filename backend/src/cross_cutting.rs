//! Stage 2 (`cross_cutting`): extract the repository's cross-cutting concerns (AC1.2).
//!
//! AC1.2 names five axes — 인프라, 저장소 구조, 아키텍처, 프레임워크/런타임,
//! 미들웨어 — and requires that every extracted item carry the file path or symbol
//! it was inferred from. Those five are the contract this module encodes; the Cross-cutting Concerns
//! mockup draws four of them (it omits 저장소 구조), and the PRD is the SSOT, so the
//! extra axis is implemented and the difference is registered as a known
//! mockup↔implementation deviation rather than silently dropped.
//!
//! The input is the file-path list stage 1 measured. Paths alone are enough for the
//! evidence AC1.2 asks for and keep the stage from having to fetch blob contents;
//! the list is sorted and truncated deterministically so the same repository
//! produces the same prompt.

use serde_json::{json, Value};

use crate::llm::{self, Ask};

/// The five axes AC1.2 enumerates, in the order the PRD lists them.
pub const AXES: [(&str, &str); 5] = [
    ("infrastructure", "인프라 (배포 매니페스트 · IaC)"),
    ("repository_structure", "저장소 구조 (monorepo 여부 · 모듈 구분)"),
    ("architecture", "아키텍처 (레이어 구분 · DDD/MVC/헥사고날 등 패턴)"),
    ("framework", "프레임워크 · 런타임"),
    ("middleware", "미들웨어 (인증 · 로깅 · 캐시 등)"),
];

/// How many paths are handed to the model. A cap keeps the prompt bounded on large
/// repositories; taking the *first* N of a sorted list rather than a sample keeps
/// it deterministic.
const MAX_PATHS: usize = 400;

const SYSTEM: &str = "\
You analyze a source repository's file tree and extract its cross-cutting concerns.
Work only from the paths you are given: never invent a file that is not in the list.
Every item must cite at least one path from the list as its evidence.
If an axis has no supporting evidence in the tree, return an empty item list for it
rather than guessing.";

/// The JSON shape the answer is constrained to. Nothing but JSON Schema goes in
/// here — it is sent to the provider verbatim (see [`crate::llm::Ask`]).
fn schema() -> Value {
    let axis_keys: Vec<&str> = AXES.iter().map(|(key, _)| *key).collect();
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["categories"],
        "properties": {
            "categories": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["axis", "items"],
                    "properties": {
                        "axis": { "type": "string", "enum": axis_keys },
                        "items": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "additionalProperties": false,
                                "required": ["name", "evidence"],
                                "properties": {
                                    "name": { "type": "string" },
                                    "evidence": {
                                        "type": "array",
                                        "items": { "type": "string" },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        },
    })
}

/// Deterministic stand-in for the model's answer.
///
/// It is derived from the actual path list rather than hard-coded, so the e2e can
/// assert the real property AC1.2 asks for — every item cites a path that exists in
/// the analyzed tree — instead of asserting a fixed string.
fn stub_answer(paths: &[String]) -> Value {
    let categories: Vec<Value> = AXES
        .iter()
        .enumerate()
        .map(|(i, (key, label))| {
            // One item per axis, citing a path chosen by position so different
            // axes cite different files and the same repo always maps the same way.
            let items: Vec<Value> = paths
                .get(i % paths.len().max(1))
                .map(|path| {
                    vec![json!({
                        "name": format!("{label} (stub 추출)"),
                        "evidence": [path],
                    })]
                })
                .unwrap_or_default();
            json!({ "axis": key, "items": items })
        })
        .collect();
    json!({ "categories": categories })
}

fn prompt(owner: &str, name: &str, branch: &str, paths: &[String]) -> String {
    let listed = paths.join("\n");
    format!(
        "Repository: {owner}/{name}@{branch}\n\
         Files ({shown} of {total}):\n{listed}\n\n\
         Extract the cross-cutting concerns for each of these axes: {axes}.",
        shown = paths.len(),
        total = paths.len(),
        axes = AXES
            .iter()
            .map(|(key, _)| *key)
            .collect::<Vec<_>>()
            .join(", "),
    )
}

/// Sorted, truncated view of the tree — the stage's actual model input.
pub fn input_paths(paths: &[String]) -> Vec<String> {
    let mut sorted = paths.to_vec();
    sorted.sort();
    sorted.dedup();
    sorted.truncate(MAX_PATHS);
    sorted
}

/// Runs stage 2 and returns the document to persist.
pub async fn extract(
    http: &reqwest::Client,
    mode: crate::config::Mode,
    provider: llm::Provider,
    key: Option<&str>,
    owner: &str,
    name: &str,
    branch: &str,
    paths: &[String],
) -> Result<llm::Answer, String> {
    let paths = input_paths(paths);
    if paths.is_empty() {
        return Err("repository tree is empty; nothing to analyze".to_string());
    }
    llm::ask(
        http,
        mode,
        provider,
        key,
        Ask {
            system: SYSTEM,
            user: prompt(owner, name, branch, &paths),
            schema: schema(),
            stub: stub_answer(&paths),
        },
    )
    .await
}

/// The one-liner Analysis Progress renders under the step, mirroring stage 1's "766 files · 2.2 MB".
pub fn detail(doc: &Value) -> String {
    let items: usize = doc
        .get("categories")
        .and_then(Value::as_array)
        .map(|cats| {
            cats.iter()
                .filter_map(|c| c.get("items").and_then(Value::as_array))
                .map(Vec::len)
                .sum()
        })
        .unwrap_or(0);
    format!("{} categories · {} items", AXES.len(), items)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Vec<String> {
        vec![
            "src/main.rs".to_string(),
            "deploy/base/deployment.yaml".to_string(),
            "Cargo.toml".to_string(),
            "src/middleware/auth.rs".to_string(),
            "src/main.rs".to_string(), // duplicate — must be collapsed
        ]
    }

    #[test]
    fn input_paths_is_sorted_deduped_and_stable() {
        let a = input_paths(&tree());
        let b = input_paths(&tree());
        assert_eq!(a, b);
        assert_eq!(a.len(), 4, "the duplicate path should collapse");
        let mut sorted = a.clone();
        sorted.sort();
        assert_eq!(a, sorted);
    }

    /// AC1.2's verification method: every item carries the path it was inferred
    /// from, and that path is one the analysis actually saw.
    #[tokio::test]
    async fn every_extracted_item_cites_a_path_from_the_tree() {
        let http = reqwest::Client::new();
        let paths = tree();
        let answer = extract(
            &http,
            crate::config::Mode::Stub,
            llm::Provider::Anthropic,
            None,
            "acme",
            "widgets",
            "main",
            &paths,
        )
        .await
        .unwrap();

        let known = input_paths(&paths);
        let cats = answer.content["categories"].as_array().unwrap();
        assert_eq!(cats.len(), AXES.len(), "one entry per AC1.2 axis");
        let mut items_seen = 0;
        for cat in cats {
            for item in cat["items"].as_array().unwrap() {
                items_seen += 1;
                let evidence = item["evidence"].as_array().unwrap();
                assert!(!evidence.is_empty(), "item has no evidence");
                for e in evidence {
                    let path = e.as_str().unwrap().to_string();
                    assert!(known.contains(&path), "cited a path not in the tree: {path}");
                }
            }
        }
        assert!(items_seen > 0);
    }

    /// The determinism clause: the same tree must reproduce exactly.
    #[tokio::test]
    async fn the_same_tree_reproduces_the_same_document() {
        let http = reqwest::Client::new();
        let paths = tree();
        let run = || {
            extract(
                &http,
                crate::config::Mode::Stub,
                llm::Provider::Anthropic,
                None,
                "acme",
                "widgets",
                "main",
                &paths,
            )
        };
        assert_eq!(run().await.unwrap().content, run().await.unwrap().content);
    }

    #[tokio::test]
    async fn an_empty_tree_fails_rather_than_inventing_a_document() {
        let http = reqwest::Client::new();
        let err = extract(
            &http,
            crate::config::Mode::Stub,
            llm::Provider::Anthropic,
            None,
            "acme",
            "widgets",
            "main",
            &[],
        )
        .await
        .unwrap_err();
        assert!(err.contains("empty"), "{err}");
    }

    /// The schema travels to the provider verbatim, so the stub answer must not
    /// be inside it — OpenAI rejects unknown keywords under `strict`.
    #[test]
    fn the_schema_is_only_a_schema() {
        let s = schema();
        assert!(s.get("stub_answer").is_none());
        assert_eq!(s["additionalProperties"], false);
        assert_eq!(s["required"][0], "categories");
    }

    #[test]
    fn detail_reads_like_an_s04_step() {
        let doc = json!({ "categories": [
            { "axis": "infrastructure", "items": [{ "name": "x", "evidence": ["a"] }] },
            { "axis": "framework", "items": [] },
        ]});
        assert_eq!(detail(&doc), "5 categories · 1 items");
    }
}
