//! AC3.1: 읽던 사람이 한 줄로 부탁하면 그 시나리오를 고쳐 주는 흐름.
//!
//! 세 번의 탭이 전부다 — 고쳐 달라 하기 → 제안 보기 → 승인. 그래서 제안은 큐를 타지
//! 않고 이 프로세스가 직접 모델을 부른다. 워커를 쓰면 「요청함 · 기다리는 중」이라는
//! 네 번째 화면이 생겨 여정이 못 박은 3탭(F8, `JRN-review-feature`)이 깨진다. 분석
//! 파이프라인이 워커에 있는 이유(수십 분짜리 다단계 작업)가 여기에는 없다 — 이것은
//! 한 번의 호출이고, 사용자가 화면 앞에서 그 답을 기다린다.
//!
//! **승인하기 전에는 아무것도 바뀌지 않는다.** 제안은 `feature_doc_edits` 에 행으로
//! 남고, 승인된 행만 읽는 시점에 자동 문서 위로 겹쳐진다([`overlay`]). 자동 문서를
//! 직접 고치지 않는 이유는 0009 마이그레이션 주석에 있다(재현성·재분석 diff 의 기준값).
//!
//! **거부는 버려지지 않는다.** AC3.1 의 검증 방법이 「거부된 제안은 기록되어 다음
//! 제안 시 회피된다」이므로, 거부된 문면과 사유는 다음 호출의 프롬프트로 들어간다
//! ([`avoid_list`]). 화면도 같은 값을 읽어 「같은 방향은 다시 제안하지 않습니다」를
//! 사람에게 먼저 알린다.

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::auth::CurrentUser;
use crate::diff;
use crate::error::AppError;
use crate::llm::{self, Ask};
use crate::pipeline;
use crate::state::AppState;
use crate::util::now_unix;

/// 이 표에 들어오는 상태 어휘. 0009 의 CHECK 와 같은 값이어야 한다.
pub mod status {
    pub const PROPOSED: &str = "proposed";
    pub const APPROVED: &str = "approved";
    pub const REJECTED: &str = "rejected";
}

/// AC3.4 의 출처 중 이 흐름이 남기는 값.
pub const SOURCE_USER_LLM: &str = "user_llm";

/// 한 요청이 회피 목록으로 들고 갈 수 있는 거부 건수의 상한.
///
/// 프롬프트가 무한히 길어지지 않게 하는 것이 목적이고, 최근 것부터 담는다 — 사람이
/// 방금 거부한 방향이 다음 제안에서 가장 먼저 피해야 할 방향이다.
const MAX_AVOIDED: usize = 5;

const SYSTEM: &str = "\
You revise one acceptance scenario of an end-user feature, following a single
instruction from the person reading it.
Answer with the scenarios that should stand in its place: one when the instruction
asks you to reword it, two when it asks for another case to be covered as well.
Keep the given / when / then shape, and change only what the instruction asks for.
Write in the words of a person using the product — never developer vocabulary,
never an HTTP route or a function name.
Never invent behaviour the feature does not have.
Some directions were already rejected by this reader; they are listed. Do not
propose them again — take a different one.";

/// 한 제안이 그 자리에 세울 수 있는 시나리오 수의 상한. 한 번의 요청이 문서를
/// 통째로 다시 쓰는 일은 「한 줄로 부탁한다」가 아니다.
const MAX_PROPOSED: usize = 3;

pub fn routes() -> Router<AppState> {
    Router::new()
        // feature key 는 경로 구분자를 품으므로 본문·쿼리로 받는다(`analysis` 와 같은 사유).
        .route("/api/analyses/{id}/features/edit-context", get(context))
        .route("/api/analyses/{id}/features/edit-proposals", post(propose))
        // 제안도 주소를 가진다 — 화면이 들고 있는 값이 아니라 서버가 들고 있는 값이라야
        // 새로고침이 같은 제안을 다시 그린다.
        .route(
            "/api/analyses/{id}/features/edit-proposals/{proposal}",
            get(proposal),
        )
        .route(
            "/api/analyses/{id}/features/edit-proposals/{proposal}/decision",
            post(decide),
        )
}

/// 한 시나리오의 세 문장. 저장·전송·프롬프트가 모두 이 모양을 쓴다.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sentences {
    pub given: String,
    pub when: String,
    pub then: String,
}

impl Sentences {
    fn from_value(value: &Value) -> Option<Self> {
        Some(Self {
            given: value.get("given")?.as_str()?.to_string(),
            when: value.get("when")?.as_str()?.to_string(),
            then: value.get("then")?.as_str()?.to_string(),
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContextView {
    feature_key: String,
    feature_name: Option<String>,
    scenario_index: usize,
    scenario_count: usize,
    target: Sentences,
    /// 이 feature 에서 사람이 거부한 제안 수. 0 이면 화면은 그 안내를 그리지 않는다.
    rejected_count: usize,
    /// 가장 최근 거부의 사유. 「같은 방향」이 무엇인지 사람에게 보여 주는 값이다.
    rejected_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProposalView {
    id: String,
    feature_key: String,
    scenario_index: usize,
    status: String,
    /// AC3.4 의 출처. 화면의 「바꾼 주체」가 이 값을 읽는다.
    source: String,
    /// 사람이 적어 보낸 한 줄. 이력에 남는 값이라 되돌려준다.
    request: String,
    before: Sentences,
    /// 그 자리에 설 시나리오들. 한 건이면 고쳐 쓴 것이고, 두 건 이상이면 사례가 늘어난 것이다.
    after: Vec<Sentences>,
    /// 사라지는 문장과 들어오는 문장. 화면의 diff 가 이 두 목록을 그대로 그린다.
    removed: Vec<String>,
    added: Vec<String>,
    /// 배지가 읽는 수 — 지워지는 줄과 들어오는 줄의 합이다.
    changed_lines: usize,
}

#[derive(Deserialize)]
struct FeatureQuery {
    key: String,
    /// 고칠 시나리오의 자리(0-based). 화면은 사람이 누른 그 줄을 보낸다.
    #[serde(default)]
    scenario: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProposeReq {
    key: String,
    #[serde(default)]
    scenario_index: usize,
    /// 「이렇게 고쳐 주세요」 한 줄. 빈 요청은 모델을 부르지 않는다.
    request: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DecisionReq {
    /// `approve` 또는 `reject`.
    decision: String,
    /// 거부 사유. 거부에는 반드시 있어야 한다 — 없으면 다음 제안이 무엇을 피할지 모른다.
    #[serde(default)]
    reason: Option<String>,
}

/// 편집 요청 화면이 열릴 때 필요한 것 전부: 고칠 대상과, 이미 거부된 방향.
async fn context(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
    Query(query): Query<FeatureQuery>,
) -> Result<Json<ContextView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let doc = document_of(&state, &id).await?;
    let scenarios = scenarios_of(&doc, &query.key)?;
    let target = scenarios
        .get(query.scenario)
        .cloned()
        .ok_or(AppError::NotFound)?;
    let rejected = rejections(&state, &id, &query.key).await?;

    Ok(Json(ContextView {
        feature_key: query.key.clone(),
        feature_name: diff::feature_name(&doc, &query.key),
        scenario_index: query.scenario,
        scenario_count: scenarios.len(),
        target,
        rejected_count: rejected.len(),
        rejected_reason: rejected.first().map(|(_, reason)| reason.clone()),
    }))
}

/// 한 줄의 요청을 제안 하나로 바꾼다. 문서는 아직 그대로다.
async fn propose(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
    Json(req): Json<ProposeReq>,
) -> Result<Json<ProposalView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let request = req.request.trim().to_string();
    if request.is_empty() {
        return Err(AppError::BadRequest(
            "무엇을 고칠지 한 줄은 있어야 합니다".into(),
        ));
    }

    let doc = document_of(&state, &id).await?;
    let scenarios = scenarios_of(&doc, &req.key)?;
    let before = scenarios
        .get(req.scenario_index)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let rejected = rejections(&state, &id, &req.key).await?;
    let avoid = avoid_list(&rejected);

    // 이 사용자의 키로 부른다 — 분석과 같은 규약이고, 평문은 이 호출 동안만 산다.
    let (provider, key) = match crate::llmkey::active_key_for_user(&state, &user.id).await? {
        Some((provider, key)) => (
            llm::Provider::parse(&provider).unwrap_or(llm::DEFAULT_PROVIDER),
            Some(key),
        ),
        None => (llm::DEFAULT_PROVIDER, None),
    };

    let answer = llm::ask(
        &state.http,
        state.config.doubles.llm,
        provider,
        key.as_deref(),
        Ask {
            system: SYSTEM,
            user: prompt(&before, &request, &avoid),
            schema: schema(),
            // mock-exception: LLM-01 — 실 LLM 산출물에 대한 결정적 단정을 위해 고정 답을 공급
            stub: stub_edit(&before, &request, &avoid),
        },
    )
    .await
    .map_err(AppError::BadRequest)?;

    let after = proposed(&answer.content).ok_or_else(|| {
        AppError::BadRequest("모델이 고쳐 쓴 문장을 읽을 수 없었습니다".into())
    })?;

    let row_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO feature_doc_edits \
           (id, analysis_id, feature_key, scenario_index, request, before_json, after_json, \
            status, source, model, input_tokens, output_tokens, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&row_id)
    .bind(&id)
    .bind(&req.key)
    .bind(req.scenario_index as i64)
    .bind(&request)
    .bind(serde_json::to_string(&before).map_err(|e| AppError::internal(e.to_string()))?)
    .bind(serde_json::to_string(&after).map_err(|e| AppError::internal(e.to_string()))?)
    .bind(status::PROPOSED)
    .bind(SOURCE_USER_LLM)
    .bind(&answer.model)
    .bind(answer.input_tokens)
    .bind(answer.output_tokens)
    .bind(now_unix())
    .execute(&state.db)
    .await?;

    Ok(Json(view(
        row_id,
        req.key,
        req.scenario_index,
        status::PROPOSED.to_string(),
        request,
        before,
        after,
    )))
}

/// 이미 만들어진 제안 하나. 결정 화면이 새로고침을 견디는 근거다.
async fn proposal(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, proposal)): Path<(String, String)>,
) -> Result<Json<ProposalView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;
    let row = edit_row(&state, &id, &proposal).await?;
    let before = sentences_from_json(&row.before_json)?;
    let after = scenarios_from_json(&row.after_json)?;
    Ok(Json(view(
        row.id,
        row.feature_key,
        row.scenario_index as usize,
        row.status,
        row.request,
        before,
        after,
    )))
}

/// 승인이면 겹쳐 읽히기 시작하고, 거부면 다음 제안이 피해야 할 것이 된다.
async fn decide(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, proposal)): Path<(String, String)>,
    Json(req): Json<DecisionReq>,
) -> Result<Json<ProposalView>, AppError> {
    crate::analysis::owned_analysis(&state, &user.id, &id).await?;

    let row = edit_row(&state, &id, &proposal).await?;

    if row.status != status::PROPOSED {
        return Err(AppError::Conflict("이미 결정된 제안입니다".into()));
    }

    let (next, reason) = match req.decision.as_str() {
        "approve" => (status::APPROVED, None),
        "reject" => {
            let reason = req.reason.unwrap_or_default().trim().to_string();
            if reason.is_empty() {
                return Err(AppError::BadRequest(
                    "왜 이게 아닌지 한 줄이 있어야 다음 제안이 그 방향을 피합니다".into(),
                ));
            }
            (status::REJECTED, Some(reason))
        }
        _ => return Err(AppError::BadRequest("알 수 없는 결정입니다".into())),
    };

    sqlx::query(
        "UPDATE feature_doc_edits SET status = ?, reason = ?, decided_at = ? WHERE id = ?",
    )
    .bind(next)
    .bind(&reason)
    .bind(now_unix())
    .bind(&proposal)
    .execute(&state.db)
    .await?;

    let before = sentences_from_json(&row.before_json)?;
    let after = scenarios_from_json(&row.after_json)?;
    Ok(Json(view(
        row.id,
        row.feature_key,
        row.scenario_index as usize,
        next.to_string(),
        row.request,
        before,
        after,
    )))
}

/// 이 분석의 제안 한 행. 분석 소유권은 부른 쪽이 이미 확인했다.
async fn edit_row(state: &AppState, id: &str, proposal: &str) -> Result<EditRow, AppError> {
    sqlx::query_as::<_, EditRow>(
        "SELECT id, feature_key, scenario_index, request, before_json, after_json, status \
           FROM feature_doc_edits WHERE id = ? AND analysis_id = ?",
    )
    .bind(proposal)
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)
}

#[derive(sqlx::FromRow)]
struct EditRow {
    id: String,
    feature_key: String,
    scenario_index: i64,
    request: String,
    before_json: String,
    after_json: String,
    status: String,
}

/// 승인된 편집을 자동 문서 위에 겹쳐 읽는다.
///
/// 저장은 건드리지 않는다 — 문서를 내보내는 자리에서만 겹친다. 같은 자리에 승인이
/// 여러 번 쌓였으면 오래된 것부터 적용되므로 마지막 승인이 이긴다.
///
/// 자리를 찾지 못한 편집(문서가 그 사이 다시 쓰여 시나리오 수가 줄어든 경우)은
/// 조용히 건너뛴다. 그 상황은 자동 결과와 사용자 편집의 충돌이고, 어느 쪽을
/// 택할지는 AC3.5 가 사람에게 묻기로 한 문제라 여기서 임의로 정하지 않는다.
pub async fn overlay(
    state: &AppState,
    analysis_id: &str,
    doc: &mut Value,
) -> Result<(), AppError> {
    let rows: Vec<(String, i64, String)> = sqlx::query_as(
        "SELECT feature_key, scenario_index, after_json FROM feature_doc_edits \
          WHERE analysis_id = ? AND status = ? ORDER BY created_at, rowid",
    )
    .bind(analysis_id)
    .bind(status::APPROVED)
    .fetch_all(&state.db)
    .await?;

    for (key, index, after_json) in rows {
        let Ok(after) = serde_json::from_str::<Vec<Sentences>>(&after_json) else {
            continue;
        };
        let Some(features) = doc.get_mut("features").and_then(Value::as_array_mut) else {
            return Ok(());
        };
        let Some(feature) = features
            .iter_mut()
            .find(|f| f.get("key").and_then(Value::as_str) == Some(key.as_str()))
        else {
            continue;
        };
        let Some(list) = feature.get_mut("scenarios").and_then(Value::as_array_mut) else {
            continue;
        };
        let at = index as usize;
        if at >= list.len() {
            continue;
        }
        // 자리 하나가 여럿이 될 수 있으므로 갈아 끼우는 것이 아니라 그 구간을 바꾼다.
        // 출처·근거는 그 자리에 있던 값을 물려준다 — 사람이 고친 것은 문장이지,
        // 그 문장이 어느 코드에서 왔는지가 아니다.
        let template = list[at].clone();
        let replacement: Vec<Value> = after
            .iter()
            .map(|sentences| {
                let mut scenario = template.clone();
                scenario["given"] = json!(sentences.given);
                scenario["when"] = json!(sentences.when);
                scenario["then"] = json!(sentences.then);
                scenario
            })
            .collect();
        list.splice(at..at + 1, replacement);
    }
    Ok(())
}

/// 사람이 거부한 제안들, 최근 순으로 `(고쳐 쓴 then, 사유)`.
async fn rejections(
    state: &AppState,
    analysis_id: &str,
    feature_key: &str,
) -> Result<Vec<(String, String)>, AppError> {
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT after_json, reason FROM feature_doc_edits \
          WHERE analysis_id = ? AND feature_key = ? AND status = ? \
          ORDER BY decided_at DESC, rowid DESC",
    )
    .bind(analysis_id)
    .bind(feature_key)
    .bind(status::REJECTED)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(after_json, reason)| {
            let after = serde_json::from_str::<Vec<Sentences>>(&after_json).ok()?;
            // 회피의 단위는 「그때 내놓은 마지막 문장」이다 — 스텁·프롬프트가 함께 보는 값.
            let last = after.last()?.then.clone();
            Some((last, reason.unwrap_or_default()))
        })
        .collect())
}

/// 프롬프트와 스텁이 함께 보는 회피 목록.
fn avoid_list(rejected: &[(String, String)]) -> Vec<String> {
    rejected
        .iter()
        .take(MAX_AVOIDED)
        .map(|(then, _)| then.clone())
        .collect()
}

async fn document_of(state: &AppState, analysis_id: &str) -> Result<Value, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT content FROM analysis_documents WHERE analysis_id = ? AND kind = ?",
    )
    .bind(analysis_id)
    .bind(pipeline::ACCEPTANCE_DEPENDENCIES)
    .fetch_optional(&state.db)
    .await?;

    let content = row.ok_or(AppError::NotFound)?.0;
    let mut doc: Value = serde_json::from_str(&content)
        .map_err(|_| AppError::BadRequest("stored document is unreadable".into()))?;
    // 고칠 대상도, 그 위에 얹을 제안도 **사람이 지금 보는 문장** 기준이어야 한다 —
    // 사람이 직접 더한 feature(AC3.2)까지 포함해서.
    crate::feature_add::overlay(state, analysis_id, &mut doc).await?;
    overlay(state, analysis_id, &mut doc).await?;
    Ok(doc)
}

/// 그 feature 의 시나리오를 문서 순서대로. 편집은 자리로 지목되므로 순서가 곧 주소다.
fn scenarios_of(doc: &Value, key: &str) -> Result<Vec<Sentences>, AppError> {
    let feature = doc
        .get("features")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .find(|f| f.get("key").and_then(Value::as_str) == Some(key))
        .ok_or(AppError::NotFound)?;

    Ok(feature
        .get("scenarios")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(Sentences::from_value)
        .collect())
}

fn sentences_from_json(raw: &str) -> Result<Sentences, AppError> {
    serde_json::from_str(raw).map_err(|_| AppError::internal("stored edit is unreadable"))
}

fn scenarios_from_json(raw: &str) -> Result<Vec<Sentences>, AppError> {
    serde_json::from_str(raw).map_err(|_| AppError::internal("stored edit is unreadable"))
}

fn view(
    id: String,
    feature_key: String,
    scenario_index: usize,
    status: String,
    request: String,
    before: Sentences,
    after: Vec<Sentences>,
) -> ProposalView {
    let (removed, added) = lines(&before, &after);
    ProposalView {
        id,
        feature_key,
        scenario_index,
        status,
        source: SOURCE_USER_LLM.to_string(),
        request,
        before,
        after,
        changed_lines: removed.len() + added.len(),
        removed,
        added,
    }
}

/// 달라진 것만 줄로 낸다.
///
/// 제안이 한 건이면 **칸 단위**로 견준다 — 그 자리를 고쳐 쓴 것이므로 어느 문장이
/// 어떻게 바뀌었는지가 사람이 보고 싶은 것이다. 두 건 이상이면 **시나리오 단위**로
/// 견준다 — 자리 하나가 여럿이 된 것이라 칸을 짝지을 상대가 없고, 그때 사람이 보는
/// 것은 「무엇이 늘었는가」다(`diff.rs` 가 재분석 diff 에서 고른 축과 같다).
fn lines(before: &Sentences, after: &[Sentences]) -> (Vec<String>, Vec<String>) {
    let mut removed = Vec::new();
    let mut added = Vec::new();
    if let [only] = after {
        let pairs = [
            (&before.given, &only.given),
            (&before.when, &only.when),
            (&before.then, &only.then),
        ];
        for (old, new) in pairs {
            if old != new {
                removed.push(old.clone());
                added.push(new.clone());
            }
        }
        return (removed, added);
    }
    if !after.iter().any(|s| s == before) {
        removed.push(before.then.clone());
    }
    for scenario in after {
        if scenario != before {
            added.push(scenario.then.clone());
        }
    }
    (removed, added)
}

/// 모델이 답할 모양 — 그 자리에 설 시나리오들. 세 칸 밖의 것은 받지 않는다.
fn schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["scenarios"],
        "properties": {
            "scenarios": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["given", "when", "then"],
                    "properties": {
                        "given": { "type": "string" },
                        "when": { "type": "string" },
                        "then": { "type": "string" }
                    }
                }
            }
        }
    })
}

/// 답에서 세울 수 있는 시나리오만 꺼낸다. 하나도 읽히지 않으면 제안이 아니다.
fn proposed(answer: &Value) -> Option<Vec<Sentences>> {
    let list: Vec<Sentences> = answer
        .get("scenarios")?
        .as_array()?
        .iter()
        .filter_map(Sentences::from_value)
        .take(MAX_PROPOSED)
        .collect();
    (!list.is_empty()).then_some(list)
}

fn prompt(before: &Sentences, request: &str, avoid: &[String]) -> String {
    let mut text = String::new();
    text.push_str("Scenario as it stands now:\n");
    text.push_str("given: ");
    text.push_str(&before.given);
    text.push_str("\nwhen: ");
    text.push_str(&before.when);
    text.push_str("\nthen: ");
    text.push_str(&before.then);
    text.push_str("\n\nWhat the reader asked for:\n");
    text.push_str(request);
    if !avoid.is_empty() {
        text.push_str("\n\nDirections this reader already rejected — do not repeat them:\n");
        for rejected in avoid {
            text.push_str("- ");
            text.push_str(rejected);
            text.push('\n');
        }
    }
    text
}

/// 요청이 「사례를 하나 더」라고 말하는지 가르는 말. 스텁이 두 모양 중 어느 쪽으로
/// 답할지는 이 한 낱말이 정한다 — 실제 모델은 문장 전체를 읽고 스스로 고른다.
const ADD_HINT: &str = "추가";

/// 스텁의 답. 요청 문장을 결과 칸 뒤에 이어 **관측 가능한 변화 하나**를 만들고,
/// 「추가」를 부탁받았으면 원래 시나리오를 남긴 채 한 건을 더 세운다.
///
/// 회피는 스텁에서도 규칙이다 — 이미 거부된 문면과 같은 답이 나오면 한 겹 더 고쳐
/// 쓴다. 그러지 않으면 시나리오 2(거부와 회피)가 스텁의 우연으로 통과한다.
fn stub_edit(before: &Sentences, request: &str, avoid: &[String]) -> Value {
    let base = before.then.clone() + " — " + request;
    let mut then = base.clone();
    let mut round = 1;
    while avoid.iter().any(|rejected| rejected == &then) {
        round += 1;
        then = base.clone() + " (" + &round.to_string() + "번째로 다시 쓴 문장)";
    }
    let written = json!({ "given": before.given, "when": before.when, "then": then });
    let scenarios = if request.contains(ADD_HINT) {
        json!([
            { "given": before.given, "when": before.when, "then": before.then },
            written
        ])
    } else {
        json!([written])
    };
    json!({ "scenarios": scenarios })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_accepted_by_openai_strict_mode() {
        crate::llm::assert_strict_schema(&schema());
    }

    fn sentences() -> Sentences {
        Sentences {
            given: "카드가 없는 사용자가".to_string(),
            when: "만료된 카드를 저장하면".to_string(),
            then: "저장되지 않습니다".to_string(),
        }
    }

    fn one(answer: &Value) -> Vec<Sentences> {
        proposed(answer).expect("스텁 답은 읽혀야 한다")
    }

    #[test]
    fn only_changed_sentences_become_diff_lines() {
        let before = sentences();
        let after = Sentences {
            then: "저장되지 않고 이유를 알려 줍니다".to_string(),
            ..before.clone()
        };
        let (removed, added) = lines(&before, std::slice::from_ref(&after));
        assert_eq!(removed, vec![before.then.clone()]);
        assert_eq!(added, vec![after.then.clone()]);
    }

    #[test]
    fn an_unchanged_scenario_has_no_diff_lines() {
        let before = sentences();
        let (removed, added) = lines(&before, std::slice::from_ref(&before));
        assert!(removed.is_empty() && added.is_empty());
    }

    /// 사례가 늘어난 제안은 **더해진 것만** 줄로 낸다 — 남아 있는 시나리오는 지워지지 않는다.
    #[test]
    fn an_added_scenario_shows_only_the_new_line() {
        let before = sentences();
        let extra = Sentences {
            then: "만료 안내가 입력칸 아래에 나타납니다".to_string(),
            ..before.clone()
        };
        let (removed, added) = lines(&before, &[before.clone(), extra.clone()]);
        assert!(removed.is_empty(), "원래 시나리오가 그대로 남는데 제거로 세어졌다");
        assert_eq!(added, vec![extra.then]);
    }

    #[test]
    fn the_stub_answers_the_same_way_for_the_same_ask() {
        let before = sentences();
        assert_eq!(
            stub_edit(&before, "더 분명하게", &[]),
            stub_edit(&before, "더 분명하게", &[])
        );
    }

    /// 「추가」를 부탁하면 원래 시나리오가 남고 한 건이 더 선다(테스트 문서 03#시나리오 1).
    #[test]
    fn asking_for_another_case_keeps_the_original() {
        let before = sentences();
        let scenarios = one(&stub_edit(&before, "에러 케이스 1개 더 추가", &[]));
        assert_eq!(scenarios.len(), 2);
        assert_eq!(scenarios[0], before);
        assert_ne!(scenarios[1].then, before.then);
    }

    #[test]
    fn asking_for_a_rewording_replaces_the_one_scenario() {
        let scenarios = one(&stub_edit(&sentences(), "더 분명하게", &[]));
        assert_eq!(scenarios.len(), 1);
    }

    /// 거부된 문면이 다시 나오지 않는다 — 회피가 규칙이라는 것의 기계 확인.
    #[test]
    fn the_stub_avoids_a_rejected_answer() {
        let before = sentences();
        let rejected = one(&stub_edit(&before, "더 분명하게", &[]))[0].then.clone();
        let again = one(&stub_edit(&before, "더 분명하게", &[rejected.clone()]));
        assert_ne!(again[0].then, rejected);
    }

    #[test]
    fn the_prompt_carries_the_rejected_directions() {
        let text = prompt(&sentences(), "더 분명하게", &["오류 코드를 덧붙인 문장".to_string()]);
        assert!(text.contains("더 분명하게"));
        assert!(text.contains("오류 코드를 덧붙인 문장"));
    }

    #[test]
    fn a_prompt_without_rejections_says_nothing_about_them() {
        assert!(!prompt(&sentences(), "더 분명하게", &[]).contains("rejected"));
    }

    #[test]
    fn sentences_need_all_three_fields() {
        assert!(Sentences::from_value(&json!({ "given": "a", "when": "b", "then": "c" })).is_some());
        assert!(Sentences::from_value(&json!({ "given": "a", "when": "b" })).is_none());
    }

    /// 세울 시나리오가 하나도 없는 답은 제안이 아니다 — 문서를 비우는 승인이 되어서는 안 된다.
    #[test]
    fn an_empty_answer_is_not_a_proposal() {
        assert!(proposed(&json!({ "scenarios": [] })).is_none());
        assert!(proposed(&json!({})).is_none());
    }
}
