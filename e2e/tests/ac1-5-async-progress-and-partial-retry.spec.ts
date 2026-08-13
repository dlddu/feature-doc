// 검증 AC: AC1.5
//
// AC1.5 (분석 작업의 비동기 실행과 진행 상황 가시성) 전용 spec.
//
// docs/test/01-analysis-pipeline.md의 두 시나리오를 그대로 따라간다:
//   시나리오 5 — 분석 중 앱을 종료했다 다시 열면 진행률과 현재 단계가 그대로 보인다.
//   시나리오 6 — 실패한 단계에서 "이 단계만 다시 시도"를 누르면 그 단계만 재실행된다.
//
// 진행은 서버 상태(`analysis_stages`)이므로 시나리오 5는 "브라우저를 새로고침해도
// 같은 화면"으로 관측한다 — S04가 클라이언트에 진행을 들고 있다면 이 단정이 깨진다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. The residual effect of a running worker is that it drains
// every queued job, including ones other specs left behind — so every assertion
// below is about a job this spec created.
//
// Like every spec it signs in as its own stub user (`?as=ac15`). Reaching S02
// requires an App installation and an active LLM key; those are set up through the
// API rather than S01's screens, which ac4-1/ac4-2 own — walking another AC's screen
// is setup, not this file's verification target.
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

async function statusOf(page: Page, id: string): Promise<string> {
  return (await analysisOf(page, id)).status;
}

async function stageOf(page: Page, id: string, key: string): Promise<StageRow> {
  const stage = (await analysisOf(page, id)).stages.find((s) => s.key === key);
  expect(stage, `analysis ${id} has no ${key} stage`).toBeTruthy();
  return stage!;
}

test.describe('AC1.5: 비동기 진행 가시성과 실패 단계의 부분 재시도', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('S04가 적재된 진행을 보여주고, 복귀 후에도 같으며, 실패 단계만 다시 실행된다', async ({
    page,
  }) => {
    try {
      // The overlay already rests at 0; make the precondition explicit so a queued
      // job cannot drain before the "nothing has run yet" assertions below.
      await scaleWorkers(0);

      // ── setup: this spec's own user, App installation, LLM key ──────────
      await page.goto('/api/auth/login?as=ac15');
      expect((await page.request.get('/api/github/setup?installation_id=4242')).ok()).toBeTruthy();
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'anthropic', key: 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa' },
      });
      expect(key.ok(), 'an active LLM key is S02의 진입 조건').toBeTruthy();

      // Two jobs: one the stub repository can serve, and one pointing at a branch
      // the repository does not have — the fetch stage fails on that one exactly as
      // the real GitHub tree request would (404). The branch is a user-typed field
      // on S03, so this failure is reachable without any test-only hook.
      const good = await enqueue(page, 'payments-api', null);
      const failing = await enqueue(page, 'checkout-web', 'no-such-branch');

      // ── before any worker: S04 shows the pipeline waiting, not "done" ───
      await page.goto(`/#/analyses/${good}`);
      await expect(page.getByTestId('pipeline-count')).toHaveText('0 of 5');
      await expect(page.getByTestId('progress-percent')).toHaveText('0');
      await expect(page.getByTestId('stage')).toHaveCount(5);
      await expect(page.locator('[data-stage="fetch"]')).toContainText('Fetch repository');
      await expect(page.locator('[data-stage="fetch"]')).toContainText('대기 중');

      // ── let one worker run both jobs ────────────────────────────────────
      await scaleWorkers(1);
      await expect
        .poll(() => statusOf(page, good), { timeout: 120_000, intervals: [1_000] })
        .toBe('awaiting_pipeline');
      await expect
        .poll(() => statusOf(page, failing), { timeout: 120_000, intervals: [1_000] })
        .toBe('failed');

      // ── 진행 가시성: the finished stage reports what it measured ────────
      await page.goto(`/#/analyses/${good}`);
      await expect(page.getByTestId('pipeline-count')).toHaveText('1 of 5');
      await expect(page.getByTestId('progress-percent')).toHaveText('20');
      // The stub repository is 2300 KiB ⇒ 766 files · 2.2 MB (repo_scan::stub_scan);
      // the number is the worker's measurement, not a fixture in this file.
      await expect(page.locator('[data-stage="fetch"]')).toContainText('766 files · 2.2 MB');
      await expect(page.getByTestId('awaiting-pipeline')).toBeVisible();
      // Cost is still the pre-flight estimate — measured spend is AC4.6.
      await expect(page.getByTestId('spend')).toContainText('Est. LLM Spend');

      // ── 시나리오 5: 앱을 닫았다 다시 열어도 같은 진행 ───────────────────
      await page.reload();
      await expect(page.getByTestId('pipeline-count')).toHaveText('1 of 5');
      await expect(page.getByTestId('progress-percent')).toHaveText('20');
      await expect(page.locator('[data-stage="fetch"]')).toContainText('766 files · 2.2 MB');

      // ── the user path into S04: S02 카드 → 진행 상황 ─────────────────────
      await page.goto('/');
      const cont = page.getByTestId('continue');
      await cont.click();
      await expect(page.getByTestId('ready')).toBeVisible();
      await cont.click();
      const card = page.getByTestId('repo-card').filter({ hasText: 'stub-account/payments-api' });
      await expect(card).toContainText('step 1 of 5');
      await card.getByTestId('open-progress').click();
      await expect(page.getByTestId('pipeline-count')).toHaveText('1 of 5');

      // ── 시나리오 6: 실패한 단계와 그 사유, 그리고 그 단계만의 재시도 ────
      await page.goto(`/#/analyses/${failing}`);
      const failedStage = page.locator('[data-stage="fetch"]');
      await expect(failedStage).toContainText('github tree rejected (404)');
      await expect(page.getByTestId('pipeline-count')).toHaveText('0 of 5');

      const beforeRetry = await stageOf(page, failing, 'fetch');
      expect(beforeRetry.startedAt).not.toBeNull();

      await failedStage.getByTestId('retry').click();

      // The reset is observed through the API rather than the DOM on purpose: a
      // worker is running, so the "waiting" render lasts only until it re-claims
      // (~2s) — asserting on that frame would be a race. What matters is below.

      // The job really re-runs: a *new* attempt, with the same deterministic
      // cause (the branch still does not exist), not the old record left in place.
      await expect
        .poll(async () => (await stageOf(page, failing, 'fetch')).startedAt, {
          message: 'the retried stage should run again, with a fresh start time',
          timeout: 120_000,
          intervals: [1_000],
        })
        .not.toBe(beforeRetry.startedAt);
      const afterRetry = await stageOf(page, failing, 'fetch');
      expect(afterRetry.status).toBe('failed');
      expect(afterRetry.error).toContain('404');

      // Only that stage moved: the four that never ran are still waiting…
      const stages = (await analysisOf(page, failing)).stages;
      for (const key of LATER_STAGES) {
        expect(stages.find((s) => s.key === key)?.status, `${key} must be untouched`).toBe(
          'pending',
        );
      }
      // …and the other analysis is not disturbed by a retry on this one.
      expect((await stageOf(page, good, 'fetch')).status).toBe('succeeded');
      expect((await stageOf(page, good, 'fetch')).detail).toBe('766 files · 2.2 MB');
    } finally {
      // Back to the overlay's resting state, whatever happened above, so a later
      // spec never finds a worker quietly draining its queue.
      await scaleWorkers(0);
    }
  });
});
