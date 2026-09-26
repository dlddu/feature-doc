// 검증 시나리오: 04-platform.md#시나리오 9
//
// Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { installApp } from '../support/github-app';

type Spend = {
  llmCalls: number;
  inputTokens: number;
  outputTokens: number;
  costCents: number;
};

type Usage = {
  total: Spend;
  analyses: (Spend & { analysisId: string; repoName: string })[];
};

type StageRow = { key: string; status: string; spend: Spend };

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

async function spendOf(page: Page, id: string): Promise<Spend> {
  const res = await page.request.get(`/api/analyses/${id}`);
  expect(res.ok()).toBeTruthy();
  return (await res.json()).spend as Spend;
}

async function stagesOf(page: Page, id: string): Promise<StageRow[]> {
  const res = await page.request.get(`/api/analyses/${id}`);
  expect(res.ok()).toBeTruthy();
  return (await res.json()).stages as StageRow[];
}

function money(cents: number): string {
  return `$${(cents / 100).toFixed(2)}`;
}

// 화면이 그리는 문자열을 그대로 다시 만든다. `frontend/src/format.ts` 의 `formatCount` 와
// 같은 규칙이고, `toLocaleString` 이 아닌 이유도 같다 — Node 와 브라우저의 ICU 가 달라도
// 이 단정은 흔들리지 않아야 한다.
function count(n: number): string {
  return String(n).replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

async function usageOf(page: Page): Promise<Usage> {
  const res = await page.request.get('/api/usage');
  expect(res.ok(), '/api/usage 는 본인 세션에 답한다').toBeTruthy();
  return (await res.json()) as Usage;
}

test.describe('AC4.6: 사용자별 비용 가시성', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('여러 분석을 돌린 사용자가 작업별·전체별 호출 수와 추정 비용을 본다', async ({ page }) => {
    try {
      await scaleWorkers(0);

      await page.goto('/api/auth/login?as=ac46');
      await installApp(page.request);
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'anthropic', key: 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa' },
      });
      expect(key.ok(), 'an active LLM key is 홈 화면의 진입 조건').toBeTruthy();

      const first = await enqueue(page, 'payments-api');
      const second = await enqueue(page, 'checkout-web');

      // 호출 전: 측정치는 0 이다. pre-flight 추정은 이미 0 이 아니므로, 이 단정이
      // 바로 「화면이 추정을 실측으로 부르지 않는다」는 음성 대조다.
      const before = await usageOf(page);
      expect(before.total.llmCalls, '아직 아무도 부르지 않았다').toBe(0);
      expect(before.total.costCents).toBe(0);
      expect(before.analyses.length, '작업별 행은 분석마다 하나다').toBe(2);

      await page.goto(`/#/analyses/${first}`);
      await expect(page.getByTestId('cost-so-far')).toHaveText('$0.00');

      await scaleWorkers(1);
      for (const id of [first, second]) {
        await expect
          .poll(() => statusOf(page, id), { timeout: 120_000, intervals: [250] })
          .toBe('awaiting_pipeline');
      }

      const firstSpend = await spendOf(page, first);
      expect(firstSpend.llmCalls, '3단계까지면 호출이 있었다').toBeGreaterThan(0);
      expect(firstSpend.inputTokens).toBeGreaterThan(0);
      expect(firstSpend.outputTokens).toBeGreaterThan(0);
      expect(firstSpend.costCents).toBeGreaterThan(0);

      await page.goto(`/#/analyses/${first}`);
      const shown = `$${(firstSpend.costCents / 100).toFixed(2)}`;
      await expect(page.getByTestId('cost-so-far')).toHaveText(shown);
      await expect(page.getByTestId('llm-calls')).toHaveText(count(firstSpend.llmCalls));
      await expect(page.getByTestId('tokens-used')).toHaveText(
        count(firstSpend.inputTokens + firstSpend.outputTokens),
      );
      await page.reload();
      await expect(page.getByTestId('cost-so-far')).toHaveText(shown);
      await expect(page.getByTestId('llm-calls')).toHaveText(count(firstSpend.llmCalls));

      const stages = await stagesOf(page, first);
      const stage = (key: string): StageRow => {
        const found = stages.find((s) => s.key === key);
        expect(found, `단계 ${key} 가 응답에 있다`).toBeTruthy();
        return found!;
      };
      expect(stage('fetch').status, '3단계까지 돌았으면 1단계는 끝나 있다').toBe('succeeded');
      // LLM 없이 도는 유일한 단계 — 끝나고도 0 이다. 이 두 줄이 「총합을 행마다 복사한
      // 것이 아니다」의 음성 대조이고, 아래 3단계 값과 쌍으로만 의미가 있다.
      expect(stage('fetch').spend.llmCalls).toBe(0);
      expect(stage('fetch').spend.costCents).toBe(0);
      expect(stage('cross_cutting').spend.llmCalls).toBeGreaterThan(0);
      expect(stage('cross_cutting').spend.costCents).toBeGreaterThan(0);
      const stageCalls = stages.reduce((n, s) => n + s.spend.llmCalls, 0);
      expect(stageCalls).toBeGreaterThan(0);
      expect(stageCalls, '단계 합은 작업 합을 넘지 않는다').toBeLessThanOrEqual(
        firstSpend.llmCalls,
      );

      const stageSpend = (key: string) =>
        page.locator(`[data-stage="${key}"] [data-testid="stage-spend"]`);
      await expect(stageSpend('cross_cutting')).toHaveText(
        money(stage('cross_cutting').spend.costCents),
      );
      await expect(stageSpend('fetch')).toHaveText('$0.00');

      const after = await usageOf(page);
      const rows = after.analyses;
      expect(rows.length).toBe(2);
      expect(after.total.llmCalls).toBe(rows.reduce((n, r) => n + r.llmCalls, 0));
      expect(after.total.inputTokens).toBe(rows.reduce((n, r) => n + r.inputTokens, 0));
      expect(after.total.outputTokens).toBe(rows.reduce((n, r) => n + r.outputTokens, 0));
      expect(after.total.costCents).toBeGreaterThan(0);

      expect(after.total.llmCalls).toBeGreaterThan(before.total.llmCalls);

      // 셋업을 API 로 끝냈어도 로드는 자격증명 화면에서 시작한다 — 라우팅이 서버 게이트가
      // 아니라 상태 머신(`App.tsx` 의 `screen`)이고 홈에는 주소가 없다. 그래서 `goto('/')`
      // 하나로는 홈이 서지 않는다. 진입 두 줄은 sc01-01·sc01-05 와 같다.
      await page.goto('/');
      const enterHome = page.getByTestId('register-key');
      await expect(enterHome).toBeEnabled();
      await enterHome.click();
      // 홈이 섰는지를 먼저 잰다 — 이 줄이 없으면 아래 세 칸의 `element(s) not found` 가
      // 「홈에 못 왔다」와 「누적 사용량이 안 그려졌다」를 구분하지 못한다.
      await expect(page.getByTestId('repo-card').first()).toBeVisible();
      await expect(page.getByTestId('usage-calls')).toHaveText(count(after.total.llmCalls));
      await expect(page.getByTestId('usage-tokens')).toHaveText(
        count(after.total.inputTokens + after.total.outputTokens),
      );
      await expect(page.getByTestId('usage-cost')).toHaveText(
        `$${(after.total.costCents / 100).toFixed(2)}`,
      );

      const mine = rows.find((r) => r.analysisId === first)!;
      expect(mine.llmCalls, '작업별 행이 상세와 같은 값을 말한다').toBe(firstSpend.llmCalls);
      expect(mine.costCents).toBe(firstSpend.costCents);

      await page.goto(`/#/analyses/${first}`);
      await page.getByTestId('open-cross-cutting').click();
      await page.getByTestId('to-discovery-strategy').click();
      await page.getByTestId('strategy-approve').click();
      await expect
        .poll(
          async () => {
            const res = await page.request.get(`/api/analyses/${first}/candidates`);
            return res.ok() ? ((await res.json()).extracted as boolean) : false;
          },
          { timeout: 120_000, intervals: [250] },
        )
        .toBe(true);
      await expect(page.getByTestId('strategy-open-candidates')).toBeEnabled();
      await page.getByTestId('strategy-open-candidates').click();

      const afterCandidates = await spendOf(page, first);
      expect(
        afterCandidates.llmCalls,
        '4단계가 한 번 더 불렀으니 호출 수가 늘었다',
      ).toBeGreaterThan(firstSpend.llmCalls);
      await expect(page.getByTestId('sift-cost')).toContainText('누적 비용');
      await expect(page.getByTestId('sift-cost')).toContainText(
        `$${(afterCandidates.costCents / 100).toFixed(2)}`,
      );
    } finally {
      await scaleWorkers(0);
    }
  });

  test('다른 사용자의 비용은 내 누적에 들어오지 않는다', async ({ browser }) => {
    const other = await browser.newContext();
    try {
      const page = await other.newPage();
      await page.goto('/api/auth/login?as=ac46-other');
      const seen = await usageOf(page);
      expect(seen.total.llmCalls, '내가 돌린 분석이 없다').toBe(0);
      expect(seen.analyses.length).toBe(0);
    } finally {
      await other.close();
    }
  });
});
