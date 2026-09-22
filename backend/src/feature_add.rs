//! AC3.2: 자동 추출이 놓친 feature 를 사람이 한 문장으로 더하는 흐름.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::auth::CurrentUser;
use crate::dependencies;
use crate::error::AppError;
use crate::llm::{self, Ask};
use crate::state::AppState;
use crate::util::now_unix;
use crate::{github_app, installations, repo_scan};

/// 이 표에 들어오는 상태 어휘. 0010 의 CHECK 와 같은 값이어야 한다.
pub mod status {
    pub const DRAFTED: &str = "drafted";
    pub const CONFIRMED: &str = "confirmed";
    pub const CANCELLED: &str = "cancelled";
}

pub const SOURCE_USER_LLM: &str = "user_llm";
pub const SOURCE_USER_DIRECT: &str = "user_direct";

pub const KEY_PREFIX: &str = "added:";

/// 한 초안이 세울 수 있는 시나리오·의존성 후보 수의 상한. 한 문장으로 더한 feature
/// 가 자동 문서보다 길어지는 일은 「초안」이 아니다.
const MAX_SCENARIOS: usize = 4;
const MAX_DEPENDENCIES: usize = 6;

const SYSTEM: &str = "\
A person names an end-user feature that the automatic extraction of their
repository missed, in one sentence.
Search the file paths you are given for the code that implements it.
If you find it, draft acceptance criteria as given / when / then in the words of a
person using the product — never developer vocabulary, never an HTTP route or a
function name — and name the dependencies that feature rests on, each in one of
the seven categories: infrastructure, data, architecture, framework, middleware,
logic, interface.
Every criterion and every dependency must name one path from the list as the
evidence it came from. Never cite a path that is not in the list.
If no path in the list implements the feature, answer found=false with empty
lists. Never invent evidence — an honest empty answer is the right answer.";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/analyses/{id}/features/additions",
            get(list).post(draft),
        )
        // 초안도 주소를 가진다 — 화면이 들고 있는 값이 아니라 서버가 들고 있는 값이라야
        // 새로고침이 같은 초안을 다시 그린다.
        .route(
            "/api/analyses/{id}/features/additions/{addition}",
            get(addition),
        )
        .route(
            "/api/analyses/{id}/features/additions/{addition}/decision",
            post(decide),
        )
}

/// 초안의 인수 기준 한 건. 저장·전송·겹쳐 읽기가 모두 이 모양을 쓴다.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DraftScenario {
    pub given: String,
    pub when: String,
    pub then: String,
    pub evidence: String,
}

/// 초안의 의존성 후보 한 건.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DraftDependency {
    pub category: String,
    pub name: String,
    pub evidence: Option<String>,
}

/// 모델이 낸 것 중 이 분석의 트리에 근거를 둔 부분.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Draft {
    pub scenarios: Vec<DraftScenario>,
    pub dependencies: Vec<DraftDependency>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdditionView {
    id: String,
    key: String,
    name: String,
    request: String,
    status: String,
    evidence_found: bool,
    source: Option<String>,
    scenarios: Vec<DraftScenario>,
    dependencies: Vec<DraftDependency>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListView {
    approved_candidates: usize,
    confirmed_additions: usize,
    final_count: usize,
    additions: Vec<AdditionView>,
}

#[derive(Deserialize)]
struct DraftReq {
    request: String,
}

#[derive(Deserialize)]
struct DecisionReq {
    decision: String,
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<ListView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let rows = rows_of(&state, &id).await?;
    let approved: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM feature_candidates \
          WHERE analysis_id = ? AND decision = 'approved' AND merged_into IS NULL",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await?;
    let confirmed = rows.iter().filter(|r| r.status == status::CONFIRMED).count();
    let additions = rows.into_iter().map(view).collect::<Result<Vec<_>, _>>()?;
    Ok(Json(ListView {
        approved_candidates: approved.0 as usize,
        confirmed_additions: confirmed,
        final_count: approved.0 as usize + confirmed,
        additions,
    }))
}

async fn draft(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
    Json(req): Json<DraftReq>,
) -> Result<Json<AdditionView>, AppError> {
    let (owner, name, branch, _) =
        crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let request = req.request.trim().to_string();
    if request.is_empty() {
        return Err(AppError::BadRequest(
            "어떤 기능인지 한 줄은 있어야 근거를 찾을 수 있습니다".into(),
        ));
    }

    let paths = tree_of(&state, &user.id, &owner, &name, &branch).await?;

    // 이 사용자의 키로 부른다 — 분석·편집 제안과 같은 규약이고, 평문은 이 호출 동안만 산다.
    let (provider, key) = match crate::llmkey::active_key_for_user(&state, &user.id).await? {
        Some((provider, key)) => (
            llm::Provider::parse(&provider).unwrap_or(llm::DEFAULT_PROVIDER),
            Some(key),
        ),
        None => (llm::DEFAULT_PROVIDER, None),
    };

    let language = crate::settings::analysis_language(&state.db, &id).await?;

    let answer = llm::ask(
        &state.http,
        state.config.doubles.llm,
        provider,
        key.as_deref(),
        Ask {
            system: SYSTEM,
            language,
            user: prompt(&owner, &name, &branch, &request, &paths),
            schema: schema(),
            // mock-exception: LLM-01 — 실 LLM 산출물에 대한 결정적 단정을 위해 고정 답을 공급
            stub: stub_draft(&request, &paths),
        },
    )
    .await
    .map_err(AppError::BadRequest)?;

    let draft = grounded(&answer.content, &paths);
    let found = !draft.scenarios.is_empty();

    let row_id = uuid::Uuid::new_v4().to_string();
    let key = format!("{KEY_PREFIX}{row_id}");
    sqlx::query(
        "INSERT INTO feature_additions \
           (id, analysis_id, key, name, request, evidence_found, draft_json, status, \
            model, input_tokens, output_tokens, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&row_id)
    .bind(&id)
    .bind(&key)
    .bind(&request)
    .bind(&request)
    .bind(found as i64)
    .bind(serde_json::to_string(&draft).map_err(|e| AppError::internal(e.to_string()))?)
    .bind(status::DRAFTED)
    .bind(&answer.model)
    .bind(answer.input_tokens)
    .bind(answer.output_tokens)
    .bind(now_unix())
    .execute(&state.db)
    .await?;

    Ok(Json(AdditionView {
        id: row_id,
        key,
        name: request.clone(),
        request,
        status: status::DRAFTED.to_string(),
        evidence_found: found,
        source: None,
        scenarios: draft.scenarios,
        dependencies: draft.dependencies,
    }))
}

async fn addition(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, addition)): Path<(String, String)>,
) -> Result<Json<AdditionView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let row = row_of(&state, &id, &addition).await?;
    Ok(Json(view(row)?))
}

async fn decide(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, addition)): Path<(String, String)>,
    Json(req): Json<DecisionReq>,
) -> Result<Json<AdditionView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let row = row_of(&state, &id, &addition).await?;
    if row.status != status::DRAFTED {
        return Err(AppError::Conflict("이미 결정된 추가입니다".into()));
    }

    let next = match req.decision.as_str() {
        "confirm" => status::CONFIRMED,
        "cancel" => status::CANCELLED,
        _ => return Err(AppError::BadRequest("알 수 없는 결정입니다".into())),
    };
    let source = match next {
        status::CONFIRMED if row.evidence_found != 0 => Some(SOURCE_USER_LLM),
        status::CONFIRMED => Some(SOURCE_USER_DIRECT),
        _ => None,
    };
    let draft = draft_from_json(&row.draft_json)?;

    // 결정과 의존성 적재는 한 트랜잭션이다 — 확정된 feature 의 의존성이 반만 실린 채
    // 읽히는 순간이 없어야 한다.
    let now = now_unix();
    let mut tx = state.db.begin().await?;
    sqlx::query(
        "UPDATE feature_additions SET status = ?, source = ?, decided_at = ? WHERE id = ?",
    )
    .bind(next)
    .bind(source)
    .bind(now)
    .bind(&addition)
    .execute(&mut *tx)
    .await?;

    if next == status::CONFIRMED && !draft.dependencies.is_empty() {
        sqlx::query(
            "INSERT INTO feature_dependency_requests \
               (id, analysis_id, feature_key, status, model, requested_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&id)
        .bind(&row.key)
        .bind(dependencies::request_status::SUCCEEDED)
        .bind(&row.model)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        for (seq, dependency) in draft.dependencies.iter().enumerate() {
            sqlx::query(
                "INSERT OR IGNORE INTO feature_dependencies \
                   (id, analysis_id, feature_key, seq, category, name, evidence, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&id)
            .bind(&row.key)
            .bind(seq as i64)
            .bind(&dependency.category)
            .bind(&dependency.name)
            .bind(dependency.evidence.as_deref())
            .bind(now)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;

    crate::audit::record(&state.db, Some(&user.id), "analysis.feature_addition", Some(next))
        .await;

    Ok(Json(AdditionView {
        id: row.id,
        key: row.key,
        name: row.name,
        request: row.request,
        status: next.to_string(),
        evidence_found: row.evidence_found != 0,
        source: source.map(str::to_string),
        scenarios: draft.scenarios,
        dependencies: draft.dependencies,
    }))
}

/// 확정된 추가를 인수 문서 끝에 겹쳐 읽는다.
///
/// 편집 겹치기(`doc_edit::overlay`)보다 **먼저** 실행해야 한다: 그래야 더해진 feature 의
/// 시나리오도 그 뒤의 편집 대상이 된다.
pub async fn overlay(state: &AppState, analysis_id: &str, doc: &mut Value) -> Result<(), AppError> {
    let rows = rows_of(state, analysis_id).await?;
    let Some(features) = doc.get_mut("features").and_then(Value::as_array_mut) else {
        return Ok(());
    };
    for row in rows.into_iter().filter(|r| r.status == status::CONFIRMED) {
        if features
            .iter()
            .any(|f| f.get("key").and_then(Value::as_str) == Some(row.key.as_str()))
        {
            continue;
        }
        let draft = draft_from_json(&row.draft_json)?;
        let source = row.source.clone().unwrap_or_else(|| SOURCE_USER_DIRECT.to_string());
        features.push(feature_json(&row, &draft, &source));
    }
    Ok(())
}

/// 확정된 추가 한 건의 이름 — 의존성 화면이 feature 를 인정하는 두 번째 경로.
pub async fn confirmed_name(
    state: &AppState,
    analysis_id: &str,
    key: &str,
) -> Result<Option<String>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM feature_additions WHERE analysis_id = ? AND key = ? AND status = ?",
    )
    .bind(analysis_id)
    .bind(key)
    .bind(status::CONFIRMED)
    .fetch_optional(&state.db)
    .await?;
    Ok(row.map(|(name,)| name))
}

/// 인수 문서 안의 feature 한 건과 같은 모양. 모순 목록은 비어 있다 — 두 패스가 다툰
/// 것이 아니라 사람이 확정한 한 판이다.
fn feature_json(row: &AdditionRow, draft: &Draft, source: &str) -> Value {
    let scenarios: Vec<Value> = draft
        .scenarios
        .iter()
        .map(|s| {
            json!({
                "given": s.given,
                "when": s.when,
                "then": s.then,
                "evidence": s.evidence,
                "symbol": null,
                "source": source,
            })
        })
        .collect();
    json!({
        "key": row.key,
        "name": row.name,
        "location": draft.scenarios.first().map(|s| s.evidence.clone()),
        "symbol": null,
        "scenarios": scenarios,
        "contradictions": [],
        "source": source,
        "evidenceFound": row.evidence_found != 0,
    })
}

async fn tree_of(
    state: &AppState,
    user_id: &str,
    owner: &str,
    name: &str,
    branch: &str,
) -> Result<Vec<String>, AppError> {
    // 설치 토큰은 이 호출 동안만 산다 — 워커 claim 과 같은 규약.
    let token = match installations::get_for_user(&state.db, user_id).await? {
        Some(inst) => github_app::mint_installation_token(state, inst.installation_id)
            .await
            .ok()
            .map(|t| t.token),
        None => None,
    };
    let scanned = repo_scan::scan(
        &state.http,
        state.config.doubles.repo_scan,
        &state.config.github.api_base,
        owner,
        name,
        branch,
        token.as_deref(),
    )
    .await
    .map_err(AppError::BadRequest)?;
    Ok(crate::cross_cutting::input_paths(&scanned.paths))
}

async fn rows_of(state: &AppState, analysis_id: &str) -> Result<Vec<AdditionRow>, AppError> {
    Ok(sqlx::query_as::<_, AdditionRow>(
        "SELECT id, key, name, request, evidence_found, draft_json, status, source, model \
           FROM feature_additions WHERE analysis_id = ? ORDER BY created_at, rowid",
    )
    .bind(analysis_id)
    .fetch_all(&state.db)
    .await?)
}

async fn row_of(state: &AppState, id: &str, addition: &str) -> Result<AdditionRow, AppError> {
    sqlx::query_as::<_, AdditionRow>(
        "SELECT id, key, name, request, evidence_found, draft_json, status, source, model \
           FROM feature_additions WHERE id = ? AND analysis_id = ?",
    )
    .bind(addition)
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)
}

#[derive(sqlx::FromRow)]
struct AdditionRow {
    id: String,
    key: String,
    name: String,
    request: String,
    evidence_found: i64,
    draft_json: String,
    status: String,
    source: Option<String>,
    model: Option<String>,
}

fn draft_from_json(raw: &str) -> Result<Draft, AppError> {
    serde_json::from_str(raw).map_err(|_| AppError::internal("stored draft is unreadable"))
}

fn view(row: AdditionRow) -> Result<AdditionView, AppError> {
    let draft = draft_from_json(&row.draft_json)?;
    Ok(AdditionView {
        id: row.id,
        key: row.key,
        name: row.name,
        request: row.request,
        status: row.status,
        evidence_found: row.evidence_found != 0,
        source: row.source,
        scenarios: draft.scenarios,
        dependencies: draft.dependencies,
    })
}

fn schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["found", "scenarios", "dependencies"],
        "properties": {
            "found": { "type": "boolean" },
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
                        "evidence": { "type": "string" }
                    }
                }
            },
            "dependencies": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["category", "name", "evidence"],
                    "properties": {
                        "category": { "type": "string" },
                        "name": { "type": "string" },
                        "evidence": { "type": ["string", "null"] }
                    }
                }
            }
        }
    })
}

fn grounded(answer: &Value, paths: &[String]) -> Draft {
    let known = |evidence: &str| paths.iter().any(|p| p == evidence);
    let scenarios = answer
        .get("scenarios")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|s| {
            let evidence = s.get("evidence")?.as_str()?;
            if !known(evidence) {
                return None;
            }
            Some(DraftScenario {
                given: s.get("given")?.as_str()?.to_string(),
                when: s.get("when")?.as_str()?.to_string(),
                then: s.get("then")?.as_str()?.to_string(),
                evidence: evidence.to_string(),
            })
        })
        .take(MAX_SCENARIOS)
        .collect::<Vec<_>>();
    if scenarios.is_empty() {
        return Draft::default();
    }
    let dependencies = answer
        .get("dependencies")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|d| {
            let category = d.get("category")?.as_str()?;
            if !dependencies::is_category(category) {
                return None;
            }
            let name = d.get("name")?.as_str()?.trim().to_string();
            if name.is_empty() {
                return None;
            }
            let evidence = match d.get("evidence").and_then(Value::as_str) {
                Some(path) if known(path) => Some(path.to_string()),
                Some(_) => return None,
                None => None,
            };
            Some(DraftDependency {
                category: category.to_string(),
                name,
                evidence,
            })
        })
        .take(MAX_DEPENDENCIES)
        .collect();
    Draft {
        scenarios,
        dependencies,
    }
}

fn prompt(owner: &str, name: &str, branch: &str, request: &str, paths: &[String]) -> String {
    format!(
        "Repository: {owner}/{name}@{branch}\n\
         The feature the person wants added:\n{request}\n\n\
         Files ({shown}):\n{listed}\n\n\
         Draft at most {max_s} acceptance criteria and at most {max_d} dependencies.",
        shown = paths.len(),
        listed = paths.join("\n"),
        max_s = MAX_SCENARIOS,
        max_d = MAX_DEPENDENCIES,
    )
}

/// 스텁의 「코드 검색」 — 요청 문장에 든 낱말이 트리의 어느 경로에 들어 있는가.
///
/// 실제 모델은 문장 전체를 읽고 스스로 고르지만, 스텁은 결정적이어야 하므로 관측 가능한
/// 규칙 하나로 대신한다. 근거가 하나도 없으면 빈 답이라 「근거 없음」 갈래도 스텁에서 도달된다.
fn stub_matches<'a>(request: &str, paths: &'a [String]) -> Vec<&'a String> {
    let words: Vec<String> = request
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-' && c != '.')
        .filter(|w| w.len() >= 3)
        .map(|w| w.to_ascii_lowercase())
        .collect();
    paths
        .iter()
        .filter(|p| {
            let lower = p.to_ascii_lowercase();
            words.iter().any(|w| lower.contains(w.as_str()))
        })
        .collect()
}

/// 스텁의 답. 근거 경로에서 파생하지 고정 문자열이 아니다 — 배선이 끊겨도 통과하는
/// 상수는 이 단계가 지키려는 바로 그 실패를 숨긴다(2~5단계와 같은 이유).
fn stub_draft(request: &str, paths: &[String]) -> Value {
    let matched = stub_matches(request, paths);
    let Some(first) = matched.first() else {
        return json!({ "found": false, "scenarios": [], "dependencies": [] });
    };
    let mut scenarios = vec![json!({
        "given": format!("{request} 을(를) 아직 한 번도 쓰지 않은 사용자가"),
        "when": format!("{request} 을(를) 처음 실행하면"),
        "then": "요청한 대로 처리되고 결과를 화면에서 바로 확인할 수 있습니다",
        "evidence": first,
    })];
    if let Some(second) = matched.get(1) {
        scenarios.push(json!({
            "given": format!("{request} 을(를) 이미 쓰고 있는 사용자가"),
            "when": "같은 요청을 한 번 더 보내면",
            "then": "같은 결과가 다시 만들어지지 않고 이미 있는 결과를 그대로 보여 줍니다",
            "evidence": second,
        }));
    }
    let dependencies: Vec<Value> = matched
        .iter()
        .filter_map(|path| {
            let category = dependencies::category_for(path)?;
            let stem = path.rsplit_once('/').map(|(_, f)| f).unwrap_or(path.as_str());
            Some(json!({
                "category": category,
                "name": stem.split('.').next().unwrap_or(stem),
                "evidence": path,
            }))
        })
        .take(MAX_DEPENDENCIES)
        .collect();
    json!({ "found": true, "scenarios": scenarios, "dependencies": dependencies })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_accepted_by_openai_strict_mode() {
        crate::llm::assert_strict_schema(&schema());
    }

    fn tree() -> Vec<String> {
        vec![
            "payments-api/README.md".to_string(),
            "payments-api/src/api/routes.rs".to_string(),
            "payments-api/src/middleware/auth.rs".to_string(),
            "payments-api/tests/integration.rs".to_string(),
        ]
    }

    #[test]
    fn the_stub_cites_only_paths_the_request_names() {
        let paths = tree();
        let draft = grounded(&stub_draft("routes 에 있는 회원 탈퇴 기능", &paths), &paths);
        assert!(!draft.scenarios.is_empty());
        for scenario in &draft.scenarios {
            assert!(paths.contains(&scenario.evidence), "{}", scenario.evidence);
            assert!(scenario.evidence.contains("routes"));
        }
        for dependency in &draft.dependencies {
            assert!(dependencies::is_category(&dependency.category));
            assert!(dependency.evidence.as_ref().is_some_and(|e| paths.contains(e)));
        }
    }

    #[test]
    fn a_request_the_tree_does_not_know_finds_no_evidence() {
        let paths = tree();
        let answer = stub_draft("관리자 일괄 삭제 추가", &paths);
        assert_eq!(answer["found"], json!(false));
        let draft = grounded(&answer, &paths);
        assert!(draft.scenarios.is_empty());
        assert!(draft.dependencies.is_empty());
    }

    #[test]
    fn the_stub_answers_the_same_way_for_the_same_ask() {
        let paths = tree();
        assert_eq!(
            stub_draft("auth 미들웨어 뒤의 탈퇴", &paths),
            stub_draft("auth 미들웨어 뒤의 탈퇴", &paths)
        );
    }

    #[test]
    fn an_invented_evidence_path_is_dropped() {
        let paths = tree();
        let answer = json!({
            "found": true,
            "scenarios": [
                { "given": "a", "when": "b", "then": "c", "evidence": "payments-api/src/made/up.rs" },
                { "given": "d", "when": "e", "then": "f", "evidence": "payments-api/src/api/routes.rs" }
            ],
            "dependencies": [
                { "category": "data", "name": "orders", "evidence": "payments-api/src/nope.rs" },
                { "category": "eighth", "name": "x", "evidence": null },
                { "category": "middleware", "name": "auth", "evidence": null }
            ]
        });
        let draft = grounded(&answer, &paths);
        assert_eq!(draft.scenarios.len(), 1);
        assert_eq!(draft.scenarios[0].evidence, "payments-api/src/api/routes.rs");
        assert_eq!(draft.dependencies.len(), 1);
        assert_eq!(draft.dependencies[0].name, "auth");
        assert!(draft.dependencies[0].evidence.is_none());
    }

    /// `found=true` 라고 주장해도 남는 시나리오가 없으면 근거 없음이다 — 의존성만으로
    /// feature 를 세우지 않는다.
    #[test]
    fn a_claim_without_grounded_scenarios_is_not_found() {
        let paths = tree();
        let answer = json!({
            "found": true,
            "scenarios": [],
            "dependencies": [{ "category": "logic", "name": "x", "evidence": null }]
        });
        let draft = grounded(&answer, &paths);
        assert!(draft.scenarios.is_empty());
        assert!(draft.dependencies.is_empty());
    }

    #[test]
    fn the_prompt_carries_the_request_and_the_tree() {
        let text = prompt("acme", "payments-api", "main", "회원 탈퇴", &tree());
        assert!(text.contains("회원 탈퇴"));
        assert!(text.contains("payments-api/src/api/routes.rs"));
    }
}
