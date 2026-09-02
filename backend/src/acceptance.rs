//! Stage 5 (`acceptance_dependencies`): what happens to the person using this
//! feature? (AC2.1 · AC2.2 · AC2.3)
//!
//! PRD-2 asks for three things about one **confirmed** feature, in order:
//!
//!   * **AC2.1** — read the feature's *logic* and say what a user experiences, as
//!     인수 기준 ("주어진 ~ / ~ 했을 때 / ~ 해야 한다"). Every criterion carries the
//!     code location it came from; that attachment is the AC, not a nicety.
//!   * **AC2.2** — read the feature's *test* code as well and **보강**: add the
//!     error cases and boundaries the logic pass missed. Where the two disagree,
//!     say so **separately** rather than merging one into the other.
//!   * **AC2.3** — hand back one document per feature, in end-user language.
//!
//! So this module makes **two** calls, not one: a logic pass over the repository's
//! paths and a test pass over the test files among them. Two passes are what make
//! AC2.2's property ("보강 후의 시나리오 수가 보강 전보다 같거나 많다") a thing that
//! can be observed at all — with a single call there is no "before" to compare to.
//!
//! **The contradiction rule is code, not judgment.** [`merge`] calls two scenarios
//! contradictory when they describe the same situation (`given` + `when`) and end
//! differently (`then`). That is a rule the model cannot bend, it works the same on
//! a real answer as on the stub, and it is the reason a contradiction never leaks
//! into the scenario list — test/02 시나리오 3 requires the separation.
//!
//! What this stage does **not** do: dependencies. `acceptance_dependencies` is the
//! wire key the roadmap picked when AC2.1~AC2.6 were one slice; AC2.4~AC2.6 are a
//! per-feature action (`docs/test/02` 시나리오 5), not a pipeline step, and the S04
//! mockup already draws this step as 「인수 시나리오 생성」.

use serde_json::{json, Value};

use crate::llm::{self, Ask};

/// How many confirmed features one run writes documents for. A reviewer opens
/// these one at a time on a phone (`JRN-review-feature`: "12개 feature가 있으면 12번
/// 반복된다"), so the cap is about the run's cost, not the screen's length.
const MAX_FEATURES: usize = 12;

/// Scenarios kept per feature, after 보강. Past this the document stops being
/// something a non-developer reads end to end, which is AC2.3's whole point.
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
                                "required": ["given", "when", "then", "evidence"],
                                "properties": {
                                    "given": { "type": "string" },
                                    "when": { "type": "string" },
                                    "then": { "type": "string" },
                                    "evidence": { "type": "string" },
                                    "symbol": { "type": "string" },
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
/// Deliberately conventional rather than clever: a `test`/`tests`/`spec` directory
/// segment, or a file named like a test. AC2.2 says "동일 feature와 연관된 **테스트
/// 코드**"; this is the one place that answers "which of these paths is that", so
/// the prompt, the answer validation and the e2e cannot disagree about it.
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

/// Whitespace-insensitive identity of the *situation* a scenario describes. Two
/// scenarios with the same situation and different endings are the contradiction
/// AC2.2 asks to surface.
fn situation(s: &Scenario) -> (String, String) {
    (norm(&s.given), norm(&s.when))
}

fn norm(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Reads the scenarios a single answer proposed for one feature key, dropping any
/// whose evidence is not a path this analysis actually saw.
///
/// The check is on the *answer*, not on trust in the instruction: a fabricated
/// location is exactly the failure AC2.1's "근거가 된 코드 위치가 첨부된다" exists to
/// prevent, and it is the same guard stage 4 puts on candidate locations.
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

/// Folds the two passes into one document per feature (AC2.3).
///
/// Order is the approved features, then their logic scenarios, then the test
/// scenarios that **added something new**. A test scenario about a situation the
/// logic pass already covered is either a confirmation (same ending — dropped; it is
/// not a second thing to read) or a disagreement (different ending — lifted into
/// `contradictions` with both sides named, and never appended to the list).
///
/// The logic scenario itself always stays. It is what the code says, and dropping it
/// would leave the contested situation described nowhere in the document a
/// non-developer reads. Deciding which side is right is the reviewer's job — the
/// mockup says so out loud ("어느 쪽이 맞는지는 이 코드를 쓰신 분만 판단할 수 있어요").
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
/// document is read as one narrative (AC2.3), and "which pass found this" is an
/// attribute of the sentence, not a section.
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
/// string — the same reason stages 2-4 do it. The e2e can then assert AC2.1's real
/// property (every criterion cites a path this analysis saw, and the feature's own
/// location is one of them) instead of a constant that would still match if the
/// wiring were cut.
fn stub_logic(subjects: &[Subject], paths: &[String]) -> Value {
    let features: Vec<Value> = subjects
        .iter()
        .take(MAX_FEATURES)
        .map(|subject| {
            let mut scenarios = vec![json!({
                "given": format!("{} 을(를) 아직 한 번도 쓰지 않은 사용자가", subject.name),
                "when": format!("{} 을(를) 처음 실행하면", subject.name),
                "then": "요청한 대로 처리되고 결과를 화면에서 바로 확인할 수 있습니다".to_string(),
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
/// Two shapes, both cited from an actual test file: one criterion the logic pass
/// did not have (보강) and one that repeats the *first* logic situation with a
/// different ending, so the separation test/02 시나리오 3 asks for is observable
/// end to end. It answers only when the repository has test files — a repository
/// without them gets no test pass at all, which is the honest reading of AC2.2.
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

    // AC2.1 — the logic pass.
    let logic = llm::ask(
        http,
        mode,
        provider,
        key,
        Ask {
            system: SYSTEM_LOGIC,
            user: prompt(owner, name, branch, subjects, &paths, "Files"),
            schema: schema(),
            stub: stub_logic(subjects, &paths),
        },
    )
    .await?;

    // AC2.2 — the test pass, skipped outright when the repository has no test code.
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
                    user: prompt(owner, name, branch, subjects, &test_paths, "Test files"),
                    schema: schema(),
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

/// The one-liner S04 renders under the step, mirroring stages 1-4.
pub fn detail(doc: &Value) -> String {
    let found = features(doc);
    let scenarios: usize = found.iter().map(|f| f.scenarios).sum();
    format!("{} features · {} scenarios", found.len(), scenarios)
}

#[cfg(test)]
mod tests {
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
        // A directory that merely *starts* with "test" is not a test directory.
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

        // 보강: nothing the logic pass found is lost, and the test pass could only
        // add to it (AC2.2's 검증 방법 — "시나리오 수가 같거나 많다").
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

        // 분리: the disagreeing sentence goes to its own section and nowhere else.
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
        // The test pass's *other* criterion is in the list, marked as its own source.
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
