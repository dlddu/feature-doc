//! Stage 4 (`feature_candidates`): what end-user features live in this codebase? (AC1.4)
//!
//! AC1.4 takes the strategy the reviewer approved in stage 3 and asks for a list a
//! person can sift: "각 후보에는 발견된 위치(파일·심볼)와 추정 근거가 함께 기록된다".
//! So a candidate is a name plus **where it was found** plus **why** — three fields
//! the screen renders side by side, and the reason the location is not optional.
//!
//! Two inputs, both already earned: the approved patterns (from
//! `discovery_strategies`, handed over by the claim response so this stage needs no
//! second round-trip) and the path list stage 1 measured, viewed through
//! [`crate::cross_cutting::input_paths`] — the same view stages 2 and 3 use, so the
//! four stages cannot disagree about what the repository contains.
//!
//! The *deciding* half of AC1.4 does not live here — it is `feature_candidates`
//! (migration 0007) and the routes in [`crate::analysis`]. This module only
//! produces the proposal, exactly as [`crate::discovery_strategy`] does for stage 3.
//!
//! One thing this module owns for both halves: [`candidate_key`]. A candidate's
//! identity across analyses is *where it was found*, not a row id — the next
//! analysis of the same repository writes new rows for the same places, and
//! 시나리오 7 ("재분석 시 이전에 거부된 항목으로 표시") is only expressible if both
//! sides derive that identity the same way.

use serde_json::{json, Value};

use crate::llm::{self, Ask};

/// Upper bound on one extraction. AC1.4's output is a list a person decides on one
/// item at a time on a phone; past this it stops being reviewable and becomes a
/// second file listing (same reasoning as stage 3's cap, one order larger because
/// features are finer-grained than entry points).
const MAX_CANDIDATES: usize = 40;

const SYSTEM: &str = "\
You extract end-user facing features from a source repository.
Work only from the file paths and the approved discovery patterns you are given.
A feature is something a person using the product can do — not a helper, not a
build step, not an internal utility.
Each candidate must name one path from the list as the location it was found at,
and say in one sentence why that code looks like an end-user feature.
Never propose a candidate whose location is not in the list.";

/// The JSON shape the answer is constrained to (sent to the provider verbatim).
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

/// A candidate's identity across analyses: `location` plus `symbol` when there is
/// one.
///
/// Not a row id and not the name. The name is the first thing the reviewer renames
/// (AC1.4 grants exactly that), so keying on it would make a renamed candidate a
/// stranger to the next analysis; the location is what the *extractor* found and is
/// stable as long as the code is.
pub fn candidate_key(location: &str, symbol: Option<&str>) -> String {
    match symbol.map(str::trim).filter(|s| !s.is_empty()) {
        Some(symbol) => format!("{location}#{symbol}"),
        None => location.to_string(),
    }
}

/// Deterministic stand-in for the model's answer.
///
/// Derived from the actual tree and the actual approved patterns rather than
/// hard-coded, for the same reason stages 2 and 3 do it: the e2e can then assert
/// AC1.4's real property — every candidate cites a path this analysis actually saw,
/// reached through a pattern the reviewer actually approved — instead of a fixed
/// string that would still pass if the wiring were cut.
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

/// Paths a glob-ish discovery pattern selects.
///
/// Deliberately small: `**` matches any run of characters, `*` matches within one
/// path segment, and a pattern with no wildcard matches any path that starts with
/// it (that is how stage 3's `cmd/admin-cli`-style hand-added entry points behave).
/// Enough to make the stub honest and to check the model's answers; the real scan
/// is the model's job.
pub fn matching<'a>(paths: &'a [String], pattern: &str) -> Vec<&'a String> {
    paths.iter().filter(|p| matches_pattern(p, pattern)).collect()
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    if !pattern.contains('*') {
        return path == pattern || path.starts_with(&format!("{}/", pattern.trim_end_matches('/')));
    }
    glob_match(path.as_bytes(), pattern.as_bytes())
}

/// `**` spans separators, `*` does not. Recursive rather than a compiled matcher:
/// patterns are short and there are at most [`MAX_CANDIDATES`] of them per run.
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

/// Runs stage 4 and returns the document to persist.
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
            stub: stub_answer(&paths, patterns),
        },
    )
    .await?;

    // Drop candidates whose location is not a path this analysis saw, then cap.
    // Both are checks on the *answer* rather than trust in the instruction: a
    // fabricated location is exactly the failure AC1.4's "발견된 위치" clause exists
    // to prevent, and a list past the cap is not reviewable.
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

/// The candidates, in document order — what seeds the reviewable list.
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

/// One extracted candidate as the review half stores it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub key: String,
    pub name: String,
    pub location: String,
    pub symbol: Option<String>,
    pub rationale: String,
}

/// The one-liner S04 renders under the step, mirroring stages 2 and 3.
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
        // A single `*` does not cross a separator, so this matches nothing.
        assert!(matching(&paths, "src/*.ts").is_empty());
        // A wildcard-free pattern is a prefix, which is how a hand-added entry
        // point like `cmd/admin-cli` reaches its directory.
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
        // Renaming is an action AC1.4 grants the reviewer, so the key must survive
        // it — otherwise a renamed candidate is a stranger to the next analysis.
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
