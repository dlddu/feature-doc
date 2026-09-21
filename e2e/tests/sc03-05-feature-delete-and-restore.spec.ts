// 검증 시나리오: 03-doc-management.md#시나리오 5
//
// 「보관소로 이동하며, 일정 기간 내 복구 가능. 즉시 영구 삭제되지 않는다」를 **관측**한다 —
// 지운 feature 는 인수 문서에서 사라지되 보관소 목록에 사유·기한과 함께 남고, 되돌리면
// 문서에 다시 선다. 기한은 화면이 아니라 서버가 지운 시점에 적은 값이다(`restoreUntil`).
// 「즉시 영구 삭제되지 않는다」의 관측은 되돌리기가 성립한다는 것 자체다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. Like every spec it signs in as its own stub user (`?as=…`).
import { expect, test } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { acceptanceOf, runToAcceptance, signInWithCredentials } from '../support/acceptance';

type Deletion = {
  id: string;
  key: string;
  name: string;
  reason: string | null;
  deletedAt: number;
  restoreUntil: number;
  restoredAt: number | null;
  restorable: boolean;
};

const WHY = '이건 결제 대행사 화면이라 우리 제품의 기능이 아니에요';

test.describe('AC3.3: feature 문서는 삭제해도 보관소로 옮겨지고 기간 안에 되돌릴 수 있다', () => {
  test.describe.configure({ mode: 'serial', timeout: 600_000 });

  test('삭제 탭 → 사유 → 확인이 feature 를 보관소로 옮기고, 되돌리기가 문서에 되살린다', async ({
    page,
  }) => {
    try {
      await signInWithCredentials(page, 'sc0305');
      await scaleWorkers(1);

      const run = await runToAcceptance(page, 'payments-api', 1);
      const before = (await acceptanceOf(page, run.id)) ?? [];
      expect(before.length, '등록된 feature 1개').toBe(1);
      const feature = before[0];

      await page.goto(`/#/analyses/${run.id}/acceptance`);
      await expect(page.getByTestId('feature-title')).toHaveText(feature.name);
      await expect(page.getByTestId('archive-list')).toHaveCount(0);

      // "삭제" 탭 → 사유 입력 → 확인.
      await page.getByTestId('delete-feature').click();
      await expect(page.getByTestId('delete-confirm')).toBeVisible();
      await page.getByTestId('delete-reason').fill(WHY);
      await page.getByTestId('confirm-delete').click();

      // 문서에서는 사라지고 보관소에 선다 — 사유와 기한을 달고.
      await expect(page.getByTestId('archive-list')).toBeVisible();
      await expect(page.getByTestId('archived-feature')).toHaveCount(1);
      await expect(page.getByTestId('archived-name')).toHaveText(feature.name);
      await expect(page.getByTestId('archived-reason')).toHaveText(WHY);
      await expect(page.getByTestId('feature-select')).toHaveCount(0);

      // 저장이 기준이다 — 문서는 여전히 있되(404 가 아니다) 그 feature 만 가려졌다.
      const hidden = await acceptanceOf(page, run.id);
      expect(hidden, '문서 자체가 사라졌다 — 자동 문서를 고쳤다는 뜻이다').not.toBeNull();
      expect(hidden?.length).toBe(0);

      const listed = await page.request.get(`/api/analyses/${run.id}/features/deletions`);
      expect(listed.status()).toBe(200);
      const body = (await listed.json()) as { retentionDays: number; deletions: Deletion[] };
      expect(body.deletions.length).toBe(1);
      const record = body.deletions[0];
      expect(record.key).toBe(feature.key);
      expect(record.reason).toBe(WHY);
      expect(record.restoredAt).toBeNull();
      expect(record.restorable, '즉시 영구 삭제되지 않는다 — 되돌릴 수 있어야 한다').toBe(true);
      // 「일정 기간」은 지운 시점의 사실이다 — 기한이 지운 시각보다 뒤에, 보관 일수만큼 떨어져 있다.
      expect(body.retentionDays).toBeGreaterThan(0);
      expect(record.restoreUntil - record.deletedAt).toBe(body.retentionDays * 24 * 60 * 60);
      await expect(page.getByTestId('restore-until')).toContainText(
        new Date(record.restoreUntil * 1000).toISOString().slice(0, 10),
      );

      // 새로고침해도 같다 — 보관소는 서버 상태다.
      await page.reload();
      await expect(page.getByTestId('archived-feature')).toHaveCount(1);

      // 되돌리기 → 문서에 다시 선다.
      await page.getByTestId('restore-feature').click();
      await expect(page.getByTestId('feature-select')).toBeVisible();
      await expect(page.getByTestId('feature-title')).toHaveText(feature.name);
      await expect(page.getByTestId('archive-list')).toHaveCount(0);

      const restored = (await acceptanceOf(page, run.id)) ?? [];
      expect(restored.length).toBe(1);
      expect(restored[0].key).toBe(feature.key);
      expect(restored[0].scenarios.length, '되돌린 feature 의 시나리오가 줄었다').toBe(
        feature.scenarios.length,
      );

      // 되돌린 뒤 보관소는 비고, 그 행은 닫힌 채 남는다(두 번 되돌리기는 거부).
      const after = (await (
        await page.request.get(`/api/analyses/${run.id}/features/deletions`)
      ).json()) as { deletions: Deletion[] };
      expect(after.deletions.length).toBe(0);
      const again = await page.request.post(
        `/api/analyses/${run.id}/features/deletions/${record.id}/restore`,
        { data: {} },
      );
      expect(again.status()).toBe(409);
    } finally {
      await scaleWorkers(0);
    }
  });
});
