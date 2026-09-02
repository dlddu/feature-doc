// 검증 AC: AC1.3
//
// AC1.3 (feature 탐색 전략 생성) 전용 spec.
//
// AC1.3 의 검증 방법을 그대로 따라간다: 자동 생성된 전략을 사용자가 **검토·수정·승인**
// 할 수 있고, **승인된 전략만 다음 단계의 입력이 된다**.
//
// 기대값을 상수로 박지 않는다. "제안된 패턴이 이 저장소에서 나온 것인지"는 문서 자신이
// 아니라 **스캔이 본 저장소**를 기준으로 판정해야 의미가 있으므로, 화면·API 가 내려준
// 값들끼리 대조한다. 픽스처를 바꿔도 이 테스트는 여전히 옳고, 3단계가 근거 없는 패턴을
// 만들어내는 순간에만 깨진다.
//
// 승인 게이트는 화면 밖에서도 확인한다 — 승인 전/후로 `PUT` 이 409 로 바뀌는 것이
// "승인된 전략은 더 이상 흔들리지 않는다"의 관측 가능한 형태다. 큐가 다음 단계를
// 내주는지(`executableStages`)는 `/internal` 이라 브라우저에서 못 보므로
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

      // ── setup: this spec's own user, App installation, LLM key ──────────
      await page.goto('/api/auth/login?as=ac13');
      expect((await page.request.get('/api/github/setup?installation_id=4242')).ok()).toBeTruthy();
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'openai', key: 'sk-proj-cccccccccccccccccccccc' },
      });
      expect(key.ok(), 'an active LLM key is 분석의 진입 조건').toBeTruthy();

      const first = await enqueue(page, 'payments-api');

      // ── before any worker: there is no strategy to review yet ───────────
      // 404, not an empty list: "아직 제안되지 않았다" 와 "제안했는데 비었다" 는
      // 사용자에게 다른 상태다.
      expect((await page.request.get(`/api/analyses/${first}/discovery-strategy`)).status()).toBe(
        404,
      );

      // ── let one worker run the job through stage 3 ──────────────────────
      await scaleWorkers(1);
      await expect
        .poll(() => statusOf(page, first), { timeout: 120_000, intervals: [1_000] })
        .toBe('awaiting_pipeline');

      // ── 제안된 전략은 이 저장소에서 나왔다 ───────────────────────────────
      // 근거의 유효성은 문서 자신이 아니라 **분석된 저장소**를 기준으로 본다.
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

      // ── S06 이 그 전략을 그린다 ─────────────────────────────────────────
      await page.goto(`/#/analyses/${first}/discovery-strategy`);
      await expect(page.getByTestId('strategy-entry')).toHaveCount(proposed.entries.length);
      await expect(page.getByTestId('strategy-count')).toHaveText(
        String(proposed.entries.length),
      );
      await expect(page.getByTestId('strategy-notice')).toContainText('승인 전까지');
      await expect(page.getByTestId('strategy-entry').first()).toContainText(
        proposed.entries[0].pattern,
      );

      // ── 검토: 지운다 ────────────────────────────────────────────────────
      const dropped = proposed.entries[0].pattern;
      await page.getByTestId('strategy-drop').first().click();
      await expect(page.getByTestId('strategy-entry')).toHaveCount(proposed.entries.length - 1);
      // 정확 일치로 본다 — 다중 요소 로케이터에 `.not.toContainText` 를 걸면 strict mode
      // 위반이고, 부분 문자열 대조는 `payments-api/**` 처럼 서로 접두사인 패턴에서 공허해진다.
      const shown = await page.locator('[data-testid="strategy-entry"] .sname').allTextContents();
      expect(shown).not.toContain(dropped);

      // ── 검토: 보탠다 ────────────────────────────────────────────────────
      const mine = 'cmd/admin-cli';
      await page.getByTestId('strategy-input').fill(mine);
      await page.getByTestId('strategy-add').click();
      const added = page.locator('[data-testid="strategy-entry"][data-source="user"]');
      await expect(added).toHaveCount(1);
      await expect(added).toContainText(mine);

      // 화면이 아니라 서버가 기억한다 — 새로고침이 곧 그 증거다.
      await page.reload();
      await expect(
        page.locator('[data-testid="strategy-entry"][data-source="user"]'),
      ).toContainText(mine);
      const edited = await strategyOf(page, first);
      expect(edited.entries.map((e) => e.pattern)).not.toContain(dropped);
      expect(edited.entries.map((e) => e.pattern)).toContain(mine);
      expect(edited.approved).toBe(false);

      // ── 승인 ────────────────────────────────────────────────────────────
      await page.getByTestId('strategy-approve').click();
      await expect(page.getByTestId('strategy-approved')).toBeVisible();
      // 승인 뒤에는 수정 액션이 화면에서 사라진다.
      await expect(page.getByTestId('strategy-drop')).toHaveCount(0);
      await expect(page.getByTestId('strategy-add')).toHaveCount(0);
      expect((await strategyOf(page, first)).approved).toBe(true);

      // …그리고 API 로도 잠긴다: 승인된 전략은 다음 단계의 입력이므로 흔들리면 안 된다.
      const late = await page.request.put(`/api/analyses/${first}/discovery-strategy/entries`, {
        data: { patterns: ['something/else'] },
      });
      expect(late.status(), '승인된 전략은 수정할 수 없다').toBe(409);

      // ── S04 → S06 사용자 경로 ───────────────────────────────────────────
      await page.goto(`/#/analyses/${first}`);
      await expect(page.locator('[data-stage="discovery_strategy"]')).toContainText('entry points');
      await page.getByTestId('open-discovery-strategy').click();
      await expect(page.getByTestId('strategy-approved')).toBeVisible();

      // ── 보탠 항목은 같은 대상의 다음 분석에도 이어진다 ───────────────────
      // S06 이 그렇게 적어 두었으므로(「여기서 보탠 항목은 다음 분석에서도 그대로
      // 참조됩니다」) 그 문장이 참인지 실제로 확인한다.
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
      // The worker is leased, not owned — hand it back whatever happened above.
      await scaleWorkers(0);
    }
  });
});
