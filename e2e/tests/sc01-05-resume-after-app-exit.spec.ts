// 검증 시나리오: 01-analysis-pipeline.md#시나리오 5
//
// 「분석 중 앱 종료 후 복귀」 전용 spec (AC1.5).
//
// 시나리오 6(부분 재시도)의 단정은 `sc01-06-partial-retry.spec.ts`가 지킨다 —
// 한 파일에 있던 두 시나리오를 분리한 것은 `rct_20260916-0002`가 닫았다.
//
// 진행은 서버 상태(`analysis_stages`)이므로 시나리오 5는 "브라우저를 새로고침해도
// 같은 화면"으로 관측한다 — 분석 진행 화면이 클라이언트에 진행을 들고 있다면 이 단정이 깨진다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. The residual effect of a running worker is that it drains
// every queued job, including ones other specs left behind — so every assertion
// below is about a job this spec created.
//
// Like every spec it signs in as its own stub user (`?as=ac15`). Reaching Home
// requires an App installation and an active LLM key; those are set up through the
// API rather than Credentials Setup's screens, which ac4-1/ac4-2 own — walking another AC's screen
// is setup, not this file's verification target.
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';

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

test.describe('AC1.5: 비동기 진행 가시성과 복귀', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('분석 진행 화면이 적재된 진행을 보여주고, 앱을 닫았다 다시 열어도 같다', async ({
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
      expect(key.ok(), 'an active LLM key is 홈 화면의 진입 조건').toBeTruthy();

      // One job the stub repository can serve. (The failing-stage half that used to
      // share this file now lives in sc01-06-partial-retry.spec.ts.)
      const good = await enqueue(page, 'payments-api', null);

      // ── before any worker: Analysis Progress shows the pipeline waiting, not "done" ───
      await page.goto(`/#/analyses/${good}`);
      await expect(page.getByTestId('pipeline-count')).toHaveText('0 of 5');
      await expect(page.getByTestId('progress-percent')).toHaveText('0');
      await expect(page.getByTestId('stage')).toHaveCount(5);
      await expect(page.locator('[data-stage="fetch"]')).toContainText('Fetch repository');
      await expect(page.locator('[data-stage="fetch"]')).toContainText('대기 중');

      // ── let one worker run the job ──────────────────────────────────────
      await scaleWorkers(1);
      await expect
        .poll(() => statusOf(page, good), { timeout: 120_000, intervals: [1_000] })
        .toBe('awaiting_pipeline');

      // ── 진행 가시성: the finished stage reports what it measured ────────
      // Three stages are implemented as of slice 4b-1 (fetch + cross_cutting, AC1.2 +
      // discovery_strategy, AC1.3); stages 4-5 stay pending, so the run lands at 3 of 5.
      // This number is a direct consequence of how much of the pipeline exists — it
      // moves every time a slice implements one more stage (1→2 in 4a, 2→3 here).
      await page.goto(`/#/analyses/${good}`);
      await expect(page.getByTestId('pipeline-count')).toHaveText('3 of 5');
      await expect(page.getByTestId('progress-percent')).toHaveText('60');
      // The stub repository is 2300 KiB ⇒ 766 files · 2.2 MB (repo_scan::stub_scan);
      // the number is the worker's measurement, not a fixture in this file.
      await expect(page.locator('[data-stage="fetch"]')).toContainText('766 files · 2.2 MB');
      await expect(page.getByTestId('awaiting-pipeline')).toBeVisible();
      // Cost is still the pre-flight estimate — measured spend is AC4.6.
      await expect(page.getByTestId('spend')).toContainText('Est. LLM Spend');

      // ── 시나리오 5: 앱을 닫았다 다시 열어도 같은 진행 ───────────────────
      await page.reload();
      await expect(page.getByTestId('pipeline-count')).toHaveText('3 of 5');
      await expect(page.getByTestId('progress-percent')).toHaveText('60');
      await expect(page.locator('[data-stage="fetch"]')).toContainText('766 files · 2.2 MB');

      // ── the user path into 분석 진행 화면: 홈 화면 카드 → 진행 상황 ─────────────────────
      await page.goto('/');
      const cont = page.getByTestId('continue');
      await cont.click();
      await expect(page.getByTestId('ready')).toBeVisible();
      await cont.click();
      const card = page.getByTestId('repo-card').filter({ hasText: 'stub-account/payments-api' });
      await expect(card).toContainText('step 3 of 5');
      await card.getByTestId('open-progress').click();
      await expect(page.getByTestId('pipeline-count')).toHaveText('3 of 5');
    } finally {
      // Back to the overlay's resting state, whatever happened above, so a later
      // spec never finds a worker quietly draining its queue.
      await scaleWorkers(0);
    }
  });
});
