// 검증 시나리오: 01-analysis-pipeline.md#시나리오 1
//
// 이 파일은 산출물의 **순서·제시·코드 근거 부착**을 한 바퀴 걷는다 — 단계별 세부
// 정합성은 각자의 전용 spec이 지킨다(sc01-02 트리거·범위, sc01-03 문서 내용,
// sc01-04 전략 편집, sc01-07 후보 결정). 여기서 그 세부를 다시 단정하면 같은 행동이
// 여러 파일의 검증으로 이중 계상되므로, 이 파일은 걷는 것 자체만 검증한다.
//
// 「모바일에서」라는 서술은 AC4.4(모바일 폭 규칙 — 구현 대기)의 몫이 아니라 이
// 시나리오의 서술 배경이다. 화면 경로는 기존 spec과 같은 데스크톱 chromium
// 프로젝트로 걷는다; 폭 규칙이 착지하면 AC4.4의 spec이 그 층을 검증한다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. The residual effect of a running worker is that it drains
// every queued job, including ones other specs left behind — so every assertion
// below is about a job this spec created.
//
// Like every spec it signs in as its own stub user (`?as=sc0101`). Reaching Home
// requires an App installation and an active LLM key; those are set up through the
// API rather than Credentials Setup's screens, which sc04-01/sc04-03 own — walking
// another scenario's screen is setup, not this file's verification target.
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';

type StageRow = {
  key: string;
  status: string;
  detail: string | null;
};

async function analysisOf(page: Page, id: string): Promise<{ status: string; stages: StageRow[] }> {
  const res = await page.request.get(`/api/analyses/${id}`);
  expect(res.ok()).toBeTruthy();
  return await res.json();
}

type Candidate = {
  key: string;
  name: string;
  location: string;
  decision: string;
};
type CandidateList = { candidates: Candidate[]; undecided: number; extracted: boolean };

async function statusOf(page: Page, id: string): Promise<string> {
  const res = await page.request.get(`/api/analyses/${id}`);
  expect(res.ok()).toBeTruthy();
  return (await res.json()).status as string;
}

async function candidatesOf(page: Page, id: string): Promise<CandidateList> {
  const res = await page.request.get(`/api/analyses/${id}/candidates`);
  expect(res.status(), `candidates for ${id}`).toBe(200);
  return (await res.json()) as CandidateList;
}

test.describe('시나리오 1: 정상 저장소 연결 및 전체 파이프라인 실행', () => {
  test.describe.configure({ mode: 'serial', timeout: 420_000 });

  test('산출물이 문서 → 전략 → 후보 순서로 제시되고, 각 산출물에 코드 근거가 붙는다', async ({
    page,
  }) => {
    try {
      // The overlay already rests at 0; make the precondition explicit so a queued
      // job cannot drain before this spec starts walking.
      await scaleWorkers(0);

      await page.goto('/api/auth/login?as=sc0101');
      expect((await page.request.get('/api/github/setup?installation_id=4242')).ok()).toBeTruthy();
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'anthropic', key: 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa' },
      });
      expect(key.ok(), 'an active LLM key is the entry condition').toBeTruthy();

      // 저장소 연결 폼은 홈과 한 화면이다(슬라이스 ⑥ 병합) — 목록 아래 「새 저장소 연결」에
      // 사용자가 직접 입력한다. 별도의 연결 화면으로 건너뛰는 단계가 없어졌다.
      // 화면 라우팅은 서버 게이트가 아니라 상태 머신이라, 셋업이 API로 됐어도 로드는
      // 자격증명 화면에서 시작한다 — continue 두 번이 실제 홈 진입 경로다(선례: sc01-05).
      await page.goto('/');
      const cont = page.getByTestId('continue');
      await expect(cont).toBeEnabled();
      await cont.click();
      await expect(page.getByTestId('ready')).toBeVisible();
      await cont.click();
      // 시작 전에는 어떤 저장소도 실행 이력이 없다(메트릭 그리드는 목업에 없어 제거됐으므로
      // 같은 사실을 카드의 실행 상태로 단정한다 — 상태 배지가 하나도 없다).
      await expect(page.getByTestId('repo-card')).not.toHaveCount(0);
      await expect(page.locator('[data-testid="repo-card"] .badge')).toHaveCount(0);
      await page.getByTestId('repo-url').fill('stub-account/payments-api');
      // 시나리오는 브랜치 입력을 명시한다 — 스텁의 기본 브랜치 이름을 그대로 입력한다.
      await page.getByTestId('branch').fill('main');
      await page.getByTestId('check-access').click();
      await expect(page.getByTestId('estimate')).toBeVisible();
      await expect(page.getByTestId('access')).toContainText('has access');
      await page.getByTestId('start-analysis').click();

      // 큐잉 단정은 하지 않는다 — 바로 워커를 켠다(임대 규약: 자기가 만든 job만 단정).
      await expect(page.getByTestId('repo-card').filter({ hasText: 'stub-account/payments-api' }))
        .toBeVisible();

      // 이 분석의 id를 잡는다 — 사용자 격리(AC4.7)가 미구현이라 목록은 전역이므로,
      // 카드의 사용자 경로(open-progress)로 진입해 주소에서 확정한다.
      const card = page.getByTestId('repo-card').filter({ hasText: 'stub-account/payments-api' });
      await card.getByTestId('open-progress').click();
      const id = page.url().match(/#\/analyses\/([^/?#]+)/)?.[1];
      expect(id, '진행 화면 주소에서 분석 id를 얻는다').toBeTruthy();

      await scaleWorkers(1);
      await expect
        .poll(() => statusOf(page, id!), { timeout: 120_000, intervals: [1_000] })
        .toBe('awaiting_pipeline');
      // 진행은 서버 상태지만 화면이 자동 갱신하지는 않는다 — 다시 진입해 관측한다.
      await page.goto(`/#/analyses/${id}`);
      // 승인 전에는 4단계가 열리지 않는다(claim이 이미 succeeded인 단계를 내주지 않는
      // 규약은 /internal — backend/tests/worker.rs가 지킨다). 화면의 관측은:
      await expect(page.getByTestId('pipeline-count')).toHaveText('3 of 5');

      // 근거의 유효성은 화면 자신이 아니라 분석된 저장소 기준으로 본다(선례: sc01-03).
      await expect(page.locator('[data-stage="cross_cutting"]')).toContainText('categories');
      await page.getByTestId('open-cross-cutting').click();
      await expect(page.getByTestId('concerns-lede')).toBeVisible();
      const docRes = await page.request.get(`/api/analyses/${id}/documents/cross-cutting`);
      expect(docRes.status(), '횡단 관심사 문서가 생겼다').toBe(200);
      const doc = (await docRes.json()) as {
        content: { categories: Array<{ axis: string; items: Array<{ name: string; evidence: string[] }> }> };
      };
      const items = doc.content.categories.flatMap((c) => c.items);
      expect(items.length, '적어도 하나의 횡단 관심사가 추출돼야 한다').toBeGreaterThan(0);
      for (const item of items) {
        expect(item.evidence.length, `근거 없는 항목: ${item.name}`).toBeGreaterThan(0);
        for (const path of item.evidence) {
          expect(path, `분석 대상 밖의 경로를 근거로 들었다: ${path}`).toContain('payments-api/');
        }
      }
      const shown = doc.content.categories.find((c) => c.items.length > 0)!;
      const axisCard = page.locator(`[data-axis="${shown.axis}"]`);
      await expect(axisCard).toContainText(shown.items[0].name);
      await expect(axisCard).toContainText(shown.items[0].evidence[0]);

      // 화면 진입 자체가 reviewable 전략을 materialise한다(AC1.3의 lazy seed).
      await page.goto(`/#/analyses/${id}`);
      await page.getByTestId('open-discovery-strategy').click();
      const strategyRes = await page.request.get(`/api/analyses/${id}/discovery-strategy`);
      expect(strategyRes.ok(), '탐색 전략이 생겼다').toBeTruthy();
      const strategy = (await strategyRes.json()) as {
        approved: boolean;
        entries: Array<{ pattern: string; source: string }>;
      };
      expect(strategy.approved).toBe(false);
      expect(strategy.entries.length, '제안된 전략 항목이 있다').toBeGreaterThan(0);
      await expect(page.getByTestId('strategy-entry')).toHaveCount(strategy.entries.length);
      for (const entry of strategy.entries) {
        expect(entry.pattern.length, '근거 없는 전략 항목은 없다').toBeGreaterThan(0);
      }
      await expect(page.getByTestId('strategy-entry').first()).toContainText(
        strategy.entries[0].pattern,
      );
      await page.getByTestId('strategy-approve').click();
      await expect(page.getByTestId('strategy-approved')).toBeVisible();

      // 승인이 분석을 재큐잉해 4단계(feature_candidates)를 실행한다 — 워커는
      // 임대 중이라 재청구한다.
      await expect
        .poll(() => candidatesOf(page, id!).then((l) => l.extracted), {
          timeout: 120_000,
          intervals: [1_000],
        })
        .toBe(true);

      // 화면에서 자연스러운 다음 CTA로 간다 — 「순서대로 제시」는 이 경로로 관측한다.
      await page.getByTestId('strategy-open-candidates').click();
      const list = await candidatesOf(page, id!);
      expect(list.candidates.length, '후보가 추출됐다').toBeGreaterThan(0);
      await expect(page.getByTestId('candidate')).toHaveCount(list.candidates.length);
      await expect(page.getByTestId('candidate').first()).toContainText(list.candidates[0].name);
      // 각 후보의 코드 근거(위치)는 API 기준으로 판정한다 — 화면이 그리는 대상의
      // 유효성은 분석된 저장소 기준이므로(선례: sc01-07).
      for (const candidate of list.candidates) {
        expect(
          candidate.location,
          `위치 없는 후보: ${candidate.name}`,
        ).toContain('payments-api/');
      }

      // 성공 단계의 행은 상태 단어가 아니라 **측정값**(detail)으로 그려진다 — 선례:
      // sc01-03의 `categories`, sc01-04의 `entry points`. 그래서 행↔API의 detail을
      // 대조한다(상수를 박지 않는다). 파이프라인 순서 자체는 행의 나열 순서이고,
      // 「문서 → 전략 → 후보」의 제시 순서는 위의 화면 경로가 이미 관측했다.
      await page.goto(`/#/analyses/${id}`);
      await expect(page.getByTestId('pipeline-count')).toHaveText('4 of 5');
      const stages = (await analysisOf(page, id!)).stages;
      const walked = ['fetch', 'cross_cutting', 'discovery_strategy', 'feature_candidates'];
      for (const key of walked) {
        const row = stages.find((s) => s.key === key);
        expect(row, `${key} stage row exists`).toBeTruthy();
        expect(row!.status, `${key} succeeded`).toBe('succeeded');
        expect(row!.detail, `${key} has a measured detail`).toBeTruthy();
        await expect(page.locator(`[data-stage="${key}"]`)).toContainText(row!.detail!);
      }
      // 아직 열리지 않은 5단계는 대기다 — 확정된 후보 결정이 5단계의 몫이다.
      const stage5 = stages.find((s) => s.key === 'acceptance_dependencies');
      expect(stage5?.status, 'stage 5 is pending').toBe('pending');
      await expect(page.locator('[data-stage="acceptance_dependencies"]')).toContainText('대기 중');
    } finally {
      await scaleWorkers(0);
    }
  });
});
