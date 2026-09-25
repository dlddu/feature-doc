// 검증 시나리오: 03-doc-management.md#시나리오 8
//
// Isolation: leases the worker replica count and one worker env var — rules in
// `e2e/support/cluster.ts`. Signs in as its own stub user (`?as=sc0308`).
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers, setWorkerEnv } from '../support/cluster';
import { acceptanceOf, runToAcceptance, signInWithCredentials } from '../support/acceptance';

/** 세 번째 리비전 — 트리가 한 걸음 더 나아가고 첫 문장을 다르게 읽는다(EXT-03 더블 안). */
const REVISION = 'FEATUREDOC_STUB_REPO_REVISION';

type Sentences = { given: string; when: string; then: string };
type Conflict = { id: string; featureKey: string; status: string; mine: Sentences[] };

async function firstThen(page: Page, id: string, key: string): Promise<string> {
  const doc = (await acceptanceOf(page, id)) ?? [];
  return doc.find((f) => f.key === key)?.scenarios[0].then ?? '';
}

/** 한 줄 부탁 → 제안 → 승인. 승인된 문장을 돌려준다(출처 `user_llm`). */
async function editWithHelp(page: Page, id: string, key: string, request: string): Promise<string> {
  const proposed = await page.request.post(`/api/analyses/${id}/features/edit-proposals`, {
    data: { key, scenarioIndex: 0, request },
  });
  expect(proposed.ok(), '편집 제안').toBeTruthy();
  const proposal = (await proposed.json()) as { id: string; after: Sentences[] };
  const approved = await page.request.post(
    `/api/analyses/${id}/features/edit-proposals/${proposal.id}/decision`,
    { data: { decision: 'approve' } },
  );
  expect(approved.ok(), '편집 승인').toBeTruthy();
  return proposal.after[0].then;
}

test.describe('AC3.4: 변경마다 출처가 남고, 고른 시점으로 정확히 되돌릴 수 있다', () => {
  test.describe.configure({ mode: 'serial', timeout: 600_000 });

  test('출처 3종이 구분되어 보이고, 두 번째 변경 시점으로 복원된다', async ({ page }) => {
    try {
      await scaleWorkers(0);
      setWorkerEnv(REVISION, null);
      await signInWithCredentials(page, 'sc0308');
      await scaleWorkers(1);

      // ── 변경 ①: 자동 분석이 쓴 문서 ────────────────────────────────
      const first = await runToAcceptance(page, 'payments-api', 1);
      const key = first.confirmed[0];
      const mine = await editWithHelp(page, first.id, key, '더 분명하게');

      // ── 변경 ②: 재분석 충돌을 「내 문장 유지」로 — 출처 `user_direct` ──
      await scaleWorkers(0);
      setWorkerEnv(REVISION, '3');
      await scaleWorkers(1);
      const second = await runToAcceptance(page, 'payments-api', 1);
      const auto = await firstThen(page, second.id, key);
      expect(auto, '코드가 바뀌었는데 자동 결과가 그대로다').not.toBe(mine);

      const open = (await (await page.request.get(`/api/analyses/${second.id}/conflicts`)).json()) as {
        conflicts: Conflict[];
      };
      expect(open.conflicts.length, '충돌이 서지 않았다').toBe(1);
      const decided = await page.request.post(
        `/api/analyses/${second.id}/conflicts/${open.conflicts[0].id}/decision`,
        { data: { decision: 'mine' } },
      );
      expect(decided.ok(), '충돌 결정').toBeTruthy();
      const direct = await firstThen(page, second.id, key);
      expect(direct, '내 문장이 다시 서지 않았다').toBe(mine);

      // ── 변경 ③: 그 위에 한 번 더 LLM 보조 수정 ──────────────────────
      const helped = await editWithHelp(page, second.id, key, '한 문장 더 분명하게');
      expect(helped).not.toBe(direct);
      expect(await firstThen(page, second.id, key)).toBe(helped);

      // ── 이력 진입점은 기능 상세에 있다(여정 `STP-open-history`) ────────
      await page.goto(`/#/analyses/${second.id}/acceptance`);
      await page.getByTestId('feature-select').selectOption(key);
      await page.getByTestId('open-history').click();
      await expect(page).toHaveURL(/\/history$/);

      // ── 출처가 구분되어 보인다 ────────────────────────────────────
      const entries = page.getByTestId('history-entry');
      await expect(entries).toHaveCount(3);
      await expect(page.getByTestId('history-count')).toHaveText('3');
      const sources = await entries.evaluateAll((nodes) =>
        nodes.map((n) => (n as HTMLElement).dataset.source ?? ''),
      );
      expect([...sources].sort(), '세 변경의 출처가 구분되지 않는다').toEqual([
        'auto',
        'user_direct',
        'user_llm',
      ]);
      const secondChange = entries.nth(1);
      await expect(secondChange).toHaveAttribute('data-source', 'user_direct');
      await expect(entries.nth(0)).toHaveAttribute('data-current', 'true');

      // ── 되돌리기 전에 무엇이 달라지는지 먼저 본다 ──────────────────
      await secondChange.getByTestId('pick-point').click();
      await expect(page.getByTestId('restore-preview')).toBeVisible();
      await expect(page.getByTestId('preview-same')).toHaveCount(0);
      const removed = await page.getByTestId('preview-removed').allTextContents();
      const added = await page.getByTestId('preview-added').allTextContents();
      expect(removed.join(' '), '미리보기가 지금 문장을 덜어 내지 않는다').toContain(helped);
      expect(added.join(' '), '미리보기가 그 시점의 문장을 세우지 않는다').toContain(direct);
      expect(await firstThen(page, second.id, key), '미리보기가 문서를 이미 바꿨다').toBe(helped);

      // ── 복원: 고른 시점의 상태로 정확히 ────────────────────────────
      await page.getByTestId('restore-point').click();
      await expect(page.getByTestId('history-count')).toHaveText('4');
      expect(await firstThen(page, second.id, key), '고른 시점으로 복원되지 않았다').toBe(direct);

      const after = page.getByTestId('history-entry');
      await expect(after.nth(0)).toHaveAttribute('data-kind', 'restore');
      await expect(after.nth(0)).toHaveAttribute('data-current', 'true');
      await expect(after.nth(1)).toHaveAttribute('data-standing', 'false');
      await expect(after.nth(1).getByTestId('entry-cut')).toBeVisible();

      // ── 되돌린 것도 되돌릴 수 있다 ────────────────────────────────
      await after.nth(1).getByTestId('pick-point').click();
      await page.getByTestId('restore-point').click();
      await expect(page.getByTestId('history-count')).toHaveText('5');
      expect(await firstThen(page, second.id, key), '복원을 되돌리지 못했다').toBe(helped);

      await page.getByTestId('history-entry').nth(0).getByTestId('pick-point').click();
      await expect(page.getByTestId('preview-same')).toBeVisible();
      await expect(page.getByTestId('restore-point')).toBeDisabled();
      await page.getByTestId('keep-current').click();
      await expect(page.getByTestId('restore-preview')).toHaveCount(0);
    } finally {
      await scaleWorkers(0);
      setWorkerEnv(REVISION, null);
    }
  });
});
