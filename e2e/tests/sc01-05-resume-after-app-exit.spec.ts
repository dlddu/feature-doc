// 검증 시나리오: 01-analysis-pipeline.md#시나리오 5
//
// 진행은 서버 상태(`analysis_stages`)이므로 "브라우저를 새로고침해도 같은 화면"으로
// 관측한다 — 분석 진행 화면이 클라이언트에 진행을 들고 있다면 이 단정이 깨진다.
//
// Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.
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

      await page.goto('/api/auth/login?as=ac15');
      expect((await page.request.get('/api/github/setup?installation_id=4242')).ok()).toBeTruthy();
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'anthropic', key: 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa' },
      });
      expect(key.ok(), 'an active LLM key is 홈 화면의 진입 조건').toBeTruthy();

      const good = await enqueue(page, 'payments-api', null);

      await page.goto(`/#/analyses/${good}`);
      await expect(page.getByTestId('pipeline-count')).toHaveText('0 of 5');
      await expect(page.getByTestId('progress-percent')).toHaveText('0');
      await expect(page.getByTestId('stage')).toHaveCount(5);
      await expect(page.locator('[data-stage="fetch"]')).toContainText('Fetch repository');
      await expect(page.locator('[data-stage="fetch"]')).toContainText('대기 중');

      await scaleWorkers(1);
      await expect
        .poll(() => statusOf(page, good), { timeout: 120_000, intervals: [1_000] })
        .toBe('awaiting_pipeline');

      await page.goto(`/#/analyses/${good}`);
      await expect(page.getByTestId('pipeline-count')).toHaveText('3 of 5');
      await expect(page.getByTestId('progress-percent')).toHaveText('60');
      // The stub repository is 2300 KiB ⇒ 766 files · 2.2 MB (repo_scan::stub_scan);
      // the number is the worker's measurement, not a fixture in this file.
      await expect(page.locator('[data-stage="fetch"]')).toContainText('766 files · 2.2 MB');
      await expect(page.getByTestId('awaiting-pipeline')).toBeVisible();
      // Cost is still the pre-flight estimate — measured spend is AC4.6.
      await expect(page.getByTestId('spend')).toContainText('Est. LLM Spend');

      await page.reload();
      await expect(page.getByTestId('pipeline-count')).toHaveText('3 of 5');
      await expect(page.getByTestId('progress-percent')).toHaveText('60');
      await expect(page.locator('[data-stage="fetch"]')).toContainText('766 files · 2.2 MB');

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
      await scaleWorkers(0);
    }
  });
});
