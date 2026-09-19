//! 종단 의존성: 이 feature 는 무엇에 기대고 있는가? (AC2.4 · AC2.5)
//!
//! PRD-2 의 뒤쪽 절반이다. 앞쪽 절반([`crate::acceptance`])이 「이 기능을 쓰면
//! 사용자에게 무슨 일이 벌어지는가」를 답한다면, 여기는 「이 기능을 건드리면 무엇이
//! 영향받는가」를 답한다.
//!
//!   * **AC2.4** — 확정된 feature **1건**에 대해 인프라 · 데이터 모델 · 아키텍처
//!     레이어 · 프레임워크/라이브러리 · 미들웨어 · 핵심 로직 모듈 · 외부 인터페이스를
//!     종단으로 식별한다. 각 항목에 **코드 근거**가 붙는다.
//!   * **AC2.5** — 그 결과는 텍스트 문서가 아니라 **행(row)** 으로 적재된다
//!     (`feature_dependencies`, 마이그레이션 0008). 그래야 「데이터 모델 X 를 쓰는
//!     feature 전부」라는 역방향 질의와 export 가 성립한다.
//!
//! **파이프라인 단계가 아니다.** `docs/test/02` 시나리오 5 의 실행 단계는 「해당
//! feature 선택 → "의존성 분석" 트리거」다 — 분석 전체가 아니라 feature 하나에 대한
//! 행동이고, Analysis Progress 목업이 그리는 다섯 단계에 여섯 번째를 더하지 않는다.
//! 그래서 [`crate::pipeline::STAGES`] 는 그대로 두고, 이미 있는 기계 — 사람이 무언가를
//! 승인하면 분석이 재큐잉되고 다음 claim 이 그 일을 제안한다 — 를 한 칸 더 쓴다.
//! 요청은 `feature_dependency_requests` 행이고, 그 행이 곧 게이트이자 실행 상태다.
//!
//! **근거를 지어내지 않는다.** 모델이 든 근거가 이 분석이 실제로 본 경로가 아니면
//! 항목을 버리는 대신 **근거만 떨어뜨린다**(`evidence: None` → 화면의 「근거 없음」).
//! 여정 `JRN-review-feature` 의 예외 표가 그렇게 요구한다 — 「근거 없음으로 명시.
//! 임의로 채우지 않음」. 버리면 의존성 자체가 사라지고, 채우면 거짓이 된다.

use serde_json::{json, Value};

use crate::acceptance::{is_test_path, Subject};
use crate::llm::{self, Ask};

/// 한 feature 가 가질 수 있는 의존성 항목 수의 상한. 넘어가면 영향 범위를 한눈에
/// 잡는다는 AC2.4 의 목적("의존성 목록만 봐도 추론 가능")이 무너진다.
const MAX_ITEMS: usize = 40;

// 프롬프트에 싣는 경로 수의 상한은 [`crate::cross_cutting::input_paths`] 가 이미
// 정한 값을 그대로 쓴다 — 단계마다 다른 상한을 두면 같은 저장소가 단계마다 다른
// 트리로 보인다.

pub const CATEGORY_INFRASTRUCTURE: &str = "infrastructure";
pub const CATEGORY_DATA: &str = "data";
pub const CATEGORY_ARCHITECTURE: &str = "architecture";
pub const CATEGORY_FRAMEWORK: &str = "framework";
pub const CATEGORY_MIDDLEWARE: &str = "middleware";
pub const CATEGORY_LOGIC: &str = "logic";
pub const CATEGORY_INTERFACE: &str = "interface";

/// AC2.4 가 열거한 7종. **한 곳에서만 정의된다** — 프롬프트, 답변 검증, 마이그레이션의
/// CHECK, 화면의 분류 칩이 전부 이 배열을 출처로 갖는다. 목업이 「전체 7종」이라고
/// 적어 둔 그 7이다.
pub const CATEGORIES: [&str; 7] = [
    CATEGORY_INFRASTRUCTURE,
    CATEGORY_DATA,
    CATEGORY_ARCHITECTURE,
    CATEGORY_FRAMEWORK,
    CATEGORY_MIDDLEWARE,
    CATEGORY_LOGIC,
    CATEGORY_INTERFACE,
];

/// 요청 행(`feature_dependency_requests.status`)이 가질 수 있는 값.
///
/// 중간 상태가 없다. claim 은 분석 단위로 배타적이므로, 리스가 끊긴 요청은 `queued`
/// 그대로 남아 다음 claim 에 다시 제안된다 — 회수하려고 상태를 되돌릴 자리가 없다.
pub mod request_status {
    pub const QUEUED: &str = "queued";
    pub const SUCCEEDED: &str = "succeeded";
    pub const FAILED: &str = "failed";

    /// 워커가 보고할 수 있는 종료 상태인가.
    pub fn is_terminal(status: &str) -> bool {
        matches!(status, SUCCEEDED | FAILED)
    }
}

const SYSTEM: &str = "\
You map the end-to-end dependencies of one feature by reading the repository's file
paths.
Work only from the file paths you are given.
Classify every dependency into exactly one of these categories: infrastructure, data,
architecture, framework, middleware, logic, interface.
Name each dependency the way a person on the product team would say it, not as a file
name.
Give the path you read it from as the evidence. If you cannot point at a path in the
list, leave the evidence out — never cite a path that is not in the list.";

/// 답변이 맞춰야 하는 JSON 모양(제공자에 그대로 전달된다).
fn schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["items"],
        "properties": {
            "items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["category", "name"],
                    "properties": {
                        "category": { "type": "string", "enum": CATEGORIES },
                        "name": { "type": "string" },
                        "evidence": { "type": "string" },
                    },
                },
            },
        },
    })
}

/// 의존성 한 줄. `evidence` 가 `None` 인 것은 기록된 사실이다 — 화면은 그것을
/// 「근거 없음」으로 그린다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub category: String,
    pub name: String,
    pub evidence: Option<String>,
}

/// 주어진 분류가 7종 중 하나인가.
pub fn is_category(category: &str) -> bool {
    CATEGORIES.contains(&category)
}

/// 분류의 표시 순서. 목업의 칩 순서이자 저장되는 `seq` 의 순서다.
fn category_rank(category: &str) -> usize {
    CATEGORIES
        .iter()
        .position(|c| *c == category)
        .unwrap_or(CATEGORIES.len())
}

/// 답변에서 의존성 항목을 읽는다. 알 수 없는 분류·빈 이름은 버리고, 같은
/// (분류, 이름)은 먼저 나온 것만 남기며, 분류 순서로 정렬해 상한까지 자른다.
///
/// 이 함수가 적재 경로의 **유일한** 해석 지점이다 — 워커가 만든 문서와 API 가
/// 행으로 옮기는 값이 서로 다른 규칙을 타면 화면과 역방향 질의가 갈린다.
pub fn items(doc: &Value) -> Vec<Item> {
    let Some(raw) = doc.get("items").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out: Vec<Item> = Vec::new();
    for entry in raw {
        let Some(category) = entry.get("category").and_then(|v| v.as_str()) else {
            continue;
        };
        if !is_category(category) {
            continue;
        }
        let name = entry
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        if out
            .iter()
            .any(|i| i.category == category && i.name == name)
        {
            continue;
        }
        let evidence = entry
            .get("evidence")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .map(str::to_string);
        out.push(Item {
            category: category.to_string(),
            name,
            evidence,
        });
    }
    out.sort_by_key(|i| category_rank(&i.category));
    out.truncate(MAX_ITEMS);
    out
}

/// 항목들을 저장·전송되는 문서 모양으로 되돌린다.
pub fn document(items: &[Item]) -> Value {
    json!({
        "items": items
            .iter()
            .map(|i| json!({
                "category": i.category,
                "name": i.name,
                "evidence": i.evidence,
            }))
            .collect::<Vec<_>>(),
    })
}

/// 상태 줄에 쓰는 한 줄 요약.
pub fn detail(doc: &Value) -> String {
    let items = items(doc);
    let categories = CATEGORIES
        .iter()
        .filter(|c| items.iter().any(|i| i.category == **c))
        .count();
    let unsourced = items.iter().filter(|i| i.evidence.is_none()).count();
    format!("{} dependencies · {categories} categories · {unsourced} without evidence", items.len())
}

/// 한 경로가 어느 분류에 해당하는가. 의존성으로 셀 것이 아니면 `None`.
///
/// 영리하기보다 **관습적**이다. 이 함수는 결정적 stub 이 어떤 트리에서도 같은 답을
/// 내게 하는 자리이고, 답변 검증이 「이 경로를 이 분류로 부를 수 있는가」를 묻는
/// 자리가 아니다 — 분류는 모델이 정하고, 이 함수는 stub 과 단위 테스트의 것이다.
pub fn category_for(path: &str) -> Option<&'static str> {
    if is_test_path(path) {
        return None;
    }
    let (dirs, file) = match path.rsplit_once('/') {
        Some((dirs, file)) => (dirs, file),
        None => ("", path),
    };
    let file = file.to_ascii_lowercase();
    let segments: Vec<String> = dirs
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase())
        .collect();

    // 숨김 디렉터리(`.github` 등)와 읽을거리는 의존성이 아니다.
    if segments.iter().any(|s| s.starts_with('.')) || file.starts_with('.') {
        return None;
    }
    if matches!(
        file.rsplit_once('.').map(|(_, ext)| ext).unwrap_or(""),
        "md" | "txt" | "rst" | "lock" | "svg" | "png" | "jpg" | "ico"
    ) {
        return None;
    }

    let seg = |names: &[&str]| segments.iter().any(|s| names.contains(&s.as_str()));

    if seg(&["deploy", "deployment", "k8s", "kubernetes", "infra", "infrastructure", "charts", "terraform"])
        || matches!(file.as_str(), "dockerfile" | "docker-compose.yml" | "docker-compose.yaml")
    {
        return Some(CATEGORY_INFRASTRUCTURE);
    }
    if seg(&["migrations", "models", "entities", "schema", "db", "database"])
        || matches!(
            file.split('.').next().unwrap_or(""),
            "model" | "models" | "schema" | "entity" | "entities"
        )
    {
        return Some(CATEGORY_DATA);
    }
    if seg(&["middleware", "middlewares", "interceptors", "filters", "guards"]) {
        return Some(CATEGORY_MIDDLEWARE);
    }
    if matches!(
        file.as_str(),
        "cargo.toml"
            | "package.json"
            | "go.mod"
            | "pyproject.toml"
            | "requirements.txt"
            | "gemfile"
            | "pom.xml"
            | "build.gradle"
    ) {
        return Some(CATEGORY_FRAMEWORK);
    }
    if seg(&["api", "routes", "handlers", "controllers", "web", "ui", "pages", "components", "cli", "cmd"]) {
        return Some(CATEGORY_INTERFACE);
    }
    if matches!(
        file.as_str(),
        "lib.rs" | "main.rs" | "mod.rs" | "index.ts" | "index.js" | "app.ts" | "app.tsx"
    ) {
        return Some(CATEGORY_ARCHITECTURE);
    }
    Some(CATEGORY_LOGIC)
}

/// 사람이 부를 법한 이름. 파일 이름이 아니라 그것이 사는 자리를 부른다
/// (`payments-api/src/domain/model.rs` → `domain · model`).
fn item_name(path: &str) -> String {
    let (dirs, file) = match path.rsplit_once('/') {
        Some((dirs, file)) => (dirs, file),
        None => ("", path),
    };
    let stem = file.split('.').next().unwrap_or(file);
    match dirs.rsplit('/').find(|s| !s.is_empty()) {
        Some(parent) => format!("{parent} · {stem}"),
        None => stem.to_string(),
    }
}

fn prompt(owner: &str, name: &str, branch: &str, subject: &Subject, paths: &[String]) -> String {
    let symbol = subject.symbol.as_deref().unwrap_or("-");
    format!(
        "Repository: {owner}/{name}@{branch}\n\
         Feature: {key} — {label}\n\
         Found at: {location} · {symbol}\n\n\
         Files ({shown}):\n{listed}\n\n\
         Answer with at most {max} dependencies for this one feature.",
        key = subject.key,
        label = subject.name,
        location = subject.location,
        shown = paths.len(),
        listed = paths.join("\n"),
        max = MAX_ITEMS,
    )
}

/// 결정적 stub.
///
/// 고정 문자열이 아니라 **스캔된 트리에서 파생**한다 — 다른 단계들이 그렇게 하는
/// 이유와 같다. 그래야 e2e 가 AC2.4 의 실제 성질(분류별로 갈리고, 각 항목이 이 분석이
/// 본 경로를 근거로 든다)을 단정할 수 있고, 배선이 끊겨도 통과하는 상수를 단정하지
/// 않게 된다. feature 자신이 발견된 자리는 언제나 한 줄로 들어간다 — 어떤 feature 든
/// 최소한 자기 진입점에는 기대고 있다.
fn stub(subject: &Subject, paths: &[String]) -> Value {
    let mut items: Vec<Item> = Vec::new();
    let mut push = |category: &str, path: &str| {
        if items
            .iter()
            .any(|i: &Item| i.category == category && i.evidence.as_deref() == Some(path))
        {
            return;
        }
        items.push(Item {
            category: category.to_string(),
            name: item_name(path),
            evidence: Some(path.to_string()),
        });
    };

    if let Some(category) = category_for(&subject.location) {
        push(category, &subject.location);
    }
    for category in CATEGORIES {
        if let Some(path) = paths
            .iter()
            .find(|p| category_for(p) == Some(category))
        {
            push(category, path.as_str());
        }
    }
    document(&items)
}

/// 한 feature 의 의존성을 추출한다 (AC2.4). 돌려주는 문서는 이미 검증된 것이다 —
/// 이 분석이 보지 않은 경로를 근거로 든 항목은 근거만 비워서 남는다.
pub async fn derive(
    http: &reqwest::Client,
    mode: crate::config::Mode,
    provider: llm::Provider,
    key: Option<&str>,
    owner: &str,
    name: &str,
    branch: &str,
    paths: &[String],
    subject: &Subject,
) -> Result<llm::Answer, String> {
    let paths: Vec<String> = crate::cross_cutting::input_paths(paths)
        .into_iter()
        .filter(|p| !is_test_path(p))
        .collect();
    if paths.is_empty() {
        return Err("repository tree is empty; nothing to trace this feature through".to_string());
    }

    let answer = llm::ask(
        http,
        mode,
        provider,
        key,
        Ask {
            system: SYSTEM,
            user: prompt(owner, name, branch, subject, &paths),
            schema: schema(),
            // mock-exception: LLM-01 — 실 LLM 산출물에 대한 결정적 단정을 위해 고정 답을 공급
            stub: stub(subject, &paths),
        },
    )
    .await?;

    let validated: Vec<Item> = items(&answer.content)
        .into_iter()
        .map(|mut item| {
            if let Some(evidence) = &item.evidence {
                if !paths.iter().any(|p| p == evidence) {
                    item.evidence = None;
                }
            }
            item
        })
        .collect();
    if validated.is_empty() {
        return Err("no dependency was named for this feature".to_string());
    }

    Ok(llm::Answer {
        content: document(&validated),
        model: answer.model,
        input_tokens: answer.input_tokens,
        output_tokens: answer.output_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject() -> Subject {
        Subject {
            key: "payments-api/src/api/routes.rs".to_string(),
            name: "결제 완료".to_string(),
            location: "payments-api/src/api/routes.rs".to_string(),
            symbol: Some("pay".to_string()),
        }
    }

    fn tree() -> Vec<String> {
        [
            "payments-api/Cargo.toml",
            "payments-api/README.md",
            "payments-api/deploy/base/deployment.yaml",
            "payments-api/src/api/routes.rs",
            "payments-api/src/domain/model.rs",
            "payments-api/src/middleware/auth.rs",
            "payments-api/src/main.rs",
            "payments-api/src/billing.rs",
            "payments-api/tests/integration.rs",
            "payments-api/.github/workflows/ci.yml",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    #[test]
    fn categories_are_the_seven_ac24_lists() {
        assert_eq!(CATEGORIES.len(), 7);
        assert!(is_category(CATEGORY_LOGIC));
        assert!(!is_category("database"));
    }

    #[test]
    fn category_for_reads_the_place_not_the_name() {
        assert_eq!(category_for("payments-api/deploy/base/deployment.yaml"), Some(CATEGORY_INFRASTRUCTURE));
        assert_eq!(category_for("payments-api/src/domain/model.rs"), Some(CATEGORY_DATA));
        assert_eq!(category_for("payments-api/src/middleware/auth.rs"), Some(CATEGORY_MIDDLEWARE));
        assert_eq!(category_for("payments-api/Cargo.toml"), Some(CATEGORY_FRAMEWORK));
        assert_eq!(category_for("payments-api/src/api/routes.rs"), Some(CATEGORY_INTERFACE));
        assert_eq!(category_for("payments-api/src/main.rs"), Some(CATEGORY_ARCHITECTURE));
        assert_eq!(category_for("payments-api/src/billing.rs"), Some(CATEGORY_LOGIC));
    }

    /// 테스트 코드·문서·CI 설정은 이 기능이 기대는 것이 아니다.
    #[test]
    fn category_for_skips_what_is_not_a_dependency() {
        assert_eq!(category_for("payments-api/tests/integration.rs"), None);
        assert_eq!(category_for("payments-api/README.md"), None);
        assert_eq!(category_for("payments-api/.github/workflows/ci.yml"), None);
    }

    #[test]
    fn items_drop_unknown_categories_and_fold_duplicates() {
        let doc = json!({
            "items": [
                { "category": "data", "name": "주문", "evidence": "a.rs" },
                { "category": "data", "name": "주문", "evidence": "b.rs" },
                { "category": "wishful", "name": "무엇", "evidence": "a.rs" },
                { "category": "logic", "name": "  ", "evidence": "a.rs" },
                { "category": "infrastructure", "name": "결제사" },
            ]
        });
        let items = items(&doc);
        assert_eq!(items.len(), 2);
        // 분류 순서(CATEGORIES)대로 정렬된다.
        assert_eq!(items[0].category, CATEGORY_INFRASTRUCTURE);
        assert_eq!(items[0].evidence, None);
        assert_eq!(items[1].name, "주문");
        assert_eq!(items[1].evidence.as_deref(), Some("a.rs"));
    }

    /// 근거가 이 분석이 본 경로가 아니면 **항목이 아니라 근거가** 사라진다.
    #[test]
    fn fabricated_evidence_becomes_no_evidence() {
        let paths = tree();
        let answered = json!({
            "items": [
                { "category": "data", "name": "결제 엔티티", "evidence": "payments-api/src/domain/model.rs" },
                { "category": "infrastructure", "name": "결제사", "evidence": "payments-api/vendor/psp.rs" },
            ]
        });
        let validated: Vec<Item> = items(&answered)
            .into_iter()
            .map(|mut item| {
                if let Some(evidence) = &item.evidence {
                    if !paths.iter().any(|p| p == evidence) {
                        item.evidence = None;
                    }
                }
                item
            })
            .collect();
        assert_eq!(validated.len(), 2);
        assert_eq!(validated[0].category, CATEGORY_INFRASTRUCTURE);
        assert_eq!(validated[0].evidence, None, "지어낸 경로는 근거로 남지 않는다");
        assert_eq!(
            validated[1].evidence.as_deref(),
            Some("payments-api/src/domain/model.rs")
        );
    }

    /// stub 은 고정 문자열이 아니라 트리에서 파생된다 — 트리가 달라지면 답도 달라진다.
    #[test]
    fn stub_is_derived_from_the_tree() {
        let paths = tree();
        let doc = stub(&subject(), &paths);
        let items = items(&doc);
        assert!(!items.is_empty());
        for item in &items {
            let evidence = item.evidence.as_deref().expect("stub always cites a path");
            assert!(paths.iter().any(|p| p == evidence), "{evidence}");
            assert!(is_category(&item.category));
        }
        let categories: Vec<&str> = CATEGORIES
            .iter()
            .filter(|c| items.iter().any(|i| i.category == **c))
            .copied()
            .collect();
        assert!(
            categories.len() >= 5,
            "다층 의존성을 가진 트리는 여러 분류로 갈려야 한다: {categories:?}"
        );

        let other = stub(&subject(), &["other/src/only.rs".to_string()]);
        assert_ne!(items_names(&doc), items_names(&other));
    }

    fn items_names(doc: &Value) -> Vec<String> {
        items(doc).into_iter().map(|i| i.name).collect()
    }

    #[test]
    fn detail_counts_what_the_status_line_shows() {
        let doc = json!({
            "items": [
                { "category": "data", "name": "주문", "evidence": "a.rs" },
                { "category": "infrastructure", "name": "결제사" },
            ]
        });
        let detail = detail(&doc);
        assert!(detail.contains("2 dependencies"), "{detail}");
        assert!(detail.contains("2 categories"), "{detail}");
        assert!(detail.contains("1 without evidence"), "{detail}");
    }
}
