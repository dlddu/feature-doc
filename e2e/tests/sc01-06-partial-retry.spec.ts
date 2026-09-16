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
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';

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
});
