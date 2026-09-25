// 검증 시나리오: 03-doc-management.md#시나리오 6
//
// Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import {
  acceptanceOf,
  candidatesOf,
  runToAcceptance,
  signInWithCredentials,
} from '../support/acceptance';

type Candidate = {
  key: string;
  decision: string;
  mergedInto: string | null;
  previouslyRejected: { reason: string } | null;
  previouslyDeleted: { reason: string | null; analysisId: string } | null;
};

const WHY = '결제 대행사가 그리는 화면이라 우리 문서에 둘 기능이 아니에요';

async function statusOf(page: Page, id: string): Promise<string> {
  const res = await page.request.get(`/api/analyses/${id}`);
  expect(res.ok()).toBeTruthy();
  return (await res.json()).status as string;
}

async function reanalyseToCandidates(page: Page, repo: string): Promise<string> {
  const created = await page.request.post('/api/analyses', {
    data: { repoUrl: `stub-account/${repo}`, branch: null },
  });
  expect(created.status(), `enqueue ${repo}`).toBe(201);
  const id = (await created.json()).id as string;
  await expect
    .poll(() => statusOf(page, id), { timeout: 120_000, intervals: [250] })
    .toBe('awaiting_pipeline');
  expect((await page.request.get(`/api/analyses/${id}/discovery-strategy`)).ok()).toBeTruthy();
  expect(
    (await page.request.post(`/api/analyses/${id}/discovery-strategy/approve`)).ok(),
    'approve the strategy',
  ).toBeTruthy();
  await expect
    .poll(() => candidatesOf(page, id).then((c) => c.length), {
      timeout: 120_000,
      intervals: [250],
    })
    .toBeGreaterThan(0);
  return id;
}

test.describe('AC3.3: 삭제한 feature 가 재분석에서 다시 잡히면 표시만 되고 되살아나지 않는다', () => {
  test.describe.configure({ mode: 'serial', timeout: 600_000 });

  test('같은 저장소 재분석의 후보가 「이전 분석에서 삭제」를 사유와 함께 달고 undecided 로 온다', async ({
    page,
  }) => {
    try {
      await signInWithCredentials(page, 'sc0306');
      await scaleWorkers(1);

      const first = await runToAcceptance(page, 'payments-api', 1);
      const deletedKey = first.confirmed[0];
      const deleted = await page.request.post(`/api/analyses/${first.id}/features/deletions`, {
        data: { key: deletedKey, reason: WHY },
      });
      expect(deleted.status(), 'delete the feature').toBe(200);
      expect(((await acceptanceOf(page, first.id)) ?? []).length).toBe(0);

      const second = await reanalyseToCandidates(page, 'payments-api');
      const again = (await page.request.get(`/api/analyses/${second}/candidates`).then((r) =>
        r.json(),
      )) as { candidates: Candidate[] };
      const carried = again.candidates.find((c) => c.key === deletedKey);
      expect(carried, '같은 위치의 feature 가 재분석에서도 발견되어야 한다').toBeTruthy();
      expect(carried?.previouslyDeleted?.reason).toBe(WHY);
      expect(carried?.previouslyDeleted?.analysisId).toBe(first.id);
      expect(carried?.previouslyRejected, '거부와 삭제는 다른 사건이다').toBeNull();
      expect(carried?.decision, '사용자 확인 없이 다시 활성화됐다').toBe('undecided');
      for (const other of again.candidates.filter((c) => c.key !== deletedKey)) {
        expect(other.previouslyDeleted, `지운 적 없는 자리 ${other.key} 에 표시가 붙었다`).toBeNull();
      }

      await page.goto(`/#/analyses/${second}/candidates`);
      await expect(page.getByTestId('previously-deleted')).toHaveCount(1);
      await expect(page.getByTestId('previously-deleted')).toContainText('삭제');
      await expect(page.getByTestId('previously-deleted')).toContainText(WHY);
      await expect(page.getByTestId('previously-rejected')).toHaveCount(0);

      expect(await acceptanceOf(page, second), '사용자 확인 없이 문서가 섰다').toBeNull();
      expect(((await acceptanceOf(page, first.id)) ?? []).length).toBe(0);
    } finally {
      await scaleWorkers(0);
    }
  });
});
