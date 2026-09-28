# 판정 상세 — 설치 범위 밖 공개 저장소 spec (1파일, 51차 패스)

- **판정일**: 2026-09-28
- **판정 범위**: `e2e/tests/sc01-09-public-repo-outside-install.spec.ts`
- **기준 트리**: **`2315e26`**(main tip, AC1.6 구현 #212 착지 직후). 이 범위의 줄 수·지문은 유입 시점과
  tip 이 같다(2 / `ae65d92d35443712f83418a01ea18525c81969b1d8b4c6840da7a532bbc41406`).
- **유입원**: PR **#212**(`2315e26`, docs-impl `rct_20260928-0015` — 01#시나리오 9 를 구현하며 이 spec 을
  신설했다). 정책 README 「판정하지 않은 주석을 들이는 PR」 규칙대로 행만 세우고 판정 축을 `—` 로 두었다.
- **reconciler task**: `tbm_feature-doc-comment-redundancy/rct_20260928-0007`
- **PR**: 이 패스의 PR (**51차 패스**)

규칙은 [../README.md](../README.md), 판정 결과의 표면은 [../ledger.md](../ledger.md)에 있다.

## 왜 이 행이 이 패스의 몫인가

원장이 스스로 인계했다 — 이 행의 결과 칸은 「**미판정** — … 판정은 다음 주석 비중복성 패스의 몫」이었고
판정 축이 `—` 다. 게이트가 그 잔여를 직접 센다:

```
c0d2126 (base) : [L] 판정 완료(①②③④) 2940줄 / 43행 (100% of 2940줄) · 축별 미판정 행 ① 0 · ② 0 · ③ 0 · ④ 0
2315e26 (tip)  : [L] 판정 완료(①②③④) 2940줄 / 43행 ( 99% of 2942줄) · 축별 미판정 행 ① 1 · ② 1 · ③ 1 · ④ 1
```

축이 `—` 인 행은 이 행 **하나뿐**이다. 게이트는 미판정을 계수만 하고 실패시키지 않으므로(`rc=0`)
이 축의 집행자는 감지다.

## 판정 — 전건 유지 (2행 → 2행 · 제거 0행)

파일 머리는 3줄이다. 1행 `// 검증 시나리오: 01-analysis-pipeline.md#시나리오 9` 는 정책 「유지 대상」의
**기계 판독** 주석이라 지문·판정 모두에서 제외되고, 판정 대상은 아래 2줄이다.

| 자리 | 문면 | 판정 | 근거 |
|---|---|---|---|
| `:2` (1) | `//` | **유지 1** | 기계 판독 헤더와 임대 고지를 가르는 **딸린 구분 줄**. 고지 바로 위에 빈 `//` 가 오는 형태가 임대 고지를 가진 spec **13개 전부**에 있고, `sc04-02`·`sc04-10` 행이 「딸린 빈 `//`」로 유지했다. 고지를 유지하는 한 따로 걷을 이유가 없다 |
| `:3` (1) | ``// Leases one API env var — lease rules in `e2e/support/cluster.ts`.`` | **유지 1** | 아래 네 경로와 공유 규약 절 |

### 임대 고지 — 네 경로

- **① 부분 복원** — 「one API env var」 는 본문의 `setApiEnv(REVOKED, HANDLE)`(`:85`)와 `finally` 의
  `setApiEnv(REVOKED, null)`(`:102`)이 보인다. 그러나 **규칙이 어디 있는가**(「자기 블록 안에서 세우고
  `finally` 에서 반납한다」의 소재)는 코드에 없다 — import 한 줄은 함수를 줄 뿐 규약을 가리키지 않는다.
- **② 규칙 자체는 히트, 이 줄은 포인터** — 임대 규칙은 `e2e/support/cluster.ts` 의 `setApiEnv` doc
  (「Deployment-wide state with the same lease rule as `setWorkerEnv`: set it inside the spec's own block,
  clear it in `finally`」)과 `docs/doc-tracker/2026-09.md:90` 「배포 전역 상태는 소유가 아니라 임대한다」가
  소유한다. 이 줄은 그것을 **재진술하지 않고 이름으로 가리킬 뿐**이라 중복 유형(선언 재진술·문서 인용)에
  들지 않는다.
- **③ 0히트** — PR #212 본문(3,493자)에 `lease`·`Lease`·`cluster.ts`·`setApiEnv`·`env var`·`임대` 전부 0회.
  최장 연속 일치는 ` API `(5자)다.
- **④ 공전** — 저작 커밋 `2315e26` 은 squash 이고 트레일러를 빼면 본문 0자다. 제목
  「설치 범위 밖 공개 저장소를 로그인 인가로 읽어 분석한다 — AC1.6 구현 (rct_20260928-0015) (#212)」과
  이 줄의 최장 연속 일치는 ` — ` 3자다.

### 공유 규약 — 한 축에서 혼자 지우면 첫 이탈

`grep -rlE '^// Leases .*lease rules in' e2e/tests` = **13파일**. 문면은 셋으로 갈린다 —
「the analysis worker」 11 · 「the analysis worker *and* one API env var」 1(`sc04-02`) · 「one API env var」
1(이 파일). 즉 이 줄은 규약 문형을 그대로 쓰고 **무엇을 빌리는가**만 이 spec 에 맞춰 적었다 —
정확하다(이 spec 은 `setApiEnv` 만 부르고 워커는 건드리지 않는다).

선례는 모두 유지다:

- `sc04-02` 행(바로 아래) — 「spec 10개가 공유하는 Isolation 규약이라 한 축에서 혼자 지우면 첫 이탈」
  ([2026-09-25-revocation-axis.md](2026-09-25-revocation-axis.md) 「유지 7행」). 「one API env var」 문면의 원형이다.
- `sc04-06` 행 — 「`Leases the analysis worker …` 는 e2e spec 10개에 바이트 동일한 공유 규약」.
- `sc04-09` 행 — 「임대 고지 2(e2e spec 공유 규약 — 한 축에서 혼자 지우면 첫 이탈)」.

이 규약을 뒤집으려면 13파일을 한 증분 재판정으로 묶어야 하고, 그것은 이 행 하나의 판정 범위가 아니다.

## 값

판정 **1행** · **유지 2행 · 제거 0행** · 판단 갈림 0건. 원장 행의 줄 수·지문은 편집 전과 바이트
동일(2 / `ae65d92d…`)이고 판정 축이 `—` → **`①②③④`** 가 된다.

이 패스로 L 표면 44행 전부가 `①②③④` 가 되어 판정 완료가 `2940줄 / 43행(99% of 2942줄)` →
**`2942줄 / 44행(100% of 2942줄)`**, D·E 표면은 100% 불변이다. 축별 미판정 행이 세 표면 모두 **0** 이다.

## 검증

- 편집 전 원장 기재값(2 / `ae65d92d…`)이 실측과 바이트 일치하는 것을 **먼저** 확인했다(게이트 `rc=0`).
- 게이트 자기 출력: `판정 대상 실측 lines=2942 files=165 unclassified=0` · `표면 D·E 실측 docstring=59/3
  eol=8/5 unparsed=0` — 편집 전과 바이트 동일(주석을 한 줄도 건드리지 않았다).
- **주석·코드 diff 0줄** — 접촉은 `docs/` 3파일뿐이다(원장 1행 제자리 수정 · 이 파일 · 허브 집계).
- 게이트 4종 전건 `rc=0`: `scripts/check-comment-ledger.py` · `tools/check-journey-mockup.py` ·
  `tools/check-mockup-render.py` · `tools/check-scenario-e2e.py`. 뒤의 둘은 출력이 편집 전과 **바이트
  동일**하고, `check-journey-mockup.py` 는 첫 두 줄의 문서 수·상대 링크 수만 움직인다.
- **허브 R9 동반 갱신** — `docs/` 에 새 `.md` 가 생기므로 `docs/index.html` 에 `doc-row` 1건을 더하고
  `Documents` 선언을 **64 → 65** 로 올렸다.
- 음성 프로브 3발 전건 발화 — ⒜ 판정 축을 `—` 로 되돌리면 축별 미판정이 `① 1 · ② 1 · ③ 1 · ④ 1` 로 복귀
  ⒝ 결과 칸의 패스 링크를 없는 파일로 바꾸면 R8 `rc=1` ⒞ 허브 선언을 64 로 되돌리면 R9 `rc=1`.

## 범위 밖 (다음 감지가 받는다)

- **열린 자매 PR #210**(Web Push, `tbm_feature-doc-docs-impl`)은 새 파일(`backend/src/push.rs` ·
  `frontend/public/sw.js` · `frontend/src/push.ts` · `0017_push_subscriptions.sql` 등)을 들이지만 원장을
  건드리지 않는다. 그 몫의 행은 착지 뒤 감지가 새 task 로 잇는다. 접촉 파일은 이 패스의 3파일과 **서로소**다.
- PR #212 본문이 「다음 주석 비중복성 패스의 입력」으로 적은 `backend/src/worker_api.rs` 의
  `installation_token` doc(이제 공개 경로에서 사용자 인가도 나른다)은 **판정된 행의 파일**이고 지문이
  움직이지 않아 이 모델의 gap 밖이다. 같은 결로 `e2e/support/cluster.ts` 의 `setApiEnv` doc 「Used by
  sc04-02 …」 는 이제 사용처가 둘인데 하나만 적는다(거짓은 아니고 불완전). 둘 다 문면 재판정이 필요하면
  별도 슬라이스다.
