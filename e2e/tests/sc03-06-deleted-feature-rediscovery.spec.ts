// 검증 시나리오: 03-doc-management.md#시나리오 6
//
// 「동일 feature 가 자동으로 발견되더라도 '이전에 거부/삭제된 항목입니다' 로 표시되며,
// 사용자 확인 없이는 다시 활성화되지 않는다」를 **관측**한다. 같은 저장소의 두 번째 자동
// 분석이 같은 자리(키 = 발견 위치)를 후보로 잡으면, 그 후보가 앞선 분석의 삭제와 사유를
// 달고 오되 결정은 `undecided` 이고, 인수 문서는 사람이 승인하기 전까지 서지 않는다.
//
// 사전 조건(시나리오 5 의 삭제)은 API 로 만든다 — 다른 시나리오의 화면을 걷는 것은 셋업이지
// 검증이 아니다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. Like every spec it signs in as its own stub user (`?as=…`).
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

/** 같은 저장소의 자동 재분석 — 4단계(후보 추출)까지만 걷고 아무것도 승인하지 않는다. */
async function reanalyseToCandidates(page: Page, repo: string): Promise<string> {
  const created = await page.request.post('/api/analyses', {
    data: { repoUrl: `stub-account/${repo}`, branch: null },
  });
  expect(created.status(), `enqueue ${repo}`).toBe(201);
  const id = (await created.json()).id as string;
  await expect
    .poll(() => statusOf(page, id), { timeout: 120_000, intervals: [1_000] })
    .toBe('awaiting_pipeline');
  expect((await page.request.get(`/api/analyses/${id}/discovery-strategy`)).ok()).toBeTruthy();
  expect(
    (await page.request.post(`/api/analyses/${id}/discovery-strategy/approve`)).ok(),
    'approve the strategy',
  ).toBeTruthy();
  await expect
    .poll(() => candidatesOf(page, id).then((c) => c.length), {
      timeout: 120_000,
      intervals: [1_000],
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

      // 사전 조건: 시나리오 5 에서 삭제된 feature.
      const first = await runToAcceptance(page, 'payments-api', 1);
      const deletedKey = first.confirmed[0];
      const deleted = await page.request.post(`/api/analyses/${first.id}/features/deletions`, {
        data: { key: deletedKey, reason: WHY },
      });
      expect(deleted.status(), 'delete the feature').toBe(200);
      expect(((await acceptanceOf(page, first.id)) ?? []).length).toBe(0);

      // 실행 단계: 같은 저장소 자동 재분석.
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

      // 화면이 그 표시를 그린다.
      await page.goto(`/#/analyses/${second}/candidates`);
      await expect(page.getByTestId('previously-deleted')).toHaveCount(1);
      await expect(page.getByTestId('previously-deleted')).toContainText('삭제');
      await expect(page.getByTestId('previously-deleted')).toContainText(WHY);
      await expect(page.getByTestId('previously-rejected')).toHaveCount(0);

      // 승인하지 않았으므로 재분석의 인수 문서는 서지 않는다 — 되살아난 것이 없다.
      expect(await acceptanceOf(page, second), '사용자 확인 없이 문서가 섰다').toBeNull();
      // 첫 분석의 삭제도 그대로다.
      expect(((await acceptanceOf(page, first.id)) ?? []).length).toBe(0);
    } finally {
      await scaleWorkers(0);
    }
  });
});
