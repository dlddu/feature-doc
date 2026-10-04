# 쿼리 패턴 카탈로그 (Query Patterns)

`backend/src`의 쿼리 지점을 **형태**로 묶은 지도다. 형태 규칙과 추출 범위는 [README](README.md)가 정하고,
판정은 체커(`python3 tools/check-data-model.py`)가 한다. 코드와 어긋나면 이 문서를 고친다 — 쿼리의 존재
여부는 코드가 이긴다.

- 행은 형태마다 하나다. ID(`Q-01` …)는 사람이 부르기 위한 라벨일 뿐 귀속 키가 아니고, 지운 ID는 다시
  쓰지 않는다.
- 호출 지점은 `파일::함수`(메서드면 `파일::타입::메서드`)다. 한 함수가 같은 형태의 쿼리를 여러 번 내도
  지점은 한 번만 적는다.
- **지원 칸**: 접근 조건이 없는 쓰기(`insert`·`upsert`)는 `—`다. 나머지는 판정 슬라이스 (3)이 빈 DB
  쿼리 플랜으로 인덱스 라벨이나 `풀스캔 허용(F<n>): <근거>`를 채운다 — 그 전까지는 `판정 전`이다.
  「미사용 인덱스」 표도 그 슬라이스가 더한다.

## 패턴

| ID | 형태 | 지원 인덱스 또는 허용 사유 | 호출 지점 |
|---|---|---|---|
| Q-01 | `delete feature_dependencies \| eq(analysis_id, feature_key) \| - \| -` | 판정 전 | `backend/src/worker_api.rs::submit_dependencies` |
| Q-02 | `delete feature_doc_conflicts \| eq(analysis_id, status) \| - \| -` | 판정 전 | `backend/src/doc_conflict.rs::inherit` |
| Q-03 | `delete feature_doc_edits \| eq(analysis_id) \| - \| -` | 판정 전 | `backend/src/doc_conflict.rs::inherit` |
| Q-04 | `delete push_subscriptions \| eq(endpoint) \| - \| -` | 판정 전 | `backend/src/push.rs::send_to_user` |
| Q-05 | `delete sessions \| eq(id) \| - \| -` | 판정 전 | `backend/src/session.rs::delete` |
| Q-06 | `insert access_requests \| - \| - \| -` | — | `backend/src/access_request.rs::create` |
| Q-07 | `insert analyses \| - \| - \| -` | — | `backend/src/analysis.rs::create` |
| Q-08 | `insert analysis_stages \| - \| - \| -` | — | `backend/src/analysis.rs::create` |
| Q-09 | `insert audit_log \| - \| - \| -` | — | `backend/src/audit.rs::record` |
| Q-10 | `insert discovery_strategies \| - \| - \| -` | — | `backend/src/analysis.rs::strategy` |
| Q-11 | `insert feature_additions \| - \| - \| -` | — | `backend/src/feature_add.rs::draft` |
| Q-12 | `insert feature_candidates \| - \| - \| -` | — | `backend/src/analysis.rs::seed_candidates` |
| Q-13 | `insert feature_deletions \| - \| - \| -` | — | `backend/src/feature_delete.rs::delete` |
| Q-14 | `insert feature_dependencies \| - \| - \| -` | — | `backend/src/feature_add.rs::decide`, `backend/src/worker_api.rs::submit_dependencies` |
| Q-15 | `insert feature_dependency_requests \| - \| - \| -` | — | `backend/src/feature_add.rs::decide` |
| Q-16 | `insert feature_doc_conflicts \| - \| - \| -` | — | `backend/src/doc_conflict.rs::inherit` |
| Q-17 | `insert feature_doc_edits \| - \| - \| -` | — | `backend/src/doc_conflict.rs::insert_edit`, `backend/src/doc_edit.rs::propose` |
| Q-18 | `insert feature_doc_restores \| - \| - \| -` | — | `backend/src/doc_history.rs::restore` |
| Q-19 | `insert installations \| - \| - \| -` | — | `backend/src/installations.rs::upsert` |
| Q-20 | `insert llm_keys \| - \| - \| -` | — | `backend/src/llmkey.rs::register` |
| Q-21 | `insert sessions \| - \| - \| -` | — | `backend/src/session.rs::create` |
| Q-22 | `insert users \| - \| - \| -` | — | `backend/src/users.rs::upsert` |
| Q-23 | `select analyses \| eq(branch=branch, id, repo_name=repo_name, repo_owner=repo_owner, user_id, user_id=user_id) \| - \| order(created_at desc, rowid desc)` | 판정 전 | `backend/src/analysis.rs::diff_view` |
| Q-24 | `select analyses \| eq(claimed_by, id, status) \| range(lease_expires_at) \| -` | 판정 전 | `backend/src/worker_api.rs::require_lease` |
| Q-25 | `select analyses \| eq(id) \| - \| -` | 판정 전 | `backend/src/analysis.rs::load_detail`, `backend/src/push.rs::notify_stage`, `backend/src/settings.rs::analysis_language`, `backend/src/worker_api.rs::heartbeat` |
| Q-26 | `select analyses \| eq(id, user_id) \| - \| -` | 판정 전 | `backend/src/analysis.rs::owned_analysis`, `backend/src/analysis.rs::retry_stage` |
| Q-27 | `select analyses \| eq(public_repo, repo_name, repo_owner, user_id) \| - \| -` | 판정 전 | `backend/src/analysis.rs::was_public` |
| Q-28 | `select analyses,analysis_documents \| eq(analyses.branch=analyses.branch, analyses.id, analyses.id=analysis_documents.analysis_id, analyses.repo_name=analyses.repo_name, analyses.repo_owner=analyses.repo_owner, analyses.user_id, analyses.user_id=analyses.user_id, analysis_documents.kind) \| - \| order(analyses.created_at desc, analyses.rowid desc)` | 판정 전 | `backend/src/analysis.rs::document` |
| Q-29 | `select analyses,analysis_documents \| eq(analyses.branch=analyses.branch, analyses.id, analyses.id=analysis_documents.analysis_id, analyses.repo_name=analyses.repo_name, analyses.repo_owner=analyses.repo_owner, analyses.user_id=analyses.user_id, analysis_documents.kind) \| - \| order(analyses.created_at desc, analyses.rowid desc)` | 판정 전 | `backend/src/doc_conflict.rs::previous_documented` |
| Q-30 | `select analyses,analysis_documents \| eq(analyses.id=analysis_documents.analysis_id, analyses.user_id, analysis_documents.analysis_id, analysis_documents.kind) \| - \| -` | 판정 전 | `backend/src/analysis.rs::document` |
| Q-31 | `select analyses,analysis_documents,feature_additions,feature_dependency_requests,feature_doc_edits \| eq(analyses.id=analysis_documents.analysis_id, analyses.id=feature_additions.analysis_id, analyses.id=feature_dependency_requests.analysis_id, analyses.id=feature_doc_edits.analysis_id, analyses.user_id) \| - \| order(analyses.created_at desc, analyses.id desc)` | 판정 전 | `backend/src/usage.rs::read` |
| Q-32 | `select analyses,analysis_stages,feature_candidates \| eq(analyses.id, analyses.id=analysis_stages.analysis_id, analyses.id=feature_candidates.analysis_id, analyses.user_id, analysis_stages.status, feature_candidates.decision, feature_candidates.merged_into) \| - \| -` | 판정 전 | `backend/src/analysis.rs::load_detail` |
| Q-33 | `select analyses,analysis_stages,feature_candidates \| eq(analyses.id=analysis_stages.analysis_id, analyses.id=feature_candidates.analysis_id, analyses.user_id, analysis_stages.status, feature_candidates.decision, feature_candidates.merged_into) \| - \| order(analyses.created_at desc, analyses.id desc)` | 판정 전 | `backend/src/analysis.rs::list` |
| Q-34 | `select analyses,discovery_strategies \| eq(analyses.branch, analyses.id, analyses.id=discovery_strategies.analysis_id, analyses.repo_name, analyses.repo_owner, analyses.user_id) \| - \| order(analyses.created_at desc, analyses.rowid desc)` | 판정 전 | `backend/src/analysis.rs::carried_over` |
| Q-35 | `select analyses,feature_candidates \| eq(analyses.branch, analyses.id, analyses.id=feature_candidates.analysis_id, analyses.repo_name, analyses.repo_owner, analyses.user_id, feature_candidates.decision, feature_candidates.key) \| - \| order(analyses.created_at desc, analyses.rowid desc)` | 판정 전 | `backend/src/analysis.rs::previous_rejection` |
| Q-36 | `select analyses,feature_candidates,feature_dependencies \| eq(analyses.id=feature_dependencies.analysis_id, analyses.user_id, feature_candidates.analysis_id=feature_dependencies.analysis_id, feature_candidates.key=feature_dependencies.feature_key, feature_dependencies.category, feature_dependencies.name) \| - \| order(analyses.created_at desc, feature_dependencies.analysis_id asc, feature_dependencies.feature_key asc)` | 판정 전 | `backend/src/analysis.rs::dependents` |
| Q-37 | `select analyses,feature_deletions \| eq(analyses.branch, analyses.id, analyses.id=feature_deletions.analysis_id, analyses.repo_name, analyses.repo_owner, analyses.user_id, feature_deletions.feature_key, feature_deletions.restored_at) \| - \| order(analyses.created_at desc, analyses.rowid desc)` | 판정 전 | `backend/src/feature_delete.rs::previous_deletion` |
| Q-38 | `select analysis_documents \| eq(analysis_id) \| - \| -` | 판정 전 | `backend/src/usage.rs::by_stage` |
| Q-39 | `select analysis_documents \| eq(analysis_id, kind) \| - \| -` | 판정 전 | `backend/src/analysis.rs::acceptance_document`, `backend/src/analysis.rs::proposed_patterns`, `backend/src/analysis.rs::seed_candidates`, `backend/src/doc_conflict.rs::raw_document`, `backend/src/doc_edit.rs::base_document`, `backend/src/doc_history.rs::baseline_at`, `backend/src/evidence.rs::read`, `backend/src/feature_delete.rs::current_document`, `backend/src/worker_api.rs::acceptance_pending`, `backend/src/worker_api.rs::stored_landscape` |
| Q-40 | `select analysis_documents,feature_additions,feature_dependency_requests,feature_doc_edits \| eq(analysis_documents.analysis_id, feature_additions.analysis_id, feature_dependency_requests.analysis_id, feature_doc_edits.analysis_id) \| - \| -` | 판정 전 | `backend/src/usage.rs::of_analysis` |
| Q-41 | `select analysis_stages \| eq(analysis_id) \| - \| -` | 판정 전 | `backend/src/worker_api.rs::offered_stages` |
| Q-42 | `select analysis_stages \| eq(analysis_id) \| - \| order(seq asc)` | 판정 전 | `backend/src/analysis.rs::load_detail` |
| Q-43 | `select audit_log \| eq(user_id) \| - \| order(created_at desc, id desc)` | 판정 전 | `backend/src/audit.rs::list` |
| Q-44 | `select discovery_strategies \| eq(analysis_id) \| - \| -` | 판정 전 | `backend/src/analysis.rs::load_strategy`, `backend/src/worker_api.rs::approved_patterns` |
| Q-45 | `select feature_additions \| eq(analysis_id) \| - \| order(created_at asc, rowid asc)` | 판정 전 | `backend/src/feature_add.rs::rows_of` |
| Q-46 | `select feature_additions \| eq(analysis_id, id) \| - \| -` | 판정 전 | `backend/src/feature_add.rs::row_of` |
| Q-47 | `select feature_additions \| eq(analysis_id, key, status) \| - \| -` | 판정 전 | `backend/src/feature_add.rs::confirmed_name` |
| Q-48 | `select feature_candidates \| eq(analysis_id) \| - \| order(seq asc, rowid asc)` | 판정 전 | `backend/src/analysis.rs::candidate_locations`, `backend/src/analysis.rs::load_candidates` |
| Q-49 | `select feature_candidates \| eq(analysis_id, decision, key, merged_into) \| - \| -` | 판정 전 | `backend/src/analysis.rs::approved_candidate_name` |
| Q-50 | `select feature_candidates \| eq(analysis_id, decision, merged_into) \| - \| -` | 판정 전 | `backend/src/feature_add.rs::list` |
| Q-51 | `select feature_candidates \| eq(analysis_id, decision, merged_into) \| - \| order(seq asc, rowid asc)` | 판정 전 | `backend/src/worker_api.rs::approved_candidates` |
| Q-52 | `select feature_candidates \| eq(analysis_id, key, merged_into) \| - \| -` | 판정 전 | `backend/src/analysis.rs::merge_candidates` |
| Q-53 | `select feature_candidates,feature_dependencies \| eq(feature_candidates.analysis_id=feature_dependencies.analysis_id, feature_candidates.key=feature_dependencies.feature_key, feature_dependencies.analysis_id) \| - \| order(feature_dependencies.feature_key asc, feature_dependencies.seq asc, feature_dependencies.rowid asc)` | 판정 전 | `backend/src/analysis.rs::export_dependencies` |
| Q-54 | `select feature_candidates,feature_dependency_requests \| eq(feature_candidates.analysis_id=feature_dependency_requests.analysis_id, feature_candidates.decision, feature_candidates.key=feature_dependency_requests.feature_key, feature_candidates.merged_into, feature_dependency_requests.analysis_id, feature_dependency_requests.status) \| - \| order(feature_candidates.seq asc, feature_candidates.rowid asc)` | 판정 전 | `backend/src/worker_api.rs::pending_dependency_requests` |
| Q-55 | `select feature_deletions \| eq(analysis_id, id) \| - \| -` | 판정 전 | `backend/src/feature_delete.rs::row_of` |
| Q-56 | `select feature_deletions \| eq(analysis_id, restored_at) \| - \| order(deleted_at desc, rowid desc)` | 판정 전 | `backend/src/feature_delete.rs::open_rows` |
| Q-57 | `select feature_dependencies \| eq(analysis_id, feature_key) \| - \| order(seq asc, rowid asc)` | 판정 전 | `backend/src/analysis.rs::load_feature_dependencies`, `backend/src/analysis.rs::traced_dependencies` |
| Q-58 | `select feature_dependency_requests \| eq(analysis_id, feature_key) \| - \| -` | 판정 전 | `backend/src/analysis.rs::load_feature_dependencies`, `backend/src/analysis.rs::traced_dependencies` |
| Q-59 | `select feature_doc_conflicts \| eq(analysis_id) \| - \| order(created_at asc, rowid asc)` | 판정 전 | `backend/src/doc_conflict.rs::list` |
| Q-60 | `select feature_doc_conflicts \| eq(analysis_id, id) \| - \| -` | 판정 전 | `backend/src/doc_conflict.rs::conflict_row` |
| Q-61 | `select feature_doc_conflicts \| eq(analysis_id, status) \| - \| -` | 판정 전 | `backend/src/push.rs::notify_stage` |
| Q-62 | `select feature_doc_conflicts \| eq(analysis_id, status) \| - \| order(created_at asc, rowid asc)` | 판정 전 | `backend/src/doc_conflict.rs::inherited_from` |
| Q-63 | `select feature_doc_edits \| eq(analysis_id, feature_key, status) \| - \| order(decided_at desc, rowid desc)` | 판정 전 | `backend/src/doc_edit.rs::rejections` |
| Q-64 | `select feature_doc_edits \| eq(analysis_id, id) \| - \| -` | 판정 전 | `backend/src/doc_edit.rs::edit_row` |
| Q-65 | `select feature_doc_edits \| eq(analysis_id, id, status) \| - \| -` | 판정 전 | `backend/src/doc_conflict.rs::pending_proposal` |
| Q-66 | `select feature_doc_edits \| eq(analysis_id, status) \| - \| order(created_at asc, rowid asc)` | 판정 전 | `backend/src/doc_conflict.rs::inherited_from`, `backend/src/doc_edit.rs::overlay`, `backend/src/doc_history.rs::approved_edits` |
| Q-67 | `select feature_doc_edits \| eq(analysis_id, status) \| - \| order(feature_key asc)` | 판정 전 | `backend/src/doc_edit.rs::reviewed` |
| Q-68 | `select feature_doc_edits \| eq(id) \| - \| -` | 판정 전 | `backend/src/doc_conflict.rs::view` |
| Q-69 | `select feature_doc_restores \| eq(analysis_id) \| - \| order(feature_key asc, seq asc)` | 판정 전 | `backend/src/doc_history.rs::restores` |
| Q-70 | `select feature_doc_restores \| eq(analysis_id, feature_key) \| - \| -` | 판정 전 | `backend/src/doc_history.rs::restore` |
| Q-71 | `select feature_doc_restores \| eq(analysis_id, feature_key) \| - \| order(seq desc)` | 판정 전 | `backend/src/doc_history.rs::current_restore` |
| Q-72 | `select feature_doc_restores \| eq(analysis_id, id) \| - \| -` | 판정 전 | `backend/src/doc_history.rs::restore` |
| Q-73 | `select installations \| eq(installation_id, user_id) \| - \| -` | 판정 전 | `backend/src/installations.rs::upsert` |
| Q-74 | `select installations \| eq(user_id) \| - \| order(created_at desc)` | 판정 전 | `backend/src/installations.rs::get_for_user` |
| Q-75 | `select llm_keys \| eq(status, user_id) \| - \| order((provider = 'openai') desc, created_at desc)` | 판정 전 | `backend/src/llmkey.rs::active_key_for_user`, `backend/src/llmkey.rs::preflight` |
| Q-76 | `select llm_keys \| eq(user_id) \| - \| order(created_at desc)` | 판정 전 | `backend/src/llmkey.rs::list` |
| Q-77 | `select push_subscriptions \| eq(user_id) \| - \| -` | 판정 전 | `backend/src/push.rs::send_to_user` |
| Q-78 | `select sessions,users \| eq(sessions.id, sessions.user_id=users.id) \| range(sessions.expires_at) \| -` | 판정 전 | `backend/src/session.rs::lookup_user` |
| Q-79 | `select users \| eq(github_id) \| - \| -` | 판정 전 | `backend/src/users.rs::upsert` |
| Q-80 | `select users \| eq(id) \| - \| -` | 판정 전 | `backend/src/analysis.rs::can_install_on`, `backend/src/github_app.rs::stub_auth_revoked`, `backend/src/settings.rs::llm_language` |
| Q-81 | `update analyses \| eq(claimed_by, id) \| - \| -` | 판정 전 | `backend/src/worker_api.rs::finish` |
| Q-82 | `update analyses \| eq(claimed_by, id, status) \| - \| -` | 판정 전 | `backend/src/worker_api.rs::heartbeat` |
| Q-83 | `update analyses \| eq(id) \| - \| -` | 판정 전 | `backend/src/analysis.rs::requeue`, `backend/src/analysis.rs::retry_stage`, `backend/src/worker_api.rs::stop_for_revoked_access` |
| Q-84 | `update analyses \| eq(id) \| - \| order(created_at asc, id asc)` | 판정 전 | `backend/src/worker_api.rs::claim` |
| Q-85 | `update analyses \| eq(id, status) \| - \| -` | 판정 전 | `backend/src/analysis.rs::cancel` |
| Q-86 | `update analysis_stages \| eq(analysis_id, key) \| - \| -` | 판정 전 | `backend/src/worker_api.rs::report_stage` |
| Q-87 | `update analysis_stages \| eq(analysis_id, key, status) \| - \| -` | 판정 전 | `backend/src/analysis.rs::retry_stage` |
| Q-88 | `update analysis_stages \| eq(analysis_id, status) \| - \| -` | 판정 전 | `backend/src/analysis.rs::cancel`, `backend/src/worker_api.rs::stop_for_revoked_access` |
| Q-89 | `update discovery_strategies \| eq(analysis_id) \| - \| -` | 판정 전 | `backend/src/analysis.rs::approve_strategy`, `backend/src/analysis.rs::update_strategy` |
| Q-90 | `update feature_additions \| eq(id) \| - \| -` | 판정 전 | `backend/src/feature_add.rs::decide` |
| Q-91 | `update feature_candidates \| eq(analysis_id, key, merged_into) \| - \| -` | 판정 전 | `backend/src/analysis.rs::decide_candidate`, `backend/src/analysis.rs::merge_candidates`, `backend/src/analysis.rs::rename_candidate` |
| Q-92 | `update feature_deletions \| eq(id, restored_at) \| - \| -` | 판정 전 | `backend/src/feature_delete.rs::restore` |
| Q-93 | `update feature_dependency_requests \| eq(analysis_id, feature_key) \| - \| -` | 판정 전 | `backend/src/worker_api.rs::submit_dependencies` |
| Q-94 | `update feature_doc_conflicts \| eq(id) \| - \| -` | 판정 전 | `backend/src/doc_conflict.rs::decide`, `backend/src/doc_conflict.rs::decide_merge`, `backend/src/doc_conflict.rs::merge` |
| Q-95 | `update feature_doc_edits \| eq(id) \| - \| -` | 판정 전 | `backend/src/doc_conflict.rs::decide_merge`, `backend/src/doc_conflict.rs::merge`, `backend/src/doc_edit.rs::decide` |
| Q-96 | `update installations \| eq(id) \| - \| -` | 판정 전 | `backend/src/installations.rs::upsert` |
| Q-97 | `update llm_keys \| eq(id, status, user_id) \| - \| -` | 판정 전 | `backend/src/llmkey.rs::revoke` |
| Q-98 | `update users \| eq(id) \| - \| -` | 판정 전 | `backend/src/settings.rs::update`, `backend/src/users.rs::upsert` |
| Q-99 | `upsert analysis_documents \| eq(analysis_id, kind) \| - \| -` | — | `backend/src/worker_api.rs::submit_document` |
| Q-100 | `upsert feature_dependency_requests \| eq(analysis_id, feature_key) \| - \| -` | — | `backend/src/analysis.rs::request_dependencies` |
| Q-101 | `upsert github_tokens \| eq(user_id) \| - \| -` | — | `backend/src/github_tokens.rs::store` |
| Q-102 | `upsert push_subscriptions \| eq(endpoint) \| - \| -` | — | `backend/src/push.rs::subscribe` |

## 수동 형태

체커가 정적으로 형태를 뽑지 못한 지점이다. 형태는 사람이 읽어 위 패턴 표에 적고, 대표 SQL은 코드가
실제로 내는 SQL(자리표시자 `?`)이라 불변식 3의 플랜이 이 SQL로 돈다. 체커가 추출 불가로 보고하는 지점
집합과 이 표의 지점 집합은 같아야 한다.

| 호출 지점 | 패턴 ID | 대표 SQL | 추출 불가 사유 |
|---|---|---|---|
| `backend/src/usage.rs::of_analysis` | Q-40 | `SELECT COALESCE(SUM(calls), 0), COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0) FROM (SELECT analysis_id, calls, input_tokens, output_tokens FROM analysis_documents UNION ALL SELECT analysis_id, 1, input_tokens, output_tokens FROM feature_dependency_requests WHERE model IS NOT NULL UNION ALL SELECT analysis_id, 1, input_tokens, output_tokens FROM feature_doc_edits WHERE model IS NOT NULL UNION ALL SELECT analysis_id, 1, input_tokens, output_tokens FROM feature_additions WHERE model IS NOT NULL) WHERE analysis_id = ?` | `FROM`/`JOIN` 의 파생 테이블 — `{CALL_ROWS}`(네 테이블의 `UNION ALL`)를 보간한 서브쿼리라 바깥 조건의 컬럼이 한 테이블에 귀속되지 않는다 |
| `backend/src/usage.rs::read` | Q-31 | `SELECT a.id AS analysis_id, a.repo_owner, a.repo_name, a.branch, a.status, a.created_at, COALESCE(SUM(u.calls), 0) AS llm_calls, COALESCE(SUM(u.input_tokens), 0) AS input_tokens, COALESCE(SUM(u.output_tokens), 0) AS output_tokens, 0 AS cost_cents FROM analyses a LEFT JOIN (SELECT analysis_id, calls, input_tokens, output_tokens FROM analysis_documents UNION ALL SELECT analysis_id, 1, input_tokens, output_tokens FROM feature_dependency_requests WHERE model IS NOT NULL UNION ALL SELECT analysis_id, 1, input_tokens, output_tokens FROM feature_doc_edits WHERE model IS NOT NULL UNION ALL SELECT analysis_id, 1, input_tokens, output_tokens FROM feature_additions WHERE model IS NOT NULL) u ON u.analysis_id = a.id WHERE a.user_id = ? GROUP BY a.id ORDER BY a.created_at DESC, a.id DESC` | `FROM`/`JOIN` 의 파생 테이블 — `{CALL_ROWS}`(네 테이블의 `UNION ALL`)를 보간한 서브쿼리라 바깥 조건의 컬럼이 한 테이블에 귀속되지 않는다 |
