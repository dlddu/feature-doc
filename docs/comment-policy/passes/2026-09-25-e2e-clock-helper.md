# 판정 상세 — e2e 시계 헬퍼 (1파일)

- **판정일**: 2026-09-25
- **판정 범위**: `e2e/support/clock.ts`
- **기준 트리**: 부모 **`908a6de`** (main, #163 `ci/path-filter` 착지 직후). 이 범위의 줄 수·지문은
  유입 시점 `67c15ad` 부터 `174a29b`(#161) · `908a6de`(#163) 까지 **세 지점에서 바이트 동일**하다
  (9 / `a0d46b59a983d30c34aaea1538e289c0882e09c04a9f466a0a8cfac109e39b33`).
- **유입원**: PR **#148**(`67c15ad`, `ci(e2e)`: 워커·테스트 폴링 단축 — 루프 밖 CI 속도 PR)
- **reconciler task**: `tbm_feature-doc-comment-redundancy/rct_20260925-0013`
- **PR**: (이 PR)

규칙은 [../README.md](../README.md), 판정 결과의 표면은 [../ledger.md](../ledger.md)에 있다.
같은 패스가 `#148` 이 원장 **행 5**(워커 · 더블 배선 축)에 연 **5행**을 함께 판정했고, 그쪽은 이미
행이 있는 범위라 상세가 [2026-09-18-worker-double-axis.md](2026-09-18-worker-double-axis.md)
「증분 재판정 ⑥」에 있다.

## 왜 새 행인가

이 파일은 `#148` 이 들여온 **신설 파일**이고 원장의 어느 행 목록에도 이름이 없다 — **행 열 밖 잔여**다.
원장 본문 「판정 절차」 3항의 「증분 재판정은 새 행을 만들지 않는다」는 *이미 행이 있는 범위*의 규칙이므로
여기서는 새 행과 새 패스 파일이 맞다(행 29 `sc04-06` · 행 30 `viewport.ts` · 행 31 `sc04-10` 이 같은
형태의 직전 선례다).

같은 디렉터리의 `e2e/support/cluster.ts` 가 원장 **행 3**(e2e 하네스 비경합 5파일)에 있으므로 그 행을
넓히는 선택지도 있었지만 **택하지 않았다** — 행 3 은 계획 시점에 **미판정 증분 29행**(145 기재 ↔ live 174)을
들고 있고 그 몫은 자매 PR **#162**(34차 패스, `rct_20260925-0012`)가 판정 중이다. 판정 중인 행에 파일을
얹으면 두 PR 이 같은 행의 줄 수·지문을 각자 다시 적어 **어느 쪽도 맞지 않는 값**이 남는다.

그 대가로 `docs/` 의 `.md` 가 하나 늘어 `tools/check-journey-mockup.py` 의 **R9**(허브 요약 `Documents`
선언 수 + 문서 링크 집합 ↔ `docs/` 실제 md 집합)가 깨진다. 계획 단계에서 게이트를 양방향으로 굴려
확인했다 — 이 파일만 더하면 정확히 2건(`Documents` 선언 불일치 · 허브↔파일 md 집합 불일치)으로 깨지고,
`docs/index.html` 에 `doc-row` 1건 + 카운트 1 을 넣으면 rc=0 으로 돌아온다.

## 판정 — 전건 유지 (9행, 제거 0)

```ts
/**                                                                    ← :1
 * Waits until the wall clock is past `unixSeconds`.                   ← :2
 *                                                                     ← :3
 * Stage `startedAt` is stored in whole unix seconds, and the specs that prove a
 * re-run do it by seeing `startedAt` change. With the stub LLM and a fast-polling
 * worker, the first run and the re-run can both start inside one second and read
 * back equal — the re-run happened, but the spec cannot see it. Triggering the
 * re-run only after that second has passed makes the comparison sound. The kind
 * cluster runs on this machine, so the two share one clock.           ← :4-9
 */
export async function afterSecond(unixSeconds: number): Promise<void> {
```

| 자리 | 문면(요약) | 판정 | 근거 |
|---|---|---|---|
| `:2` (1) | `Waits until the wall clock is past 'unixSeconds'.` | **유지 1** | 정책 본문이 명시 열거한 **「TS export 함수의 JSDoc 요약 1줄 유지」** 조항 대상이다. `afterSecond` 는 `export async function` 이고 두 spec(`sc01-06` · `sc01-08`)이 import 한다 |
| `:4-8` (5) | 「stage `startedAt` 이 **초 단위**라, stub LLM + 빠른 폴링에서는 첫 실행과 재실행이 같은 초 안에 시작해 값이 같게 읽힌다 — **재실행은 일어났는데 spec 이 그것을 볼 수 없다**. 그 초가 지난 뒤에 트리거해야 비교가 성립한다」 | **유지 5** | 「**이 순서를 바꾸면 무엇이 조용히 깨지는가**」형 가드다. `afterSecond` 호출을 지우거나 앞당기려는 손이 읽어야 하는 자리이고, 그렇게 하면 테스트는 **실패하지 않고 조용히 거짓 통과**한다. 복원 경로 ③ 이 **실재하지만**(#148 본문 §변경 3항이 같은 경위를 적는다) ③ 히트만으로는 뒤집지 않는다 — 아래 「③ 히트가 있는데 왜 유지인가」 |
| `:9` (1) | 「The kind cluster runs on this machine, so the two share one clock.」 | **유지 1** | 네 경로 **전부 부재**다(아래 실측). 「애매하면 남긴다」 이전에, 애매하지도 않다 |
| `:1` · `:3` (2) | `/**` · 빈 ` *` | **유지 2** | 위 일곱 줄을 남기는 이상 블록 구조다. 지문은 이 둘을 세므로 원장 줄 수 9 에 포함된다 |

### `:9` 의 네 경로 부재 — 실측

- **① 코드**: 레포 전수에서 이 명제는 `clock.ts:9` **자신 외 0히트**다. `waitMs` 계산은 `Date.now()`
  하나만 쓰고 클러스터가 같은 시계를 본다는 전제는 코드 어디에도 서 있지 않다 — 그런데 **그 전제가 깨지면
  이 헬퍼가 통째로 무의미**해진다(원격 클러스터로 옮기는 순간 벽시계 비교가 성립하지 않는다).
- **② 저장소 문서**: `docs/` 전수에 「같은 기계 / 시계 공유」 취지의 문장 0건.
- **③ PR 본문**: #148 본문 §변경 3항은 「초 단위라 재실행을 볼 수 없었다 → 그 초가 지나길 기다린다」
  까지만 적고 **kind 클러스터의 시계는 언급하지 않는다**.
- **④ 커밋 메시지**: `67c15ad` 의 메시지는 PR 제목 한 줄이다.

### ③ 히트가 있는데 왜 `:4-8` 이 유지인가

**이 레포는 「③ 이 실재해도 가드는 남긴다」를 이미 두 번 세웠다.** 둘 다 같은 잣대를 이 자리에 그대로
적용할 수 있는 형태다.

1. **같은 날 패스의 같은 모양** — [2026-09-25-residual-pool-closeout.md](2026-09-25-residual-pool-closeout.md)
   가 `backend/src/doc_history.rs:25-28` 을 「`seq` 가 시각이 아니라 순번인 이유 — 복원과 그 직후 편집이
   **같은 초**에 들어오면 앞뒤를 가릴 수 없고, 그 모호함이 「복원 뒤에 한 편집이 **조용히 사라진다**」로
   나타난다」로 두고, 복원 경로 **③(#126 본문)이 실재함을 적은 채** **유지 4행**으로 닫았다. 사유는
   「이 순서를 바꾸면 무엇이 조용히 깨지는가형 가드 — `seq` 를 `created_at` 으로 바꾸려는 손이 읽어야
   하는 자리」였다. **초 단위 해상도 때문에 관측이 무너진다**는 명제까지 같다.
2. **같은 디렉터리의 쌍둥이** — 원장 행 3 의 `e2e/support/cluster.ts:52-61` 이 「`kubectl scale` +
   `rollout status` 는 내려갈 때 충분하지 않다 … 그 창의 워커가 다음 단정이 볼 큐를 드레인한다」를
   **유지**로 닫았다(2026-09-18). 정책 본문이 유지 대상으로 이름 붙인 **「`kubectl scale`·rollout 이 pod
   정착 전에 돌아오는 것 같은 실패 모드의 함정」** 의 축자 예시가 그 주석이다. `clock.ts:4-8` 은 같은
   파일군·같은 형태의 함정이다.

반대로 이 자리에서 ③ 을 근거로 걷으면, 위 두 선례를 **함께 뒤집지 않고는 설명되지 않는다**. 원장이
정한 방식은 「같은 범위의 묶음 재판정」이지 **새 파일 한 건이 선례를 단독으로 뒤집는 것**이 아니다
(원장 2행의 17행 묶음 재판정 — 「이번 4행짜리 증분이 단독으로 선례를 뒤집는 자리가 아니다」 — 이
레포가 같은 상황에서 쓴 문장이다).

## 값

판정 9행 · **순 제거 0행 · 유지 9행**(전건 유지). 원장 새 행:
**9 / `a0d46b59a983d30c34aaea1538e289c0882e09c04a9f466a0a8cfac109e39b33`**.
행 열의 합이 **+9**, 행 열 밖 잔여 ⑴ 이 그만큼 **내려간다**.

「아무것도 지우지 않았으니 적을 것도 없다」가 아니다 — 원장에 행이 서지 않으면 다음 감지가 같은 9행을
다시 열고 **그 몫의 주인이 영영 생기지 않는다**(baseline 은 이미 전진해 있다). 이 행과 이 파일이 그
판정의 기록이다.

## 검증

- 이 범위의 **코드 diff 0** — `git diff --name-only` 에 `e2e/support/clock.ts` 가 나타나지 않는다.
- 행 지문은 원장 규약(개행 **포함** 해싱, `printf '%s\n'`)으로 쟀다. 전역 지문(모델
  `asIs.versionScript`)은 개행 없이 해싱하므로 값이 다르다 — 두 규약을 섞지 말 것.
- `#148` 창의 순 추가 14행 = 이 파일 **9** + `bin/worker.rs` **3** + `deploy/e2e/kustomization.yaml` **2**.
  뒤 5행은 「증분 재판정 ⑥」이 닫았고, 이 패스의 판정 대상은 그 14행뿐이다.

## 범위 밖으로 남긴 것 (다음 감지가 받는다)

- **`#161`(`174a29b`, 자매 `mockup-render` 패스)이 연 +8행** — 전역 2,859 → 2,867, 전부 행 열 *안*.
  이 task 의 baseline(2,859) 밖이라 **다음 감지 주기의 몫**이다.
- **`#162`(34차 패스)가 판정 중인 ⑴ 19(`sc04-02`) · ⑵ 105** — 그 PR 의 몫. 흡수하지 않았다.
