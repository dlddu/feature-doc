// 검증 시나리오: 01-analysis-pipeline.md#시나리오 6
//
// 「특정 단계 실패 후 부분 재시도」 전용 spec (AC1.5).
//
// docs/test/01-analysis-pipeline.md 시나리오 6을 그대로 따라간다:
//   feature 추출 단계가 실패한 상태에서 실패 화면의 "이 단계만 다시 시도"를 누르면,
//   횡단 분석/탐색 전략은 그대로 유지되고 그 단계만 재실행된다.
//
// 이 단정은 `sc01-05-resume-after-app-exit.spec.ts`와 한 파일에 있었다(선언은
// 시나리오 5 하나) — 규칙 2 상 분리 대상으로 등재됐다가 이 파일로 옮겨왔다.
// 도달성은 제품 표면만으로 충분하다: 브랜치는 Connect Repository의 사용자 입력
// 필드라, 존재하지 않는 브랜치를 입력하면 fetch 단계가 실제 GitHub 트리 요청과 같은
// 이유(404)로 실패한다 — 테스트 전용 훅이 필요 없다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. The residual effect of a running worker is that it drains
// every queued job, including ones other specs left behind — so every assertion
// below is about a job this spec created.
//
// Like every spec it signs in as its own stub user (`?as=sc0106`). Reaching Home
// requires an App installation and an active LLM key; those are set up through the
// API rather than Credentials Setup's screens, which sc04-01/sc04-03 own — walking
// another scenario's screen is setup, not this file's verification target.
//
// The second declaration closes blocker-ledger R1 (docs/e2e-mocking-policy.md).
// 시나리오 6 원문의 사전 조건은 「feature 추출 단계가 LLM 호출 한도 초과로 실패한
// 상태」인데, stub LLM은 실패를 표현하지 못해 위 arc는 트리 404로 같은 실패 분기를
// 밟는 우회로 시작했다. 이제 stub 모드가 결정적 실패 트리거를 갖추었다
// (FEATUREDOC_STUB_LLM_FAIL — 프롬프트 부분열 일치 시 real과 같은 429 사유로 실패;
// backend/src/llm.rs). 이것은 새 제품 표면이 아니라 등재된 stub 더블(LLM-01) 자체의
// 충실도 확장이다. 두 번째 선언이 그 사전 조건을 그대로 재현하며, 트리거 env는
// 임대 창 안에서만 실재한다(setWorkerEnv: scale 0 뒤 set, finally에서 unset).
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers, setWorkerEnv } from '../support/cluster';

/** Stage keys seeded at enqueue, in pipeline order (backend/src/pipeline.rs). */
const LATER_STAGES = [
  'cross_cutting',
  'discovery_strategy',
  'feature_candidates',
  'acceptance_dependencies',
];

type StageRow = {
  key: string;
  status: string;
  detail: string | null;
  error: string | null;
  startedAt: number | null;
};

async function enqueue(page: Page, repo: string, branch: string | null): Promise<string> {
  const res = await page.request.post('/api/analyses', {
    data: { repoUrl: `stub-account/${repo}`, branch },
  });
  expect(res.status(), `enqueue ${repo}@${branch ?? 'default'}`).toBe(201);
  return (await res.json()).id as string;
}

async function analysisOf(page: Page, id: string): Promise<{ status: string; stages: StageRow[] }> {
  const res = await page.request.get(`/api/analyses/${id}`);
  expect(res.ok()).toBeTruthy();
  return await res.json();
}

async function stageOf(page: Page, id: string, key: string): Promise<StageRow> {
  const stage = (await analysisOf(page, id)).stages.find((s) => s.key === key);
  expect(stage, `analysis ${id} has no ${key} stage`).toBeTruthy();
  return stage!;
}

test.describe('시나리오 6: 특정 단계 실패 후 부분 재시도', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('실패한 단계와 그 사유가 보이고, 재시도는 그 단계만 다시 실행한다', async ({ page }) => {
    try {
      // The overlay already rests at 0; make the precondition explicit so a queued
      // job cannot drain before the setup below.
      await scaleWorkers(0);

      // ── setup: this spec's own user, App installation, LLM key ──────────
      await page.goto('/api/auth/login?as=sc0106');
      expect((await page.request.get('/api/github/setup?installation_id=4242')).ok()).toBeTruthy();
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'anthropic', key: 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa' },
      });
      expect(key.ok(), 'an active LLM key is 분석의 진입 조건').toBeTruthy();

      // 시나리오의 실패 상태를 만들 분석: 존재하지 않는 브랜치 → fetch가 실패한다.
      // 격리 대조용으로 스텁이 받을 수 있는 정상 분석 하나를 함께 둔다 — 「그 단계만」
      // 의 반대편(다른 분석이 재시도에 휘말리지 않음)은 이 둘의 관계로 관측한다.
      const good = await enqueue(page, 'payments-api', null);
      const failing = await enqueue(page, 'checkout-web', 'no-such-branch');

      // ── 워커가 둘 다 처리한다 ────────────────────────────────────────────
      await scaleWorkers(1);
      await expect
        .poll(() => analysisOf(page, good).then((a) => a.status), {
          timeout: 120_000,
          intervals: [1_000],
        })
        .toBe('awaiting_pipeline');
      await expect
        .poll(() => analysisOf(page, failing).then((a) => a.status), {
          timeout: 120_000,
          intervals: [1_000],
        })
        .toBe('failed');

      // ── 실패한 단계와 그 사유 ───────────────────────────────────────────
      await page.goto(`/#/analyses/${failing}`);
      const failedStage = page.locator('[data-stage="fetch"]');
      await expect(failedStage).toContainText('github tree rejected (404)');
      await expect(page.getByTestId('pipeline-count')).toHaveText('0 of 5');

      const beforeRetry = await stageOf(page, failing, 'fetch');
      expect(beforeRetry.startedAt).not.toBeNull();

      // ── 「이 단계만 다시 시도」 ──────────────────────────────────────────
      await failedStage.getByTestId('retry').click();

      // The reset is observed through the API rather than the DOM on purpose: a
      // worker is running, so the "waiting" render lasts only until it re-claims
      // (~2s) — asserting on that frame would be a race. What matters is below.

      // The job really re-runs: a *new* attempt, with the same deterministic cause
      // (the branch still does not exist), not the old record left in place.
      //
      // Both halves are polled together on purpose. `startedAt` alone is not the
      // signal: the retry *clears* it, so "changed from the old value" is satisfied
      // by the reset itself, a second before a worker has touched the job.
      await expect
        .poll(
          async () => {
            const s = await stageOf(page, failing, 'fetch');
            const reran = s.startedAt !== null && s.startedAt !== beforeRetry.startedAt;
            return `${s.status}${reran ? ' (reran)' : ''}`;
          },
          {
            message: 'the retried stage should run again and fail on the same cause',
            timeout: 120_000,
            intervals: [1_000],
          },
        )
        .toBe('failed (reran)');
      const afterRetry = await stageOf(page, failing, 'fetch');
      expect(afterRetry.error).toContain('404');

      // ── 「부분」 재시도: 실행되지 않은 단계는 그대로, 다른 분석도 그대로 ──
      const stages = (await analysisOf(page, failing)).stages;
      for (const key of LATER_STAGES) {
        expect(stages.find((s) => s.key === key)?.status, `${key} must be untouched`).toBe(
          'pending',
        );
      }
      expect((await stageOf(page, good, 'fetch')).status).toBe('succeeded');
      expect((await stageOf(page, good, 'fetch')).detail).toBe('766 files · 2.2 MB');
    } finally {
      // Back to the overlay's resting state, whatever happened above.
      await scaleWorkers(0);
    }
  });

  // 원장 R1의 arc: 시나리오 6 원문이 이름 붙인 실패 원인(LLM 호출 한도 초과)은
  // 승인이 열어 주는 단계(feature_candidates)에서 착지한다. 트리거 needle이
  // stage-4 프롬프트에만 실재하는 문구라 단계 1-3의 ask에는 발화하지 않는다 —
  // 실패가 정확히 원문이 지목한 단계에 착지하고, 기대 결과의 「횡단 분석·탐색 전략
  // 유지」가 실제 성공 상태로 검증된다.
  test('LLM 한도 초과로 실패한 단계의 사유가 보이고, 재시도는 그 단계만 다시 실행한다', async ({
    page,
  }) => {
    try {
      // The overlay rests at 0 (the sibling arc's finally put it back). Set the
      // failure trigger while no pod runs, so only pods born inside this lease
      // window carry it.
      await scaleWorkers(0);
      setWorkerEnv('FEATUREDOC_STUB_LLM_FAIL', 'end-user feature candidates');

      // ── setup: this spec's own user, App installation, LLM key ──────────
      await page.goto('/api/auth/login?as=sc0106');
      expect((await page.request.get('/api/github/setup?installation_id=4242')).ok()).toBeTruthy();
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'anthropic', key: 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa' },
      });
      expect(key.ok(), 'an active LLM key is 분석의 진입 조건').toBeTruthy();

      // 시나리오 6 원문의 사전 조건을 만들 분석: fetch는 통과하고 LLM 단계가 돌아
      // 간 뒤, 승인으로 stage 4가 열린다. 격리 대조는 유지 대상 단계 자체로 관측한다.
      const llmFailing = await enqueue(page, 'checkout-web', null);

      // ── 워커가 단계 1-3까지 처리한다 ────────────────────────────────────
      await scaleWorkers(1);
      await expect
        .poll(() => analysisOf(page, llmFailing).then((a) => a.status), {
          timeout: 120_000,
          intervals: [1_000],
        })
        .toBe('awaiting_pipeline');

      // 기대 결과의 「유지」 반쪽은 실패 전의 성공 상태에서만 성립한다 — 여기서
      // 고정해 둔다(재시도 뒤의 단정이 이 값과 비교된다).
      const kept: Record<string, StageRow> = {};
      for (const k of ['fetch', 'cross_cutting', 'discovery_strategy']) {
        kept[k] = await stageOf(page, llmFailing, k);
        expect(kept[k].status, `${k} succeeded before stage 4 runs`).toBe('succeeded');
      }

      // ── 승인 → 재큐 → stage 4 → 트리거 → LLM 오류 ───────────────────────
      const approved = await page.request.post(
        `/api/analyses/${llmFailing}/discovery-strategy/approve`,
      );
      expect(approved.ok(), 'strategy approval opens stage 4').toBeTruthy();
      await expect
        .poll(
          async () => {
            const s = await stageOf(page, llmFailing, 'feature_candidates');
            return s.error === null ? s.status : `${s.status}: ${s.error}`;
          },
          {
            message: 'stage 4 should fail on the LLM failure trigger',
            timeout: 120_000,
            intervals: [1_000],
          },
        )
        .toBe('failed: LLM rejected the request (429)');

      // ── 실패한 단계와 그 사유 ───────────────────────────────────────────
      await page.goto(`/#/analyses/${llmFailing}`);
      const failedStage = page.locator('[data-stage="feature_candidates"]');
      await expect(failedStage).toContainText('LLM rejected the request (429)');
      await expect(page.getByTestId('pipeline-count')).toHaveText('3 of 5');

      const beforeRetry = await stageOf(page, llmFailing, 'feature_candidates');
      expect(beforeRetry.startedAt).not.toBeNull();

      // ── 「이 단계만 다시 시도」 — 트리거가 결정적이므로 같은 사유로 실패 ──
      await failedStage.getByTestId('retry').click();
      await expect
        .poll(
          async () => {
            const s = await stageOf(page, llmFailing, 'feature_candidates');
            const reran = s.startedAt !== null && s.startedAt !== beforeRetry.startedAt;
            return `${s.status}${reran ? ' (reran)' : ''}`;
          },
          {
            message: 'the retried stage should run again and fail on the same cause',
            timeout: 120_000,
            intervals: [1_000],
          },
        )
        .toBe('failed (reran)');
      const afterRetry = await stageOf(page, llmFailing, 'feature_candidates');
      expect(afterRetry.error).toContain('429');

      // ── 「부분」 재시도: 선행 단계는 시작 시각·사유까지 그대로, 이후 단계는
      //     그대로 pending ─────────────────────────────────────────────────
      for (const k of ['fetch', 'cross_cutting', 'discovery_strategy']) {
        const s = await stageOf(page, llmFailing, k);
        expect(s.startedAt, `${k} must be untouched by the retry`).toBe(kept[k].startedAt);
        expect(s.detail, `${k} must keep its measured detail`).toBe(kept[k].detail);
      }
      expect((await stageOf(page, llmFailing, 'acceptance_dependencies')).status).toBe('pending');
    } finally {
      // Back to the overlay's resting state and no trigger, whatever happened.
      await scaleWorkers(0);
      setWorkerEnv('FEATUREDOC_STUB_LLM_FAIL', null);
    }
  });
});
