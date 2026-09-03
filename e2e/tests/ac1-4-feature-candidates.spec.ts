// 검증 AC: AC1.4
//
// AC1.4 (feature 후보 목록 추출) 전용 spec.
//
// AC1.4 의 검증 방법을 그대로 따라간다: 추출 결과가 **후보 목록으로 제시**되고, 사용자가
// **승인·거부·병합·이름 변경** 할 수 있으며, **거부된 후보의 사유는 다음 분석 시 참고될 수
// 있도록 기록된다**(test/01 시나리오 7).
//
// 기대값을 상수로 박지 않는다. "추출된 후보가 이 저장소에서 나온 것인지"는 문서 자신이
// 아니라 **스캔이 본 저장소**를 기준으로 판정해야 의미가 있으므로, 화면·API 가 내려준
// 값들끼리 대조한다. 픽스처를 바꿔도 이 테스트는 여전히 옳고, 4단계가 근거 없는 후보를
// 만들어내는 순간에만 깨진다.
//
// 이 spec 이 브라우저에서 관측하는 또 하나: **승인이 4단계를 실제로 열어 준다**. 승인은
// 분석을 다시 큐에 넣고, 그 다음 claim 은 이미 성공한 단계를 내주지 않는다 — 후자는
// `/internal` 이라 여기서 볼 수 없어 `backend/tests/candidates.rs` 가 지키지만, 전자는
// "승인했더니 후보가 나왔다"로 화면에서 그대로 보인다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs.
//
// Like every spec it signs in as its own stub user (`?as=ac14`).
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';

type Candidate = {
  key: string;
  name: string;
  location: string;
  rationale: string;
  decision: 'undecided' | 'approved' | 'rejected';
  rejectReason: string | null;
  mergedInto: string | null;
  previouslyRejected: { reason: string; analysisId: string } | null;
};
type CandidateList = { candidates: Candidate[]; undecided: number; extracted: boolean };

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

async function candidatesOf(page: Page, id: string): Promise<CandidateList> {
  const res = await page.request.get(`/api/analyses/${id}/candidates`);
  expect(res.status(), `candidates for ${id}`).toBe(200);
  return (await res.json()) as CandidateList;
}

/** Walks one analysis to "the reviewer approved the strategy and stage 4 has run". */
async function runToCandidates(page: Page, repo: string): Promise<string> {
  const id = await enqueue(page, repo);
  await expect
    .poll(() => statusOf(page, id), { timeout: 120_000, intervals: [1_000] })
    .toBe('awaiting_pipeline');
  // Reading materialises the reviewable strategy (AC1.3's lazy seed), then approving
  // is what re-queues the job so stage 4 gets a turn.
  expect((await page.request.get(`/api/analyses/${id}/discovery-strategy`)).ok()).toBeTruthy();
  const approved = await page.request.post(`/api/analyses/${id}/discovery-strategy/approve`);
  expect(approved.ok(), 'approve the strategy').toBeTruthy();
  await expect
    .poll(() => candidatesOf(page, id).then((l) => l.extracted), {
      timeout: 120_000,
      intervals: [1_000],
    })
    .toBe(true);
  return id;
}

test.describe('AC1.4: feature 후보 추출·검토·결정', () => {
  test.describe.configure({ mode: 'serial', timeout: 420_000 });

  test('4단계가 이 저장소에서 나온 후보를 뽑고, 사용자가 결정하면 그 결정이 다음 분석까지 이어진다', async ({
    page,
  }) => {
    try {
      // The overlay already rests at 0; make the precondition explicit so a queued
      // job cannot drain before the "nothing has run yet" assertion below.
      await scaleWorkers(0);

      // ── setup: this spec's own user, App installation, LLM key ──────────
      await page.goto('/api/auth/login?as=ac14');
      expect((await page.request.get('/api/github/setup?installation_id=4242')).ok()).toBeTruthy();
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'openai', key: 'sk-proj-dddddddddddddddddddddd' },
      });
      expect(key.ok(), 'an active LLM key is 분석의 진입 조건').toBeTruthy();

      const first = await enqueue(page, 'payments-api');

      // ── before any worker: there is nothing to sift yet ─────────────────
      // Not a 404 — "아직 추출 전"은 화면이 그릴 수 있는 상태이고, 그 구분이 기능 후보 화면의
      // 빈 상태가 존재하는 이유다.
      const empty = await candidatesOf(page, first);
      expect(empty.extracted).toBe(false);
      expect(empty.undecided).toBe(0);

      // ── let one worker run the job through stage 3 ──────────────────────
      await scaleWorkers(1);
      await expect
        .poll(() => statusOf(page, first), { timeout: 120_000, intervals: [1_000] })
        .toBe('awaiting_pipeline');

      // 승인 전에는 4단계가 열리지 않는다 — 큐가 그 단계를 내주지 않으므로 잡을 아무리
      // 오래 돌려도 후보는 나오지 않는다(AC1.3 의 게이트를 사용자 쪽에서 본 모습).
      expect((await candidatesOf(page, first)).extracted).toBe(false);

      // ── 탐색 전략 화면의 승인 버튼이 곧 기능 후보 화면으로 가는 길이다(목업의 `data-goto`) ─────
      await page.goto(`/#/analyses/${first}/discovery-strategy`);
      await page.getByTestId('strategy-approve').click();
      await expect(page.getByTestId('strategy-approved')).toBeVisible();

      await expect
        .poll(() => candidatesOf(page, first).then((l) => l.extracted), {
          timeout: 120_000,
          intervals: [1_000],
        })
        .toBe(true);

      // ── 추출된 후보는 이 저장소에서 나왔다 ───────────────────────────────
      const extracted = await candidatesOf(page, first);
      expect(extracted.candidates.length, '후보가 없으면 검토할 것이 없다').toBeGreaterThan(0);
      for (const candidate of extracted.candidates) {
        // AC1.4: 각 후보에는 발견된 위치와 추정 근거가 함께 기록된다.
        expect(candidate.location, `분석 대상 밖의 경로: ${candidate.location}`).toContain(
          'payments-api/',
        );
        expect(candidate.rationale.length, `근거 없는 후보: ${candidate.name}`).toBeGreaterThan(0);
        expect(candidate.decision).toBe('undecided');
      }
      expect(extracted.undecided).toBe(extracted.candidates.length);

      // ── 기능 후보 화면이 그 목록을 그린다 ─────────────────────────────────────────
      await page.getByTestId('strategy-open-candidates').click();
      await expect(page.getByTestId('candidate')).toHaveCount(extracted.candidates.length);
      await expect(page.getByTestId('undecided-count')).toHaveText(
        String(extracted.candidates.length),
      );
      await expect(page.getByTestId('candidate').first()).toContainText(
        extracted.candidates[0].name,
      );

      // ── 승인 ────────────────────────────────────────────────────────────
      const approvedKey = extracted.candidates[0].key;
      await page.getByTestId('candidate-approve').first().click();
      await expect(page.locator('[data-testid="candidate"][data-decision="approved"]')).toHaveCount(
        1,
      );
      await expect(page.getByTestId('undecided-count')).toHaveText(
        String(extracted.candidates.length - 1),
      );

      // ── 이름 변경: 이름은 바뀌어도 신원(위치)은 그대로다 ─────────────────
      const renamed = '비밀번호 찾기';
      await page
        .locator('[data-testid="candidate"][data-decision="approved"]')
        .getByTestId('candidate-rename')
        .click();
      await page.getByTestId('rename-input').fill(renamed);
      await page.getByTestId('rename-confirm').click();
      await expect(
        page.locator('[data-testid="candidate"][data-decision="approved"]'),
      ).toContainText(renamed);
      const afterRename = await candidatesOf(page, first);
      const stillThere = afterRename.candidates.find((c) => c.key === approvedKey);
      expect(stillThere?.name, '이름은 바뀌고 키는 그대로여야 한다').toBe(renamed);

      // ── 거부: 사유 없이는 확정할 수 없다 ─────────────────────────────────
      const rejectedKey = afterRename.candidates.filter((c) => c.key !== approvedKey)[0].key;
      await page
        .locator(`[data-testid="candidate"][data-key="${cssEscape(rejectedKey)}"]`)
        .getByTestId('candidate-reject')
        .click();
      await expect(page.getByTestId('reject-guard')).toBeVisible();
      await expect(page.getByTestId('reject-confirm')).toBeDisabled();
      // 화면만의 규칙이 아니다 — 서버도 같은 이유로 거절한다.
      const reasonless = await page.request.post(`/api/analyses/${first}/candidates/decision`, {
        data: { key: rejectedKey, decision: 'reject' },
      });
      expect(reasonless.status(), '사유 없는 거부는 서버가 400 으로 막는다').toBe(400);

      const why = '내부 도구라 사용자 기능이 아님';
      await page.getByTestId('reject-reason').fill(why);
      await page.getByTestId('reject-confirm').click();
      await expect(page.locator('[data-testid="candidate"][data-decision="rejected"]')).toHaveCount(
        1,
      );

      // ── 병합: 접힌 후보는 사라지지 않고 「합쳐졌다」로 남는다 ─────────────
      const beforeMerge = await candidatesOf(page, first);
      const open = beforeMerge.candidates.filter(
        (c) => c.decision === 'undecided' && c.mergedInto === null,
      );
      if (open.length >= 2) {
        const merged = await page.request.post(`/api/analyses/${first}/candidates/merge`, {
          data: { into: open[0].key, keys: [open[1].key] },
        });
        expect(merged.ok(), 'merge').toBeTruthy();
        const afterMerge = await candidatesOf(page, first);
        expect(afterMerge.candidates.find((c) => c.key === open[1].key)?.mergedInto).toBe(
          open[0].key,
        );
        expect(afterMerge.undecided).toBe(beforeMerge.undecided - 1);
      }

      // 화면이 아니라 서버가 기억한다 — 새로고침이 곧 그 증거다.
      await page.reload();
      await expect(page.locator('[data-testid="candidate"][data-decision="rejected"]')).toHaveCount(
        1,
      );
      await expect(page.locator('[data-testid="candidate"][data-decision="approved"]')).toContainText(
        renamed,
      );

      // ── 거부 사유는 같은 대상의 다음 분석으로 이어진다 (시나리오 7) ───────
      const second = await runToCandidates(page, 'payments-api');
      const again = await candidatesOf(page, second);
      const carried = again.candidates.find((c) => c.key === rejectedKey);
      expect(carried, '같은 위치의 후보가 재분석에서도 발견되어야 한다').toBeTruthy();
      expect(carried?.previouslyRejected?.reason).toBe(why);
      expect(carried?.previouslyRejected?.analysisId).toBe(first);
      // 자동으로 다시 거부하지 않는다 — 화면이 그렇게 약속했다.
      expect(carried?.decision).toBe('undecided');

      await page.goto(`/#/analyses/${second}/candidates`);
      await expect(page.getByTestId('previously-rejected').first()).toContainText(why);
    } finally {
      // The worker is leased, not owned — hand it back whatever happened above.
      await scaleWorkers(0);
    }
  });
});

/** `data-key` holds a repository path, so quote it for the attribute selector. */
function cssEscape(value: string): string {
  return value.replace(/["\\]/g, '\\$&');
}
