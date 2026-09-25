// 검증 시나리오: 01-analysis-pipeline.md#시나리오 3
//
// 기대값을 상수로 박지 않는다. 근거 경로가 "실제로 분석된 트리 안의 경로인지"는
// 문서 자신이 아니라 **스캔이 본 저장소**를 기준으로 판정해야 의미가 있으므로,
// 화면·API가 내려준 값들끼리 대조한다. 픽스처를 바꿔도 이 테스트는 여전히 옳고,
// 추출이 근거 없는 항목을 만들어내는 순간에만 깨진다.
//
// Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { installApp } from '../support/github-app';

/** Must match `backend/src/cross_cutting.rs` AXES. */
const AXES = [
  'infrastructure',
  'repository_structure',
  'architecture',
  'framework',
  'middleware',
];

type Concern = { name: string; evidence: string[] };
type Category = { axis: string; items: Concern[] };
type Doc = {
  kind: string;
  content: { categories: Category[] };
  model: string;
  reproducibility: { verdict: string; comparedTo: string | null };
};

async function enqueue(page: Page, repo: string): Promise<string> {
  const res = await page.request.post('/api/analyses', {
    data: { repoUrl: `stub-account/${repo}`, branch: null },
  });
  expect(res.status(), `enqueue ${repo}`).toBe(201);
  return (await res.json()).id as string;
}

async function statusOf(page: Page, id: string): Promise<string> {
  const res = await page.request.get(`/api/analyses/${id}`);
  expect(res.ok()).toBeTruthy();
  return (await res.json()).status as string;
}

async function documentOf(page: Page, id: string): Promise<Doc> {
  const res = await page.request.get(`/api/analyses/${id}/documents/cross-cutting`);
  expect(res.status(), `cross-cutting document for ${id}`).toBe(200);
  return (await res.json()) as Doc;
}

test.describe('AC1.2: 횡단 관심사 자동 추출 및 문서화', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('분석이 5축 횡단 관심사 문서를 근거 경로와 함께 남기고, 재분석 결과를 명시한다', async ({
    page,
  }) => {
    try {
      // The overlay already rests at 0; make the precondition explicit so a queued
      // job cannot drain before the "nothing has run yet" assertion below.
      await scaleWorkers(0);

      await page.goto('/api/auth/login?as=ac12');
      await installApp(page.request);
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'openai', key: 'sk-proj-bbbbbbbbbbbbbbbbbbbbbb' },
      });
      expect(key.ok(), 'an active LLM key is 분석의 진입 조건').toBeTruthy();

      const first = await enqueue(page, 'payments-api');

      // 404, not an empty document: "아직 실행되지 않음" and "실행했고 아무것도 못
      // 찾음"은 사용자에게 다른 상태다.
      expect((await page.request.get(`/api/analyses/${first}/documents/cross-cutting`)).status())
        .toBe(404);

      await scaleWorkers(1);
      await expect
        .poll(() => statusOf(page, first), { timeout: 120_000, intervals: [1_000] })
        .toBe('awaiting_pipeline');

      const doc = await documentOf(page, first);
      expect(doc.kind).toBe('cross_cutting');
      expect(doc.content.categories.map((c) => c.axis).sort()).toEqual([...AXES].sort());

      const items = doc.content.categories.flatMap((c) => c.items);
      expect(items.length, '적어도 하나의 횡단 관심사가 추출돼야 한다').toBeGreaterThan(0);
      for (const item of items) {
        expect(item.name.length, `이름 없는 항목: ${JSON.stringify(item)}`).toBeGreaterThan(0);
        expect(item.evidence.length, `근거 없는 항목: ${item.name}`).toBeGreaterThan(0);
        for (const path of item.evidence) {
          expect(path, `분석 대상 밖의 경로를 근거로 들었다: ${path}`).toContain('payments-api/');
        }
      }

      expect(doc.reproducibility.verdict).toBe('first');
      expect(doc.reproducibility.comparedTo).toBeNull();

      await page.goto(`/#/analyses/${first}/cross-cutting`);
      await expect(page.getByTestId('axis')).toHaveCount(AXES.length);
      await expect(page.getByTestId('concern')).toHaveCount(items.length);
      await expect(page.getByTestId('reproducibility')).toContainText('첫 분석');
      const shown = doc.content.categories.find((c) => c.items.length > 0)!;
      const shownItem = shown.items[0];
      const axisCard = page.locator(`[data-axis="${shown.axis}"]`);
      await expect(axisCard).toContainText(shownItem.name);
      await expect(axisCard).toContainText(shownItem.evidence[0]);

      await page.goto(`/#/analyses/${first}`);
      await expect(page.locator('[data-stage="cross_cutting"]')).toContainText('categories');
      await page.getByTestId('open-cross-cutting').click();
      await expect(page.getByTestId('concerns-lede')).toBeVisible();

      const second = await enqueue(page, 'payments-api');
      await expect
        .poll(() => statusOf(page, second), { timeout: 120_000, intervals: [1_000] })
        .toBe('awaiting_pipeline');

      const rerun = await documentOf(page, second);
      expect(rerun.reproducibility.verdict).toBe('unchanged');
      expect(rerun.reproducibility.comparedTo).toBe(first);
      expect(rerun.content).toEqual(doc.content);

      await page.goto(`/#/analyses/${second}/cross-cutting`);
      await expect(page.getByTestId('reproducibility')).toContainText('직전 분석과 동일');
    } finally {
      await scaleWorkers(0);
    }
  });
});
