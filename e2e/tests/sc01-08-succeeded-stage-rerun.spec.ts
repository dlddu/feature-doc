// 검증 시나리오: 01-analysis-pipeline.md#시나리오 8
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { installApp } from '../support/github-app';

type StageRow = {
  key: string;
  status: string;
  detail: string | null;
  startedAt: number | null;
};

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

async function candidateKeys(page: Page, id: string): Promise<string[]> {
  const res = await page.request.get(`/api/analyses/${id}/candidates`);
  expect(res.ok(), 'candidate list').toBeTruthy();
  const body = await res.json();
  return (body.candidates as { key: string }[]).map((c) => c.key);
}

async function strategyApproved(page: Page, id: string): Promise<boolean> {
  const res = await page.request.get(`/api/analyses/${id}/discovery-strategy`);
  expect(res.ok(), 'strategy').toBeTruthy();
  return (await res.json()).approved as boolean;
}

test.describe('시나리오 8: 끝난 단계의 단독 재실행', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('끝난 탐색 전략 단계만 다시 돌고, 앞의 횡단 분석과 뒤의 후보·승인은 그대로다', async ({
    page,
  }) => {
    try {
      await scaleWorkers(0);

      await page.goto('/api/auth/login?as=sc0108');
      await installApp(page.request);
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'anthropic', key: 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa' },
      });
      expect(key.ok(), 'an active LLM key is 분석의 진입 조건').toBeTruthy();

      const created = await page.request.post('/api/analyses', {
        data: { repoUrl: 'stub-account/payments-api', branch: null },
      });
      expect(created.status()).toBe(201);
      const id = (await created.json()).id as string;

      await scaleWorkers(1);
      await expect
        .poll(() => analysisOf(page, id).then((a) => a.status), {
          timeout: 120_000,
          intervals: [1_000],
        })
        .toBe('awaiting_pipeline');

      expect(await strategyApproved(page, id)).toBe(false);
      const approved = await page.request.post(`/api/analyses/${id}/discovery-strategy/approve`);
      expect(approved.ok(), 'strategy approval opens stage 4').toBeTruthy();
      await expect
        .poll(
          async () => {
            const a = await analysisOf(page, id);
            const s4 = a.stages.find((s) => s.key === 'feature_candidates')?.status;
            return `${a.status} / ${s4}`;
          },
          { timeout: 120_000, intervals: [1_000] },
        )
        .toBe('awaiting_pipeline / succeeded');

      const kept: Record<string, StageRow> = {};
      for (const k of ['cross_cutting', 'feature_candidates']) {
        kept[k] = await stageOf(page, id, k);
      }
      const before = await stageOf(page, id, 'discovery_strategy');
      expect(before.status).toBe('succeeded');
      const keysBefore = await candidateKeys(page, id);
      expect(keysBefore.length).toBeGreaterThan(0);

      await page.goto(`/#/analyses/${id}`);
      const strategyStage = page.locator('[data-stage="discovery_strategy"]');
      await expect(strategyStage.getByTestId('rerun')).toBeVisible();
      await expect(strategyStage.getByTestId('rerun')).toHaveText('이 단계 다시 실행');
      await strategyStage.getByTestId('rerun').click();

      // A *new* attempt: `startedAt` is cleared by the reset, so only a value that is
      // present and different from the old one means a worker really ran it again.
      await expect
        .poll(
          async () => {
            const s = await stageOf(page, id, 'discovery_strategy');
            const reran = s.startedAt !== null && s.startedAt !== before.startedAt;
            return `${s.status}${reran ? ' (reran)' : ''}`;
          },
          {
            message: 'the re-run stage should run again and succeed',
            timeout: 120_000,
            intervals: [1_000],
          },
        )
        .toBe('succeeded (reran)');
      await expect
        .poll(() => analysisOf(page, id).then((a) => a.status), {
          timeout: 60_000,
          intervals: [1_000],
        })
        .toBe('awaiting_pipeline');

      for (const k of ['cross_cutting', 'feature_candidates']) {
        const s = await stageOf(page, id, k);
        expect(s.status, `${k} stays succeeded`).toBe('succeeded');
        expect(s.startedAt, `${k} must not re-run`).toBe(kept[k].startedAt);
        expect(s.detail, `${k} keeps its result`).toBe(kept[k].detail);
      }
      expect(await candidateKeys(page, id)).toEqual(keysBefore);
      expect(await strategyApproved(page, id), 'the approval survives the re-run').toBe(true);
    } finally {
      await scaleWorkers(0);
    }
  });
});
