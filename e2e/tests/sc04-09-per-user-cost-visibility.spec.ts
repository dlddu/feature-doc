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
// 화면 단정으로 잰다.
//
// AC4.6 **검증 방법**의 「단계별 비용」은 응답(`stages[].spend`)과 화면(단계 행의 비용
// 칸) 두 자리에서 함께 잰다. 그 절만이 단계별을 사용자에게까지 요구하고 동사가
// 「추적 가능」이라 도달로 닫히지만, 같은 AC 에서 도달 논거가 한 번 번복된 자리라
// 화면도 함께 단정한다. 가르는 단정은 **끝난 단계 둘이 서로 다른 값을 말하는가**다 —
// 1단계는 LLM 없이 도는 유일한 단계라 `succeeded` 인데도 0 이고, 총합을 행마다
// 복사했다면 그 줄이 먼저 깨진다. 비용 축의 등식은 묻지 않는다(단계마다 센트로
// 올림되므로 단계 합이 총비용을 넘을 수 있다) — 등식은 호출·토큰 축의 부등식이다.
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

      // 단계별 — AC4.6 검증 방법의 「단계별 비용」. 작업 합은 위에서 이미 쟀으므로 여기서
      // 재는 것은 **그 합이 단계로 쪼개져 도달하는가**다.
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
      // 호출 축의 부등식. 비용 축으로 물으면 단계마다의 올림 때문에 참인 구현에서도 깨진다.
      const stageCalls = stages.reduce((n, s) => n + s.spend.llmCalls, 0);
      expect(stageCalls).toBeGreaterThan(0);
      expect(stageCalls, '단계 합은 작업 합을 넘지 않는다').toBeLessThanOrEqual(
        firstSpend.llmCalls,
      );

      // 화면 쪽 — 같은 값이 그 단계의 행에 그려진다.
      const stageSpend = (key: string) =>
        page.locator(`[data-stage="${key}"] [data-testid="stage-spend"]`);
      await expect(stageSpend('cross_cutting')).toHaveText(
        money(stage('cross_cutting').spend.costCents),
      );
      await expect(stageSpend('fetch')).toHaveText('$0.00');

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
      //
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
