//! Stage 5: acceptance scenarios for a confirmed feature.
//!
//! Two calls rather than one, because "보강" is only observable against a *before*:
//! a single call would produce a list with nothing to compare it to.
//!
//! **The contradiction rule is code, not judgment** ([`merge`]) — a rule the model
//! cannot bend, and one that behaves the same on a real answer as on the stub.
//!
//! The wire key is `acceptance_dependencies`, but this stage does no dependency
//! work; the key predates the split and stayed for compatibility.

use serde_json::{json, Value};

use crate::llm::{self, Ask};

/// How many confirmed features one run writes documents for. The cap is about the
/// run's cost, not the screen's length — the reviewer opens these one at a time.
const MAX_FEATURES: usize = 12;

/// Scenarios kept per feature, after 보강. Past this the document stops being
/// something a non-developer reads end to end.
const MAX_SCENARIOS: usize = 12;

const SYSTEM_LOGIC: &str = "\
You write acceptance criteria for one end-user feature by reading the logic that
implements it.
Work only from the file paths you are given.
Write each criterion as given / when / then, in the words of a person using the
product — never in developer vocabulary, never as an HTTP route or a function call.
Every criterion must name one path from the list as the evidence it came from.
Never cite a path that is not in the list.";

const SYSTEM_TESTS: &str = "\
You strengthen existing acceptance criteria for one end-user feature by reading its
test code.
Work only from the test file paths you are given.
Add the cases the logic pass missed — error paths, boundaries, refusals — as
given / when / then, in the words of a person using the product.
If a test says the feature ends differently than the logic pass claimed, repeat that
situation with the ending the test describes; do not silently overwrite it.
Every criterion must name one path from the list as the evidence it came from.
Never cite a path that is not in the list.";

/// The JSON shape both answers are constrained to (sent to the provider verbatim).
fn schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["features"],
        "properties": {
            "features": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["key", "scenarios"],
                    "properties": {
                        "key": { "type": "string" },
                        "scenarios": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "additionalProperties": false,
                                "required": ["given", "when", "then", "evidence", "symbol"],
                                "properties": {
                                    "given": { "type": "string" },
                                    "when": { "type": "string" },
                                    "then": { "type": "string" },
                                    "evidence": { "type": "string" },
                                    "symbol": { "type": ["string", "null"] },
                                },
                            },
                        },
                    },
                },
            },
        },
    })
}

/// The feature this stage writes a document for — one approved candidate, as the
/// claim handed it over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subject {
    pub key: String,
    pub name: String,
    pub location: String,
    pub symbol: Option<String>,
}

/// Whether a repository path is test code.
///
/// Deliberately conventional rather than clever. This is the one place that answers
/// it, so the prompt, the answer validation and the e2e cannot disagree about it.
pub fn is_test_path(path: &str) -> bool {
    let (dirs, file) = match path.rsplit_once('/') {
        Some((dirs, file)) => (dirs, file),
        None => ("", path),
    };
    if dirs
        .split('/')
        .any(|seg| matches!(seg, "test" | "tests" | "spec" | "specs" | "__tests__"))
    {
        return true;
    }
    let stem = file.split('.').next().unwrap_or(file);
    file.contains(".test.")
        || file.contains(".spec.")
        || stem.ends_with("_test")
        || stem.ends_with("-test")
        || stem.starts_with("test_")
}

/// One acceptance criterion, flattened out of the answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub given: String,
    pub when: String,
    pub then: String,
    pub evidence: String,
    pub symbol: Option<String>,
}

/// Whitespace-insensitive identity of the *situation* a scenario describes.
fn situation(s: &Scenario) -> (String, String) {
    (norm(&s.given), norm(&s.when))
}

fn norm(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Reads the scenarios a single answer proposed for one feature key, dropping any
/// whose evidence is not a path this analysis actually saw.
///
/// The check is on the *answer*, not on trust in the instruction — the same guard
/// stage 4 puts on candidate locations.
fn scenarios_for(doc: &Value, key: &str, allowed: &[String]) -> Vec<Scenario> {
    doc.get("features")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter(|f| f.get("key").and_then(Value::as_str) == Some(key))
        .flat_map(|f| {
            f.get("scenarios")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default()
        })
        .filter_map(|s| {
            let evidence = s.get("evidence").and_then(Value::as_str)?;
            if !allowed.iter().any(|p| p == evidence) {
                return None;
            }
            Some(Scenario {
                given: s.get("given").and_then(Value::as_str)?.to_string(),
                when: s.get("when").and_then(Value::as_str)?.to_string(),
                then: s.get("then").and_then(Value::as_str)?.to_string(),
                evidence: evidence.to_string(),
                symbol: s
                    .get("symbol")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .filter(|s| !s.trim().is_empty()),
            })
        })
        .collect()
}

fn scenario_json(s: &Scenario, source: &str) -> Value {
    json!({
        "given": s.given,
        "when": s.when,
        "then": s.then,
        "evidence": s.evidence,
        "symbol": s.symbol,
        "source": source,
    })
}

/// Folds the two passes into one document per feature.
///
/// The logic scenario always stays, even when contested. It is what the code says,
/// and dropping it would leave the contested situation described nowhere in the
/// document a non-developer reads; deciding which side is right is the reviewer's job.
fn merge(subjects: &[Subject], logic: &Value, tests: &Value, allowed_tests: &[String], allowed: &[String]) -> Value {
    let mut features = Vec::new();
    for subject in subjects.iter().take(MAX_FEATURES) {
        let from_logic = scenarios_for(logic, &subject.key, allowed);
        let from_tests = scenarios_for(tests, &subject.key, allowed_tests);

        let mut scenarios: Vec<Value> = from_logic
            .iter()
            .map(|s| scenario_json(s, SOURCE_LOGIC))
            .collect();
        let mut contradictions = Vec::new();
        for candidate in &from_tests {
            match from_logic
                .iter()
                .find(|l| situation(l) == situation(candidate))
            {
                Some(conflicting) if norm(&conflicting.then) != norm(&candidate.then) => {
                    contradictions.push(json!({
                        "given": conflicting.given,
                        "when": conflicting.when,
                        "codeSays": conflicting.then,
                        "codeEvidence": conflicting.evidence,
                        "testSays": candidate.then,
                        "testEvidence": candidate.evidence,
                    }));
                }
                // The same situation with the same ending is the test confirming the
                // logic pass, not a second scenario to read.
                Some(_) => {}
                None => scenarios.push(scenario_json(candidate, SOURCE_TEST)),
            }
        }
        scenarios.truncate(MAX_SCENARIOS);

        if scenarios.is_empty() {
            continue;
        }
        features.push(json!({
            "key": subject.key,
            "name": subject.name,
            "location": subject.location,
            "symbol": subject.symbol,
            "scenarios": scenarios,
            "contradictions": contradictions,
        }));
    }
    json!({ "features": features })
}

/// Where a scenario came from. Kept on every scenario rather than as two lists: the
/// document is read as one narrative, and "which pass found this" is an attribute of
/// the sentence, not a section.
pub const SOURCE_LOGIC: &str = "logic";
pub const SOURCE_TEST: &str = "test";

fn subject_line(subject: &Subject) -> String {
    match subject.symbol.as_deref() {
        Some(symbol) => format!("- {} — {} · {}", subject.key, subject.location, symbol),
        None => format!("- {} — {}", subject.key, subject.location),
    }
}

fn prompt(
    owner: &str,
    name: &str,
    branch: &str,
    subjects: &[Subject],
    paths: &[String],
    what: &str,
) -> String {
    let listed_subjects = subjects
        .iter()
        .take(MAX_FEATURES)
        .map(subject_line)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Repository: {owner}/{name}@{branch}\n\
         Confirmed features ({nfeat}), given as `key — location`:\n{listed_subjects}\n\n\
         {what} ({shown}):\n{listed}\n\n\
         Answer with one entry per feature key above, at most {max} scenarios each.",
        nfeat = subjects.len().min(MAX_FEATURES),
        shown = paths.len(),
        listed = paths.join("\n"),
        max = MAX_SCENARIOS,
    )
}

/// Deterministic stand-in for the logic pass.
///
/// Derived from the approved feature and the tree it was found in, never a fixed
/// string — the same reason stages 2-4 do it. A constant would still match if the
/// wiring were cut, so the e2e would keep passing over a dead stage.
fn stub_logic(subjects: &[Subject], paths: &[String]) -> Value {
    let rewritten = paths
        .iter()
        .any(|p| p.ends_with(crate::repo_scan::REWRITE_MARKER));
    let first_then = if rewritten {
        "요청한 대로 처리되고, 환불 규정이 바뀐 뒤에는 그 규정이 결과와 함께 표시됩니다"
    } else {
        "요청한 대로 처리되고 결과를 화면에서 바로 확인할 수 있습니다"
    };
    let features: Vec<Value> = subjects
        .iter()
        .take(MAX_FEATURES)
        .map(|subject| {
            let mut scenarios = vec![json!({
                "given": format!("{} 을(를) 아직 한 번도 쓰지 않은 사용자가", subject.name),
                "when": format!("{} 을(를) 처음 실행하면", subject.name),
                "then": first_then,
                "evidence": subject.location,
                "symbol": subject.symbol,
            })];
            // A second criterion only when the tree offers a second place to cite —
            // an invented evidence path would be the exact failure this stage guards.
            if let Some(other) = paths.iter().find(|p| **p != subject.location) {
                scenarios.push(json!({
                    "given": format!("{} 을(를) 이미 쓰고 있는 사용자가", subject.name),
                    "when": "같은 요청을 한 번 더 보내면",
                    "then": "같은 결과가 다시 만들어지지 않고 이미 있는 결과를 그대로 보여 줍니다",
                    "evidence": other,
                }));
            }
            json!({ "key": subject.key, "scenarios": scenarios })
        })
        .collect();
    json!({ "features": features })
}

/// Deterministic stand-in for the test pass.
///
/// Two shapes, both cited from an actual test file: one criterion the logic pass did
/// not have, and one that repeats the *first* logic situation with a different ending
/// so that the separation is observable end to end. A repository without test files
/// gets no test pass at all rather than an empty one.
fn stub_tests(subjects: &[Subject], test_paths: &[String]) -> Value {
    let Some(first_test) = test_paths.first() else {
        return json!({ "features": [] });
    };
    let features: Vec<Value> = subjects
        .iter()
        .take(MAX_FEATURES)
        .map(|subject| {
            json!({
                "key": subject.key,
                "scenarios": [
                    {
                        "given": format!("{} 을(를) 쓰는 사용자가", subject.name),
                        "when": "필요한 값을 비워 둔 채 보내면",
                        "then": "처리되지 않고 무엇이 비었는지 그 자리에서 알려 줍니다",
                        "evidence": first_test,
                    },
                    {
                        "given": format!("{} 을(를) 아직 한 번도 쓰지 않은 사용자가", subject.name),
                        "when": format!("{} 을(를) 처음 실행하면", subject.name),
                        "then": "먼저 준비가 끝났는지 확인한 뒤에야 처리됩니다",
                        "evidence": first_test,
                    }
                ],
            })
        })
        .collect();
    json!({ "features": features })
}

/// Runs stage 5 and returns the document to persist.
#[allow(clippy::too_many_arguments)]
pub async fn derive(
    http: &reqwest::Client,
    mode: crate::config::Mode,
    provider: llm::Provider,
    key: Option<&str>,
    language: Option<llm::Language>,
    owner: &str,
    name: &str,
    branch: &str,
    paths: &[String],
    subjects: &[Subject],
) -> Result<llm::Answer, String> {
    let paths = crate::cross_cutting::input_paths(paths);
    if paths.is_empty() {
        return Err("repository tree is empty; nothing to read this feature from".to_string());
    }
    if subjects.is_empty() {
        return Err("no approved feature candidate; approve one first".to_string());
    }
    let test_paths: Vec<String> = paths
        .iter()
        .filter(|p| is_test_path(p))
        .cloned()
        .collect();

    let logic = llm::ask(
        http,
        mode,
        provider,
        key,
        Ask {
            system: SYSTEM_LOGIC,
            language,
            user: prompt(owner, name, branch, subjects, &paths, "Files"),
            schema: schema(),
            // mock-exception: LLM-01 — 실 LLM 산출물에 대한 결정적 단정을 위해 고정 답을 공급
            stub: stub_logic(subjects, &paths),
        },
    )
    .await?;

    // "보강할 것이 없다"와 "보강에 실패했다"는 다르고, 없는 입력으로 모델을 부르는 것은
    // 사용자 돈을 쓰는 일이다.
    let tests = if test_paths.is_empty() {
        None
    } else {
        Some(
            llm::ask(
                http,
                mode,
                provider,
                key,
                Ask {
                    system: SYSTEM_TESTS,
                    language,
                    user: prompt(owner, name, branch, subjects, &test_paths, "Test files"),
                    schema: schema(),
                    // mock-exception: LLM-01 — 실 LLM 산출물에 대한 결정적 단정을 위해 고정 답을 공급
                    stub: stub_tests(subjects, &test_paths),
                },
            )
            .await?,
        )
    };

    let empty = json!({ "features": [] });
    let content = merge(
        subjects,
        &logic.content,
        tests.as_ref().map(|a| &a.content).unwrap_or(&empty),
        &test_paths,
        &paths,
    );
    if features(&content).is_empty() {
        return Err("no acceptance scenario cited a path from this repository".to_string());
    }

    Ok(llm::Answer {
        content,
        model: logic.model.clone(),
        // Two provider calls when the repository has test files, one when it does
        // not — the merged answer is stored as a single row, so it carries the count
        // the row cannot otherwise show (AC4.6).
        calls: logic.calls + tests.as_ref().map_or(0, |a| a.calls),
        input_tokens: logic.input_tokens + tests.as_ref().map_or(0, |a| a.input_tokens),
        output_tokens: logic.output_tokens + tests.as_ref().map_or(0, |a| a.output_tokens),
    })
}

/// One feature's acceptance document, in document order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feature {
    pub key: String,
    pub name: String,
    pub scenarios: usize,
    pub contradictions: usize,
}

/// The documents this stage produced, in the order the reviewer reads them.
pub fn features(doc: &Value) -> Vec<Feature> {
    doc.get("features")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|f| {
            Some(Feature {
                key: f.get("key").and_then(Value::as_str)?.to_string(),
                name: f
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                scenarios: f
                    .get("scenarios")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len),
                contradictions: f
                    .get("contradictions")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len),
            })
        })
        .collect()
}

/// The one-liner Analysis Progress renders under the step, mirroring stages 1-4.
pub fn detail(doc: &Value) -> String {
    let found = features(doc);
    let scenarios: usize = found.iter().map(|f| f.scenarios).sum();
    format!("{} features · {} scenarios", found.len(), scenarios)
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_is_accepted_by_openai_strict_mode() {
        crate::llm::assert_strict_schema(&schema());
    }

    use super::*;

    fn tree() -> Vec<String> {
        vec![
            "payments-api/src/api/routes.rs".to_string(),
            "payments-api/src/domain/model.rs".to_string(),
            "payments-api/tests/integration.rs".to_string(),
            "payments-api/README.md".to_string(),
        ]
    }

    fn subjects() -> Vec<Subject> {
        vec![Subject {
            key: "payments-api/src/api/routes.rs".to_string(),
            name: "결제 수단 등록".to_string(),
            location: "payments-api/src/api/routes.rs".to_string(),
            symbol: None,
        }]
    }

    fn merged() -> Value {
        let paths = tree();
        let test_paths: Vec<String> = paths.iter().filter(|p| is_test_path(p)).cloned().collect();
        merge(
            &subjects(),
            &stub_logic(&subjects(), &paths),
            &stub_tests(&subjects(), &test_paths),
            &test_paths,
            &paths,
        )
    }

    #[test]
    fn test_paths_are_recognised_by_convention_not_by_guessing() {
        assert!(is_test_path("payments-api/tests/integration.rs"));
        assert!(is_test_path("web/src/__tests__/cart.ts"));
        assert!(is_test_path("web/src/cart.test.ts"));
        assert!(is_test_path("web/src/cart.spec.tsx"));
        assert!(is_test_path("api/handler_test.go"));
        assert!(is_test_path("api/test_handler.py"));
        assert!(!is_test_path("payments-api/src/api/routes.rs"));
        assert!(!is_test_path("payments-api/testimonials/page.tsx"));
    }

    #[test]
    fn every_scenario_cites_a_path_the_analysis_saw() {
        let doc = merged();
        let paths = tree();
        let features = doc["features"].as_array().unwrap();
        assert!(!features.is_empty());
        for feature in features {
            for scenario in feature["scenarios"].as_array().unwrap() {
                let evidence = scenario["evidence"].as_str().unwrap();
                assert!(
                    paths.iter().any(|p| p == evidence),
                    "scenario cites a path outside the tree: {evidence}"
                );
            }
        }
    }

    #[test]
    fn a_fabricated_evidence_path_is_dropped_rather_than_trusted() {
        let invented = json!({
            "features": [{
                "key": "payments-api/src/api/routes.rs",
                "scenarios": [{
                    "given": "누구든",
                    "when": "무엇을 하든",
                    "then": "무슨 일이 벌어집니다",
                    "evidence": "somewhere/else.rs",
                }],
            }],
        });
        assert!(scenarios_for(&invented, "payments-api/src/api/routes.rs", &tree()).is_empty());
    }

    #[test]
    fn the_test_pass_can_only_add_or_disagree_never_overwrite() {
        let doc = merged();
        let feature = &doc["features"][0];
        let scenarios = feature["scenarios"].as_array().unwrap();
        let contradictions = feature["contradictions"].as_array().unwrap();

        let logic_only = scenarios_for(
            &stub_logic(&subjects(), &tree()),
            &subjects()[0].key,
            &tree(),
        );
        assert!(scenarios.len() >= logic_only.len());
        for logic in &logic_only {
            assert!(
                scenarios
                    .iter()
                    .any(|s| s["given"] == logic.given.as_str() && s["then"] == logic.then.as_str()),
                "the test pass overwrote a logic scenario instead of disagreeing with it"
            );
        }

        assert_eq!(contradictions.len(), 1, "the stub disagrees exactly once");
        let clash = &contradictions[0];
        assert_ne!(clash["codeSays"], clash["testSays"]);
        assert!(is_test_path(clash["testEvidence"].as_str().unwrap()));
        for scenario in scenarios {
            assert_ne!(
                scenario["then"], clash["testSays"],
                "the test's version of a contradicted situation must not be read as a scenario"
            );
        }
        assert!(scenarios
            .iter()
            .any(|s| s["source"] == SOURCE_TEST && is_test_path(s["evidence"].as_str().unwrap())));
    }

    #[test]
    fn a_repository_without_tests_gets_scenarios_but_no_contradictions() {
        let paths = vec!["app/src/main.rs".to_string(), "app/README.md".to_string()];
        let subjects = vec![Subject {
            key: "app/src/main.rs".to_string(),
            name: "무언가".to_string(),
            location: "app/src/main.rs".to_string(),
            symbol: None,
        }];
        let doc = merge(
            &subjects,
            &stub_logic(&subjects, &paths),
            &stub_tests(&subjects, &[]),
            &[],
            &paths,
        );
        let feature = &doc["features"][0];
        assert!(!feature["scenarios"].as_array().unwrap().is_empty());
        assert!(feature["contradictions"].as_array().unwrap().is_empty());
    }

    #[test]
    fn only_the_rewrite_marker_changes_a_sentence_and_only_the_first() {
        use crate::repo_scan::{stub_scan_at, Revision};
        let scan = |r| stub_scan_at("payments-api", "main", r).unwrap().paths;
        let logic = |paths: &[String]| stub_logic(&subjects(), paths);
        let (first, second, third) = (
            logic(&scan(Revision::First)),
            logic(&scan(Revision::Second)),
            logic(&scan(Revision::Third)),
        );
        assert_eq!(first, second);
        assert_ne!(second, third);
        let then = |doc: &Value, i: usize| doc["features"][0]["scenarios"][i]["then"].clone();
        assert_ne!(then(&second, 0), then(&third, 0));
        assert_eq!(then(&second, 1), then(&third, 1));
        assert_eq!(
            doc_len(&second),
            doc_len(&third),
            "리비전은 문장을 고쳐 쓸 뿐 자리를 늘리지 않는다"
        );
    }

    fn doc_len(doc: &Value) -> usize {
        doc["features"][0]["scenarios"].as_array().unwrap().len()
    }

    #[test]
    fn the_same_inputs_produce_the_same_document() {
        assert_eq!(merged(), merged());
    }

    #[test]
    fn detail_counts_what_the_stage_actually_produced() {
        let doc = merged();
        let found = features(&doc);
        assert_eq!(found.len(), 1);
        assert_eq!(
            detail(&doc),
            format!("1 features · {} scenarios", found[0].scenarios)
        );
    }
}
