//! 종단 의존성: 이 feature 는 무엇에 기대고 있는가?
//!
//! **파이프라인 단계를 더하지 않는다.** [`crate::pipeline::STAGES`] 는 그대로 두고,
//! 승인이 분석을 재큐잉하고 다음 claim 이 그 일을 제안하는 기계를 한 칸 더 쓴다 —
//! `feature_dependency_requests` 행 하나가 게이트이자 실행 상태다.

use serde_json::{json, Value};

use crate::acceptance::{is_test_path, Subject};
use crate::llm::{self, Ask};

/// 한 feature 가 가질 수 있는 의존성 항목 수의 상한. 넘어가면 목록만 보고 영향 범위를
/// 잡는다는 목적이 무너진다.
const MAX_ITEMS: usize = 40;

pub const CATEGORY_INFRASTRUCTURE: &str = "infrastructure";
pub const CATEGORY_DATA: &str = "data";
pub const CATEGORY_ARCHITECTURE: &str = "architecture";
pub const CATEGORY_FRAMEWORK: &str = "framework";
pub const CATEGORY_MIDDLEWARE: &str = "middleware";
pub const CATEGORY_LOGIC: &str = "logic";
pub const CATEGORY_INTERFACE: &str = "interface";

/// 분류 7종이 **한 곳에서만 정의되는** 자리 — 프롬프트, 답변 검증, 마이그레이션의
/// CHECK, 화면의 분류 칩이 전부 이 배열을 출처로 갖는다.
pub const CATEGORIES: [&str; 7] = [
    CATEGORY_INFRASTRUCTURE,
    CATEGORY_DATA,
    CATEGORY_ARCHITECTURE,
    CATEGORY_FRAMEWORK,
    CATEGORY_MIDDLEWARE,
    CATEGORY_LOGIC,
    CATEGORY_INTERFACE,
];

/// 요청 행의 상태값. 중간 상태가 없다 — claim 은 분석 단위로 배타적이라 리스가 끊긴
/// 요청은 `queued` 그대로 남아 다음 claim 에 다시 제안된다.
pub mod request_status {
    pub const QUEUED: &str = "queued";
    pub const SUCCEEDED: &str = "succeeded";
    pub const FAILED: &str = "failed";

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
list, set evidence to null — never cite a path that is not in the list.";

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
                    "required": ["category", "name", "evidence"],
                    "properties": {
                        "category": { "type": "string", "enum": CATEGORIES },
                        "name": { "type": "string" },
                        "evidence": { "type": ["string", "null"] },
                    },
                },
            },
        },
    })
}

/// 의존성 한 줄. `evidence: None` 은 흠결이 아니라 기록된 사실이다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub category: String,
    pub name: String,
    pub evidence: Option<String>,
}

pub fn is_category(category: &str) -> bool {
    CATEGORIES.contains(&category)
}

/// 분류의 표시 순서 — 화면 칩 순서이자 저장되는 `seq` 의 순서다.
fn category_rank(category: &str) -> usize {
    CATEGORIES
        .iter()
        .position(|c| *c == category)
        .unwrap_or(CATEGORIES.len())
}

/// 답변에서 의존성 항목을 읽는다. 적재 경로의 **유일한** 해석 지점이다 — 워커가 만든
/// 문서와 API 가 행으로 옮기는 값이 다른 규칙을 타면 화면과 역방향 질의가 갈린다.
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
/// 영리하기보다 **관습적**이다 — 분류는 모델이 정하고, 이 함수는 결정적 stub 과 단위
/// 테스트가 어떤 트리에서도 같은 답을 내게 하는 자리다.
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

/// 사람이 부를 법한 이름 — 파일 이름이 아니라 그것이 사는 자리를 부른다.
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
/// 고정 문자열이 아니라 **스캔된 트리에서 파생**한다 — 배선이 끊겨도 통과하는 상수를
/// e2e 가 단정하지 않게 하려는 것이다.
fn stub_dependencies(subject: &Subject, paths: &[String]) -> Value {
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

/// 한 feature 의 의존성을 추출한다. 돌려주는 문서는 이미 검증된 것이다.
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
            language,
            user: prompt(owner, name, branch, subject, &paths),
            schema: schema(),
            // mock-exception: LLM-01 — 실 LLM 산출물에 대한 결정적 단정을 위해 고정 답을 공급
            stub: stub_dependencies(subject, &paths),
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
        calls: answer.calls,
        input_tokens: answer.input_tokens,
        output_tokens: answer.output_tokens,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_is_accepted_by_openai_strict_mode() {
        crate::llm::assert_strict_schema(&schema());
    }

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
        assert_eq!(items[0].category, CATEGORY_INFRASTRUCTURE);
        assert_eq!(items[0].evidence, None);
        assert_eq!(items[1].name, "주문");
        assert_eq!(items[1].evidence.as_deref(), Some("a.rs"));
    }

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

    #[test]
    fn stub_is_derived_from_the_tree() {
        let paths = tree();
        let doc = stub_dependencies(&subject(), &paths);
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

        let other = stub_dependencies(&subject(), &["other/src/only.rs".to_string()]);
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
