//! Stage 4 (`feature_candidates`): proposes the end-user features of a codebase.
//!
//! The approved patterns arrive on the claim response, so this stage needs no second
//! round-trip. Paths are read through [`crate::cross_cutting::input_paths`], the same
//! view stages 2 and 3 use, so the four stages cannot disagree about what the
//! repository contains.

use serde_json::{json, Value};

use crate::llm::{self, Ask};

const MAX_CANDIDATES: usize = 40;

const SYSTEM: &str = "\
You extract end-user facing features from a source repository.
Work only from the file paths and the approved discovery patterns you are given.
A feature is something a person using the product can do — not a helper, not a
build step, not an internal utility.
Each candidate must name one path from the list as the location it was found at,
and say in one sentence why that code looks like an end-user feature.
Never propose a candidate whose location is not in the list.";

fn schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["candidates"],
        "properties": {
            "candidates": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["name", "location", "rationale"],
                    "properties": {
                        "name": { "type": "string" },
                        "location": { "type": "string" },
                        "symbol": { "type": "string" },
                        "rationale": { "type": "string" },
                    },
                },
            },
        },
    })
}

/// A candidate's identity across analyses — deliberately not a row id and not the
/// name. The next analysis writes new rows for the same places, and the reviewer may
/// rename; only the location survives both, so both sides must key on it.
pub fn candidate_key(location: &str, symbol: Option<&str>) -> String {
    match symbol.map(str::trim).filter(|s| !s.is_empty()) {
        Some(symbol) => format!("{location}#{symbol}"),
        None => location.to_string(),
    }
}

/// Derived from the tree and the approved patterns rather than hard-coded so the
/// e2e can assert the real property — every candidate cites a path this analysis
/// saw, reached through a pattern the reviewer approved — instead of a fixed string
/// that would still pass with the wiring cut.
fn stub_answer(paths: &[String], patterns: &[String]) -> Value {
    let mut candidates = Vec::new();
    for pattern in patterns {
        for path in matching(paths, pattern).into_iter().take(2) {
            let stem = path
                .rsplit_once('/')
                .map(|(_, file)| file)
                .unwrap_or(path.as_str());
            let name = stem.split('.').next().unwrap_or(stem);
            candidates.push(json!({
                "name": name,
                "location": path,
                "rationale": format!("`{pattern}` 패턴이 가리키는 진입점 (stub 후보)"),
            }));
        }
    }
    json!({ "candidates": candidates })
}

/// The pattern dialect is deliberately small: `**` matches any run of characters,
/// `*` matches within one path segment, and a wildcard-free pattern is a prefix —
/// which is how a hand-added entry point like `cmd/admin-cli` reaches its directory.
pub fn matching<'a>(paths: &'a [String], pattern: &str) -> Vec<&'a String> {
    paths.iter().filter(|p| matches_pattern(p, pattern)).collect()
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    if !pattern.contains('*') {
        return path == pattern || path.starts_with(&format!("{}/", pattern.trim_end_matches('/')));
    }
    glob_match(path.as_bytes(), pattern.as_bytes())
}

fn glob_match(path: &[u8], pattern: &[u8]) -> bool {
    if pattern.is_empty() {
        return path.is_empty();
    }
    if pattern.starts_with(b"**") {
        let rest = &pattern[2..];
        let rest = rest.strip_prefix(b"/").unwrap_or(rest);
        if rest.is_empty() {
            return true;
        }
        for i in 0..=path.len() {
            if glob_match(&path[i..], rest) {
                return true;
            }
        }
        return false;
    }
    if pattern[0] == b'*' {
        for i in 0..=path.len() {
            if path[..i].contains(&b'/') {
                break;
            }
            if glob_match(&path[i..], &pattern[1..]) {
                return true;
            }
        }
        return false;
    }
    if path.is_empty() || path[0] != pattern[0] {
        return false;
    }
    glob_match(&path[1..], &pattern[1..])
}

fn prompt(owner: &str, name: &str, branch: &str, paths: &[String], patterns: &[String]) -> String {
    format!(
        "Repository: {owner}/{name}@{branch}\n\
         Approved discovery patterns ({npat}):\n{listed_patterns}\n\n\
         Files ({shown}):\n{listed}\n\n\
         Extract at most {max} end-user feature candidates.",
        npat = patterns.len(),
        listed_patterns = patterns.join("\n"),
        shown = paths.len(),
        listed = paths.join("\n"),
        max = MAX_CANDIDATES,
    )
}

pub async fn extract(
    http: &reqwest::Client,
    mode: crate::config::Mode,
    provider: llm::Provider,
    key: Option<&str>,
    owner: &str,
    name: &str,
    branch: &str,
    paths: &[String],
    patterns: &[String],
) -> Result<llm::Answer, String> {
    let paths = crate::cross_cutting::input_paths(paths);
    if paths.is_empty() {
        return Err("repository tree is empty; nothing to extract features from".to_string());
    }
    if patterns.is_empty() {
        return Err("no approved discovery pattern; approve a strategy first".to_string());
    }

    let mut answer = llm::ask(
        http,
        mode,
        provider,
        key,
        Ask {
            system: SYSTEM,
            user: prompt(owner, name, branch, &paths, patterns),
            schema: schema(),
            // mock-exception: LLM-01 — 실 LLM 산출물에 대한 결정적 단정을 위해 고정 답을 공급
            stub: stub_answer(&paths, patterns),
        },
    )
    .await?;

    // Checks on the *answer* rather than trust in the instruction: a model will
    // cite a location it invented, and it will run past "at most N".
    let Some(items) = answer
        .content
        .get_mut("candidates")
        .and_then(Value::as_array_mut)
    else {
        return Err("the candidate document has no candidates".to_string());
    };
    items.retain(|c| {
        c.get("location")
            .and_then(Value::as_str)
            .is_some_and(|loc| paths.iter().any(|p| p == loc))
    });
    items.truncate(MAX_CANDIDATES);
    if items.is_empty() {
        return Err("no feature candidate cited a path from this repository".to_string());
    }
    Ok(answer)
}

pub fn candidates(doc: &Value) -> Vec<Candidate> {
    doc.get("candidates")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|c| {
            let location = c.get("location").and_then(Value::as_str)?;
            let symbol = c
                .get("symbol")
                .and_then(Value::as_str)
                .map(str::to_string)
                .filter(|s| !s.trim().is_empty());
            Some(Candidate {
                key: candidate_key(location, symbol.as_deref()),
                name: c
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or(location)
                    .to_string(),
                location: location.to_string(),
                symbol,
                rationale: c
                    .get("rationale")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub key: String,
    pub name: String,
    pub location: String,
    pub symbol: Option<String>,
    pub rationale: String,
}

/// The one-liner rendered under the step on Analysis Progress.
pub fn detail(doc: &Value) -> String {
    format!("{} candidates · awaiting review", candidates(doc).len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Vec<String> {
        vec![
            "src/routes/auth.ts".to_string(),
            "src/routes/billing.ts".to_string(),
            "src/jobs/weekly.worker.ts".to_string(),
            "README.md".to_string(),
        ]
    }

    #[test]
    fn a_pattern_selects_only_paths_it_actually_matches() {
        let paths = tree();
        assert_eq!(
            matching(&paths, "src/routes/**"),
            vec![&paths[0], &paths[1]]
        );
        assert_eq!(matching(&paths, "src/jobs/*.worker.ts"), vec![&paths[2]]);
        assert!(matching(&paths, "src/*.ts").is_empty());
        assert_eq!(matching(&paths, "src/jobs"), vec![&paths[2]]);
    }

    #[test]
    fn every_stub_candidate_cites_a_path_the_analysis_saw() {
        let paths = tree();
        let doc = stub_answer(&paths, &["src/routes/**".to_string()]);
        let found = candidates(&doc);
        assert!(!found.is_empty());
        for candidate in &found {
            assert!(
                paths.contains(&candidate.location),
                "candidate cites a path outside the tree: {}",
                candidate.location
            );
        }
    }

    #[test]
    fn the_same_tree_and_patterns_produce_the_same_candidates() {
        let paths = tree();
        let patterns = vec!["src/routes/**".to_string(), "src/jobs/*.worker.ts".to_string()];
        assert_eq!(
            candidates(&stub_answer(&paths, &patterns)),
            candidates(&stub_answer(&paths, &patterns))
        );
    }

    #[test]
    fn identity_is_the_location_not_the_name() {
        assert_eq!(
            candidate_key("src/routes/auth.ts", None),
            candidate_key("src/routes/auth.ts", Some("   "))
        );
        assert_ne!(
            candidate_key("src/routes/auth.ts", Some("resetPassword")),
            candidate_key("src/routes/auth.ts", None)
        );
    }
}
