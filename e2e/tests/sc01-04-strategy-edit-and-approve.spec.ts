// 검증 시나리오: 01-analysis-pipeline.md#시나리오 4
//
// 기대값을 상수로 박지 않는다. "제안된 패턴이 이 저장소에서 나온 것인지"는 문서 자신이
// 아니라 **스캔이 본 저장소**를 기준으로 판정해야 의미가 있으므로, 화면·API 가 내려준
// 값들끼리 대조한다. 픽스처를 바꿔도 이 테스트는 여전히 옳고, 3단계가 근거 없는 패턴을
// 만들어내는 순간에만 깨진다.
//
// 큐가 다음 단계를 내주는지(`executableStages`)는 `/internal` 이라 브라우저에서 못 보므로
// `backend/tests/strategy.rs` 가 대신 지킨다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs.
//
// Like every spec it signs in as its own stub user (`?as=ac13`).
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';

type Entry = { pattern: string; source: 'generated' | 'user' };
type Strategy = { entries: Entry[]; approved: boolean };
type Proposal = { content: { entries: { pattern: string; evidence: string[] }[] } };

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

async function strategyOf(page: Page, id: string): Promise<Strategy> {
  const res = await page.request.get(`/api/analyses/${id}/discovery-strategy`);
  expect(res.status(), `strategy for ${id}`).toBe(200);
  return (await res.json()) as Strategy;
}

test.describe('AC1.3: feature 탐색 전략 생성·검토·수정·승인', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('3단계가 이 저장소에서 나온 전략을 제안하고, 사용자가 고쳐 승인하면 그 전략이 확정된다', async ({
    page,
  }) => {
    try {
      // The overlay already rests at 0; make the precondition explicit so a queued
      // job cannot drain before the "nothing has run yet" assertion below.
      await scaleWorkers(0);

      await page.goto('/api/auth/login?as=ac13');
      expect((await page.request.get('/api/github/setup?installation_id=4242')).ok()).toBeTruthy();
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'openai', key: 'sk-proj-cccccccccccccccccccccc' },
      });
      expect(key.ok(), 'an active LLM key is 분석의 진입 조건').toBeTruthy();

      const first = await enqueue(page, 'payments-api');

      expect((await page.request.get(`/api/analyses/${first}/discovery-strategy`)).status()).toBe(
        404,
      );

      await scaleWorkers(1);
      await expect
        .poll(() => statusOf(page, first), { timeout: 120_000, intervals: [1_000] })
        .toBe('awaiting_pipeline');

      const proposal = (await (
        await page.request.get(`/api/analyses/${first}/documents/discovery-strategy`)
      ).json()) as Proposal;
      expect(proposal.content.entries.length, '전략이 비어 있으면 검토할 것이 없다').toBeGreaterThan(
        0,
      );
      for (const entry of proposal.content.entries) {
        expect(entry.evidence.length, `근거 없는 항목: ${entry.pattern}`).toBeGreaterThan(0);
        for (const path of entry.evidence) {
          expect(path, `분석 대상 밖의 경로를 근거로 들었다: ${path}`).toContain('payments-api/');
        }
      }

      const proposed = await strategyOf(page, first);
      expect(proposed.approved).toBe(false);
      expect(proposed.entries.every((e) => e.source === 'generated')).toBeTruthy();
      expect(proposed.entries.map((e) => e.pattern)).toEqual(
        proposal.content.entries.map((e) => e.pattern),
      );

      await page.goto(`/#/analyses/${first}/discovery-strategy`);
      await expect(page.getByTestId('strategy-entry')).toHaveCount(proposed.entries.length);
      await expect(page.getByTestId('strategy-count')).toHaveText(
        String(proposed.entries.length),
      );
      await expect(page.getByTestId('strategy-notice')).toContainText('승인 전까지');
      await expect(page.getByTestId('strategy-entry').first()).toContainText(
        proposed.entries[0].pattern,
      );

      const dropped = proposed.entries[0].pattern;
      await page.getByTestId('strategy-drop').first().click();
      await expect(page.getByTestId('strategy-entry')).toHaveCount(proposed.entries.length - 1);
      // 정확 일치로 본다 — 다중 요소 로케이터에 `.not.toContainText` 를 걸면 strict mode
      // 위반이고, 부분 문자열 대조는 `payments-api/**` 처럼 서로 접두사인 패턴에서 공허해진다.
      const shown = await page.locator('[data-testid="strategy-entry"] .sname').allTextContents();
      expect(shown).not.toContain(dropped);

      const mine = 'cmd/admin-cli';
      await page.getByTestId('strategy-input').fill(mine);
      await page.getByTestId('strategy-add').click();
      const added = page.locator('[data-testid="strategy-entry"][data-source="user"]');
      await expect(added).toHaveCount(1);
      await expect(added).toContainText(mine);

      await page.reload();
      await expect(
        page.locator('[data-testid="strategy-entry"][data-source="user"]'),
      ).toContainText(mine);
      const edited = await strategyOf(page, first);
      expect(edited.entries.map((e) => e.pattern)).not.toContain(dropped);
      expect(edited.entries.map((e) => e.pattern)).toContain(mine);
      expect(edited.approved).toBe(false);

      await page.getByTestId('strategy-approve').click();
      await expect(page.getByTestId('strategy-approved')).toBeVisible();
      await expect(page.getByTestId('strategy-drop')).toHaveCount(0);
      await expect(page.getByTestId('strategy-add')).toHaveCount(0);
      expect((await strategyOf(page, first)).approved).toBe(true);

      const late = await page.request.put(`/api/analyses/${first}/discovery-strategy/entries`, {
        data: { patterns: ['something/else'] },
      });
      expect(late.status(), '승인된 전략은 수정할 수 없다').toBe(409);

      await page.goto(`/#/analyses/${first}`);
      await expect(page.locator('[data-stage="discovery_strategy"]')).toContainText('entry points');
      await page.getByTestId('open-discovery-strategy').click();
      await expect(page.getByTestId('strategy-approved')).toBeVisible();

      const second = await enqueue(page, 'payments-api');
      await expect
        .poll(() => statusOf(page, second), { timeout: 120_000, intervals: [1_000] })
        .toBe('awaiting_pipeline');

      const carried = await strategyOf(page, second);
      expect(carried.approved).toBe(false);
      const carriedMine = carried.entries.filter((e) => e.source === 'user');
      expect(carriedMine.map((e) => e.pattern)).toEqual([mine]);
      // 새 분석의 제안은 그 분석의 것이다 — 직전에 지운 항목이 여기서 다시 나타나도
      // 이상하지 않다(같은 트리에 대한 새 제안이다). 이어받는 것은 사용자가 보탠 것뿐이다.
      expect(carried.entries.filter((e) => e.source === 'generated').length).toBeGreaterThan(0);

      await page.goto(`/#/analyses/${second}/discovery-strategy`);
      await expect(
        page.locator('[data-testid="strategy-entry"][data-source="user"]'),
      ).toContainText(mine);
    } finally {
      await scaleWorkers(0);
    }
  });
});
