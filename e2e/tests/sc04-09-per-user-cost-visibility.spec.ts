// 검증 시나리오: 04-platform.md#시나리오 9
//
// 이 spec 이 재는 것은 「숫자가 화면에 있다」가 아니라 **그 숫자가 실제로 일어난
// 호출에서 나왔는가**다. 시작 전 pre-flight 추정은 저장소 크기에서 나온 값이라
// 호출을 한 번도 안 해도 0 이 아니다 — 그래서 단정을 「0 이 아니다」로 두면
// 추정을 실측으로 오인하고도 초록이 된다. 대신 **호출 전후의 델타**로 잰다:
// 워커를 세워 둔 동안은 측정 비용이 0 이고, 단계가 돌고 난 뒤에 올라간다.
//
// 「운영자 화면에서도 동일한 데이터에 접근 가능」은 같은 주소(`/api/usage`)로
// 닫는다. 이 제품에는 운영자 역할도 운영자 여정도 없고(여정 6개가 전부 최종
// 사용자용이다), 같은 숫자의 두 번째 사본은 원본과 어긋날 수 있는 두 번째
// 자리일 뿐이다.
//
// **사용자 쪽은 같은 이유로 닫히지 않는다.** 시나리오 9 의 동사는 「접근 가능」이
// 아니라 「확인」·「표시된다」이고, AC4.6 도 「노출된다」라고 쓴다 — 도달만으로는
// 모자라고 화면이 그려야 한다. 그래서 작업별(분석 진행)과 전체별(홈) 두 자리를
// 화면 단정으로 잰다. 단계별 내역은 아직 `worker_api::submit_document` 의
// `llm usage recorded` 로그에만 있다(AC4.6 검증 방법의 「단계별 비용」 — 후속).
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

      // 두 분석을 모두 3단계까지 돌린다.
      await scaleWorkers(1);
      for (const id of [first, second]) {
        await expect
          .poll(() => statusOf(page, id), { timeout: 120_000, intervals: [250] })
          .toBe('awaiting_pipeline');
      }

      // 작업별 — 화면이 이 분석의 측정 비용을 띄운다.
      const firstSpend = await spendOf(page, first);
      expect(firstSpend.llmCalls, '3단계까지면 호출이 있었다').toBeGreaterThan(0);
      expect(firstSpend.inputTokens).toBeGreaterThan(0);
      expect(firstSpend.outputTokens).toBeGreaterThan(0);
      expect(firstSpend.costCents).toBeGreaterThan(0);

      await page.goto(`/#/analyses/${first}`);
      const shown = `$${(firstSpend.costCents / 100).toFixed(2)}`;
      await expect(page.getByTestId('cost-so-far')).toHaveText(shown);
      // 시나리오 9 의 실행 단계는 「사용자 화면에서 누적 LLM 호출 횟수와 토큰 사용량 확인」이다.
      // 값이 `/api/analyses/{id}` 에 실려 있다는 것으로는 그 단계가 닫히지 않는다 — 화면이
      // 그려야 닫힌다. 그래서 여기서 재는 것은 응답이 아니라 그리드의 두 칸이다.
      await expect(page.getByTestId('llm-calls')).toHaveText(count(firstSpend.llmCalls));
      await expect(page.getByTestId('tokens-used')).toHaveText(
        count(firstSpend.inputTokens + firstSpend.outputTokens),
      );
      // 새로고침해도 같다 — 비용도 진행과 같은 서버 상태다.
      await page.reload();
      await expect(page.getByTestId('cost-so-far')).toHaveText(shown);
      await expect(page.getByTestId('llm-calls')).toHaveText(count(firstSpend.llmCalls));

      // 전체별 — 작업별의 합과 같아야 한다. 두 숫자가 다른 곳에서 나오면 언젠가
      // 어긋나므로 등식으로 잰다.
      const after = await usageOf(page);
      const rows = after.analyses;
      expect(rows.length).toBe(2);
      expect(after.total.llmCalls).toBe(rows.reduce((n, r) => n + r.llmCalls, 0));
      expect(after.total.inputTokens).toBe(rows.reduce((n, r) => n + r.inputTokens, 0));
      expect(after.total.outputTokens).toBe(rows.reduce((n, r) => n + r.outputTokens, 0));
      expect(after.total.costCents).toBeGreaterThan(0);

      // 델타가 곧 「이 숫자는 실제 호출에서 왔다」의 증거다.
      expect(after.total.llmCalls).toBeGreaterThan(before.total.llmCalls);

      // 「작업별·전체별 … 이 표시된다」의 전체별 쪽. 이 사용자의 분석 전부가 한자리에
      // 모이는 화면은 홈뿐이라, 합계가 설 자리도 거기 하나다.
      await page.goto('/');
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

      // 후보 화면도 같은 값을 쓴다 — 4단계까지 열어 그 자리를 관측한다.
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
