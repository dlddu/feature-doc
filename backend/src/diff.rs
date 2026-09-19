//! 재분석이 무엇을 바꿨는가 (AC2.6).
//!
//! 이 경계는 **계산만** 한다 — DB 도 HTTP 도 모른다. 두 시점의 같은 타깃 분석이
//! 남긴 것(인수 시나리오 문서 · 의존성 행)을 받아 feature 별로 `+`/`-` 줄을 만든다.
//! 순수 함수로 떼어 둔 이유는 AC2.6 의 검증 방법("동일 feature를 두 시점에 분석하면
//! 변경된 시나리오/추가·제거된 의존성이 diff 형태로 노출된다")이 **판정 규칙**에 대한
//! 요구이고, 그 규칙은 픽스처 없이 단위 테스트로 세울 수 있어야 하기 때문이다.
//!
//! 세 가지 판정 규칙이 여기 있다.
//!
//! 1. **시나리오의 정체성은 정규화한 (given, when, then) 삼중**이다. 근거 경로가
//!    바뀐 것은 같은 시나리오가 다른 자리를 인용한 것이지 *달라진 시나리오*가
//!    아니다 — 그것을 변경으로 세면 화면에 글자가 똑같은 `+`/`-` 한 쌍이 뜬다.
//!    `acceptance::situation` 이 상황의 정체성에 쓰는 공백 정규화와 같은 규칙이다.
//! 2. **의존성의 정체성은 (category, name)** 이다. 0008 마이그레이션의
//!    `UNIQUE(analysis_id, feature_key, category, name)` 가 고른 축과 같다 —
//!    같은 이름이라도 분류가 다르면 다른 항목이라는 성질(`02#시나리오 6`)을 diff 도
//!    그대로 따른다.
//! 3. **변경 0줄인 feature 는 결과에 넣지 않는다.** 여정 `JRN-follow-code-change`
//!    의 `STP-scan-diff` 가 적어 둔 이탈 위험이 그것이다 — "변경되지 않은 기능까지
//!    갱신된 것처럼 보이면 diff를 신뢰하지 않게 된다".

use serde::Serialize;
use serde_json::Value;

/// 줄 한 개의 방향. 화면은 이것을 `+`/`−` 로 그린다.
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
    /// 이번 분석이 그 feature 에 대해 들고 있는 시나리오 수(목업의 "N scenarios").
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
/// 문서는 워커가 넣은 그대로다 — `features[].scenarios[]` 가 `given`·`when`·`then`
/// 을 각각 가진다. 셋 중 하나라도 없는 항목은 화면에 그릴 문장이 없으므로 버린다.
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

    /// 같은 답을 두 번 받은 재분석은 **아무 줄도 만들지 않는다** — 그 feature 는
    /// 목록에 서지 않는다.
    #[test]
    fn an_unchanged_feature_produces_no_diff() {
        let before = doc("k", &[("a", "b", "c"), ("d", "e", "f")]);
        assert_eq!(diff_of(&before, &before.clone()), None);
    }

    /// 공백만 다른 문장은 같은 시나리오다. 이것이 아니면 재포맷 한 번에 문서
    /// 전체가 "달라진 것"으로 뜬다.
    #[test]
    fn whitespace_alone_is_not_a_change() {
        let before = doc("k", &[("a", "b", "c")]);
        let after = doc("k", &[("a  ", "b", " c")]);
        assert_eq!(diff_of(&before, &after), None);
    }

    /// 추가·제거가 각각 한 줄로 선다. 표시 문장은 `then` 이다.
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

    /// 근거 경로만 달라진 것은 변경이 아니다 — 글자가 같은 `+`/`-` 한 쌍을
    /// 만들지 않는다.
    #[test]
    fn a_moved_evidence_path_is_not_a_changed_scenario() {
        let before = doc("k", &[("a", "b", "c")]);
        let mut after = before.clone();
        after["features"][0]["scenarios"][0]["evidence"] =
            json!("payments-api/src/domain/model.rs");
        assert_eq!(diff_of(&before, &after), None);
    }

    /// 처음 문서화된 feature 는 자기 시나리오 전부를 `+` 로 들고 목록에 선다.
    #[test]
    fn a_newly_documented_feature_is_all_additions() {
        let after = doc("k", &[("a", "b", "c"), ("d", "e", "f")]);
        let diff = diff_of(&json!({ "features": [] }), &after).expect("전부 새것이다");
        assert_eq!(diff.scenario_lines.len(), 2);
        assert!(diff.scenario_lines.iter().all(|l| l.mark == "+"));
    }

    /// 의존성은 (분류, 이름) 으로 같고 다름을 가른다 — 이름이 같아도 분류가
    /// 다르면 다른 항목이다(`02#시나리오 6` 이 단정하는 성질).
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

    /// 한쪽이 묻지 않은 것은 "제거됨"이 아니다.
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

    /// 다른 feature 의 시나리오를 끌어오지 않는다.
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

    /// 문서가 아직 없을 때(=단계가 돌지 않았을 때)도 답이 선다.
    #[test]
    fn an_absent_document_reads_as_empty() {
        assert!(scenarios_of(&json!({}), "k").is_empty());
        assert!(feature_keys(&json!({})).is_empty());
        assert_eq!(feature_name(&json!({}), "k"), None);
    }
}
