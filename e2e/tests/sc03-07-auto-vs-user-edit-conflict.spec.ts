// 검증 시나리오: 03-doc-management.md#시나리오 7
//
// 사전 조건의 사용자 편집은 API 로 셋업한다 — 3탭 흐름은 sc03-01 의 소유다. 이 파일이
// 단정하는 것은 재분석 뒤의 일이다: 사용자 문장이 자동으로 덮이지 않고, 양쪽이 나란히
// 보이고, 합치기는 제안이며 사람이 확정해야 문서에 선다.
//
// Isolation: leases the worker replica count and one worker env var — rules in
// `e2e/support/cluster.ts`. Signs in as its own stub user (`?as=sc0307`).
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers, setWorkerEnv } from '../support/cluster';
import { acceptanceOf, runToAcceptance, signInWithCredentials } from '../support/acceptance';

const REVISION = 'FEATUREDOC_STUB_REPO_REVISION';

type Sentences = { given: string; when: string; then: string };
type Conflict = {
  id: string;
  featureKey: string;
  status: string;
  mine: Sentences[];
  auto: Sentences;
  mergeProposal: { id: string; status: string; after: Sentences[] } | null;
};

async function conflictsOf(page: Page, id: string): Promise<{ open: number; conflicts: Conflict[] }> {
  const res = await page.request.get(`/api/analyses/${id}/conflicts`);
  expect(res.status(), `conflicts for ${id}`).toBe(200);
  return (await res.json()) as { open: number; conflicts: Conflict[] };
}

async function firstThen(page: Page, id: string, key: string): Promise<string> {
  const doc = (await acceptanceOf(page, id)) ?? [];
  const feature = doc.find((f) => f.key === key);
  expect(feature, `${key} 의 인수 문서`).toBeDefined();
  return feature?.scenarios[0].then ?? '';
}

test.describe('AC3.5: 자동 재분석은 사용자 편집을 결정 없이 덮어쓰지 않는다', () => {
  test.describe.configure({ mode: 'serial', timeout: 600_000 });

  test('충돌은 양쪽이 보이고, 병합은 제안이며, 확정해야 문서에 선다', async ({ page }) => {
    try {
      await scaleWorkers(0);
      setWorkerEnv(REVISION, null);
      await signInWithCredentials(page, 'sc0307');
      await scaleWorkers(1);

      const first = await runToAcceptance(page, 'payments-api', 1);
      const key = first.confirmed[0];
      const autoBefore = await firstThen(page, first.id, key);

      const proposed = await page.request.post(`/api/analyses/${first.id}/features/edit-proposals`, {
        data: { key, scenarioIndex: 0, request: '더 분명하게' },
      });
      expect(proposed.ok(), '편집 제안').toBeTruthy();
      const proposal = (await proposed.json()) as { id: string; after: Sentences[] };
      const mine = proposal.after[0].then;
      expect(mine).not.toBe(autoBefore);
      const approved = await page.request.post(
        `/api/analyses/${first.id}/features/edit-proposals/${proposal.id}/decision`,
        { data: { decision: 'approve' } },
      );
      expect(approved.ok(), '편집 승인').toBeTruthy();
      expect(await firstThen(page, first.id, key), '편집이 문서에 섰다').toBe(mine);

      await scaleWorkers(0);
      setWorkerEnv(REVISION, '3');
      await scaleWorkers(1);

      const second = await runToAcceptance(page, 'payments-api', 1);
      expect(second.confirmed, '같은 자리는 같은 키다').toEqual(first.confirmed);

      const autoNow = await firstThen(page, second.id, key);
      expect(autoNow, '코드가 바뀌었는데 자동 결과가 그대로다').not.toBe(autoBefore);
      expect(autoNow, '결정 없이 사용자 문장이 얹혔다').not.toBe(mine);
      expect(await firstThen(page, first.id, key), '앞선 분석의 사용자 문장은 이력으로 남는다').toBe(
        mine,
      );

      const before = await conflictsOf(page, second.id);
      expect(before.open, '충돌이 서지 않았다').toBe(1);
      const conflict = before.conflicts[0];
      expect(conflict.featureKey).toBe(key);
      expect(conflict.mine[0].then, '사람이 고친 쪽').toBe(mine);
      expect(conflict.auto.then, '자동 결과 쪽').toBe(autoNow);
      expect(conflict.mergeProposal).toBeNull();

      await page.goto(`/#/analyses/${second.id}/diff`);
      await expect(page.getByTestId('diff-conflict-banner')).toBeVisible();
      await expect(page.getByTestId('diff-conflict-count')).toHaveText('1');
      await expect(page.getByTestId('diff-no-conflict')).toHaveCount(0);
      const card = page.locator(`[data-feat="${key}"]`);
      await expect(card).toHaveAttribute('data-conflict', '1');
      await expect(card.getByTestId('diff-feature-tag')).toHaveText(/확인 필요/);
      await expect(page.getByTestId('diff-to-conflict')).toBeDisabled();
      await card.getByTestId('diff-open').click();
      await expect(page.getByTestId('scenario-list')).toBeVisible();
      await page.goto(`/#/analyses/${second.id}/diff`);
      await expect(page.getByTestId('diff-to-conflict')).toBeEnabled();
      await page.getByTestId('diff-to-conflict').click();

      await expect(page).toHaveURL(new RegExp(`/conflicts/${conflict.id}$`));
      await expect(page.getByTestId('conflict-mine')).toHaveText(mine);
      await expect(page.getByTestId('conflict-auto')).toHaveText(autoNow);
      await expect(page.getByTestId('save-decision')).toBeDisabled();

      await page.getByTestId('decide-merge').check();
      await expect(page.getByTestId('merge-panel')).toBeVisible();
      const firstMerge = (await page.getByTestId('merge-added').first().innerText()).replace(
        /^\+\s*/,
        '',
      );
      expect(firstMerge, '내가 보탠 내용이 합친 문장에서 빠졌다').toContain(mine);
      expect(firstMerge, '새 규칙이 합친 문장에 얹히지 않았다').toContain(autoNow);
      expect(await firstThen(page, second.id, key), '확정하지 않았는데 문서가 바뀌었다').toBe(autoNow);
      await expect(page.getByTestId('save-decision')).toBeDisabled();

      await page.getByTestId('reject-merge').click();
      await expect(page.getByTestId('merge-rejected')).toBeVisible();
      await expect(page.getByTestId('merge-panel')).toHaveCount(0);
      expect((await conflictsOf(page, second.id)).open, '버렸는데 충돌이 닫혔다').toBe(1);
      expect(await firstThen(page, second.id, key)).toBe(autoNow);

      await page.getByTestId('decide-merge').check();
      await expect(page.getByTestId('merge-panel')).toBeVisible();
      const secondMerge = (await page.getByTestId('merge-added').first().innerText()).replace(
        /^\+\s*/,
        '',
      );
      expect(secondMerge, '버린 합친 문장이 그대로 다시 왔다').not.toBe(firstMerge);
      expect(secondMerge).toContain(mine);

      await page.getByTestId('approve-merge').click();
      await expect(page.getByTestId('save-decision')).toBeEnabled();
      expect(await firstThen(page, second.id, key), '확정한 합친 문장이 문서에 서지 않았다').toBe(
        secondMerge,
      );

      const after = await conflictsOf(page, second.id);
      expect(after.open).toBe(0);
      expect(after.conflicts[0].status).toBe('merged');
      expect(after.conflicts[0].mergeProposal?.status).toBe('approved');

      await page.getByTestId('save-decision').click();
      await expect(page).toHaveURL(new RegExp(`/analyses/${second.id}/candidates$`));

      await page.goto(`/#/analyses/${second.id}/diff`);
      await expect(page.getByTestId('diff-no-conflict')).toBeVisible();
      await expect(page.getByTestId('diff-to-conflict')).toBeDisabled();
    } finally {
      await scaleWorkers(0);
      setWorkerEnv(REVISION, null);
    }
  });
});
