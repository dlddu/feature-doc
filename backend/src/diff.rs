//! 재분석이 무엇을 바꿨는가.
//!
//! 의존성의 정체성 축 `(category, name)` 은 0008 마이그레이션의
//! `UNIQUE(analysis_id, feature_key, category, name)` 가 고른 축과 같다 — 둘이
//! 어긋나면 저장에서 한 행인 것이 diff 에서는 두 항목이 된다.

use serde::Serialize;
use serde_json::Value;

pub const ADDED: &str = "+";
pub const REMOVED: &str = "-";

/// 한 시점이 그 feature 에 대해 들고 있는 인수 시나리오 한 건.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    /// 정규화한 (given, when, then) — 같고 다름을 가르는 값.
    pub identity: String,
    /// 사람이 읽는 줄. 관측 가능한 결과인 `then` 을 쓴다.
    pub text: String,
}

/// 한 시점이 그 feature 에 대해 들고 있는 의존성 한 건.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub category: String,
    pub name: String,
}

/// 달라진 인수 시나리오 한 줄.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioLine {
    pub mark: String,
    pub text: String,
}

/// 추가·제거된 의존성 한 줄.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyLine {
    pub mark: String,
    pub category: String,
    pub name: String,
}

/// 한 feature 가 이번 재분석에서 어떻게 달라졌는가.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureDiff {
    pub key: String,
    pub name: String,
    /// 이 분석이 그 feature 를 찾은 자리. 후보 행이 없으면 `None`.
    pub location: Option<String>,
    /// 세는 것은 **이번** 시나리오뿐이다 — 견준 상대의 수가 아니다.
    pub scenarios: usize,
    pub scenario_lines: Vec<ScenarioLine>,
    pub dependency_lines: Vec<DependencyLine>,
}

impl FeatureDiff {
    pub fn lines(&self) -> usize {
        self.scenario_lines.len() + self.dependency_lines.len()
    }
}

/// 공백만 다른 문장을 같은 문장으로 본다 (`acceptance::situation` 과 같은 규칙).
fn norm(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 저장된 인수 문서에서 한 feature 의 시나리오를 읽는다.
///
/// 문서는 워커가 넣은 그대로라 검증되지 않았다 — 셋 중 하나라도 없는 항목은 화면에
/// 그릴 문장이 없으므로 오류가 아니라 누락으로 버린다.
pub fn scenarios_of(doc: &Value, key: &str) -> Vec<Scenario> {
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
            let given = s.get("given").and_then(Value::as_str)?;
            let when = s.get("when").and_then(Value::as_str)?;
            let then = s.get("then").and_then(Value::as_str)?;
            Some(Scenario {
                identity: format!("{}\u{1}{}\u{1}{}", norm(given), norm(when), norm(then)),
                text: norm(then),
            })
        })
        .collect()
}

/// 세 문장에서 견줄 값을 만든다.
pub fn scenario_of(given: &str, when: &str, then: &str) -> Scenario {
    Scenario {
        identity: format!("{}\u{1}{}\u{1}{}", norm(given), norm(when), norm(then)),
        text: norm(then),
    }
}

/// 저장된 인수 문서가 들고 있는 feature 키를 문서 순서대로.
pub fn feature_keys(doc: &Value) -> Vec<String> {
    doc.get("features")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|f| f.get("key").and_then(Value::as_str).map(str::to_string))
        .collect()
}

/// 문서가 그 feature 에 붙여 둔 이름. 없으면 키가 곧 이름이다.
pub fn feature_name(doc: &Value, key: &str) -> Option<String> {
    doc.get("features")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .find(|f| f.get("key").and_then(Value::as_str) == Some(key))
        .and_then(|f| f.get("name").and_then(Value::as_str))
        .map(str::to_string)
        .filter(|n| !n.trim().is_empty())
}

/// 한 feature 의 두 시점을 견준다. 달라진 것이 없으면 `None`.
///
/// `previous_dependencies` / `current_dependencies` 가 `None` 인 것은 **그 시점에
/// 의존성을 묻지 않았다**는 뜻이지 "없다"가 아니다. 한쪽이라도 묻지 않았으면
/// 의존성 줄을 만들지 않는다 — 묻지 않은 것을 "제거됨"으로 읽으면, 재분석 뒤
/// 아직 「의존성 분석」을 누르지 않았다는 사실이 화면에서는 의존성이 통째로
/// 사라진 것처럼 보인다.
pub fn feature_diff(
    key: &str,
    name: &str,
    location: Option<String>,
    previous_scenarios: &[Scenario],
    current_scenarios: &[Scenario],
    previous_dependencies: Option<&[Dependency]>,
    current_dependencies: Option<&[Dependency]>,
) -> Option<FeatureDiff> {
    let mut scenario_lines = Vec::new();
    for scenario in current_scenarios {
        if !previous_scenarios.iter().any(|s| s.identity == scenario.identity) {
            scenario_lines.push(ScenarioLine {
                mark: ADDED.to_string(),
                text: scenario.text.clone(),
            });
        }
    }
    for scenario in previous_scenarios {
        if !current_scenarios.iter().any(|s| s.identity == scenario.identity) {
            scenario_lines.push(ScenarioLine {
                mark: REMOVED.to_string(),
                text: scenario.text.clone(),
            });
        }
    }

    let mut dependency_lines = Vec::new();
    if let (Some(previous), Some(current)) = (previous_dependencies, current_dependencies) {
        let same = |a: &Dependency, b: &Dependency| a.category == b.category && a.name == b.name;
        for dependency in current {
            if !previous.iter().any(|p| same(p, dependency)) {
                dependency_lines.push(DependencyLine {
                    mark: ADDED.to_string(),
                    category: dependency.category.clone(),
                    name: dependency.name.clone(),
                });
            }
        }
        for dependency in previous {
            if !current.iter().any(|c| same(c, dependency)) {
                dependency_lines.push(DependencyLine {
                    mark: REMOVED.to_string(),
                    category: dependency.category.clone(),
                    name: dependency.name.clone(),
                });
            }
        }
    }

    if scenario_lines.is_empty() && dependency_lines.is_empty() {
        return None;
    }
    Some(FeatureDiff {
        key: key.to_string(),
        name: name.to_string(),
        location,
        scenarios: current_scenarios.len(),
        scenario_lines,
        dependency_lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc(key: &str, scenarios: &[(&str, &str, &str)]) -> Value {
        json!({
            "features": [{
                "key": key,
                "name": "결제 수단 등록",
                "scenarios": scenarios
                    .iter()
                    .map(|(given, when, then)| json!({
                        "given": given, "when": when, "then": then,
                        "evidence": "payments-api/src/api/routes.rs",
                    }))
                    .collect::<Vec<_>>(),
            }],
        })
    }

    fn dependency(category: &str, name: &str) -> Dependency {
        Dependency {
            category: category.to_string(),
            name: name.to_string(),
        }
    }

    fn diff_of(previous: &Value, current: &Value) -> Option<FeatureDiff> {
        feature_diff(
            "k",
            "결제 수단 등록",
            None,
            &scenarios_of(previous, "k"),
            &scenarios_of(current, "k"),
            None,
            None,
        )
    }

    #[test]
    fn an_unchanged_feature_produces_no_diff() {
        let before = doc("k", &[("a", "b", "c"), ("d", "e", "f")]);
        assert_eq!(diff_of(&before, &before.clone()), None);
    }

    #[test]
    fn whitespace_alone_is_not_a_change() {
        let before = doc("k", &[("a", "b", "c")]);
        let after = doc("k", &[("a  ", "b", " c")]);
        assert_eq!(diff_of(&before, &after), None);
    }

    #[test]
    fn added_and_removed_scenarios_each_get_a_line() {
        let before = doc("k", &[("a", "b", "묵은 결과")]);
        let after = doc("k", &[("a", "b", "새 결과")]);
        let diff = diff_of(&before, &after).expect("바뀌었다");
        assert_eq!(
            diff.scenario_lines,
            vec![
                ScenarioLine { mark: "+".into(), text: "새 결과".into() },
                ScenarioLine { mark: "-".into(), text: "묵은 결과".into() },
            ]
        );
        assert_eq!(diff.scenarios, 1, "카드가 밝히는 것은 **이번** 시나리오 수다");
    }

    #[test]
    fn a_moved_evidence_path_is_not_a_changed_scenario() {
        let before = doc("k", &[("a", "b", "c")]);
        let mut after = before.clone();
        after["features"][0]["scenarios"][0]["evidence"] =
            json!("payments-api/src/domain/model.rs");
        assert_eq!(diff_of(&before, &after), None);
    }

    #[test]
    fn a_newly_documented_feature_is_all_additions() {
        let after = doc("k", &[("a", "b", "c"), ("d", "e", "f")]);
        let diff = diff_of(&json!({ "features": [] }), &after).expect("전부 새것이다");
        assert_eq!(diff.scenario_lines.len(), 2);
        assert!(diff.scenario_lines.iter().all(|l| l.mark == "+"));
    }

    #[test]
    fn dependencies_are_identified_by_category_and_name() {
        let same = doc("k", &[("a", "b", "c")]);
        let previous = [dependency("logic", "billing · charge")];
        let current = [
            dependency("logic", "billing · charge"),
            dependency("data", "billing · charge"),
        ];
        let diff = feature_diff(
            "k",
            "결제 수단 등록",
            None,
            &scenarios_of(&same, "k"),
            &scenarios_of(&same, "k"),
            Some(&previous),
            Some(&current),
        )
        .expect("의존성이 하나 늘었다");
        assert!(diff.scenario_lines.is_empty());
        assert_eq!(
            diff.dependency_lines,
            vec![DependencyLine {
                mark: "+".into(),
                category: "data".into(),
                name: "billing · charge".into(),
            }]
        );
    }

    #[test]
    fn an_untraced_side_produces_no_dependency_lines() {
        let same = doc("k", &[("a", "b", "c")]);
        let previous = [dependency("logic", "billing · charge")];
        let diff = feature_diff(
            "k",
            "결제 수단 등록",
            None,
            &scenarios_of(&same, "k"),
            &scenarios_of(&same, "k"),
            Some(&previous),
            None,
        );
        assert_eq!(diff, None, "이번에 묻지 않았을 뿐이다");
    }

    #[test]
    fn scenarios_are_read_per_feature() {
        let document = json!({
            "features": [
                { "key": "a", "scenarios": [{ "given": "g", "when": "w", "then": "t1" }] },
                { "key": "b", "scenarios": [{ "given": "g", "when": "w", "then": "t2" }] },
            ],
        });
        assert_eq!(scenarios_of(&document, "a").len(), 1);
        assert_eq!(scenarios_of(&document, "a")[0].text, "t1");
        assert_eq!(feature_keys(&document), vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn an_absent_document_reads_as_empty() {
        assert!(scenarios_of(&json!({}), "k").is_empty());
        assert!(feature_keys(&json!({})).is_empty());
        assert_eq!(feature_name(&json!({}), "k"), None);
    }
}
