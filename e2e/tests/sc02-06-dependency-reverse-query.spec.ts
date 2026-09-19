// 검증 시나리오: 02-feature-representation.md#시나리오 6
//
// 되묻는 축을 상수로 박지 않는다 — 두 기능이 공유하는 항목을 트리가 고르게 두고, 그
// 이름으로 되묻는다.
//
// 이 질의에 화면은 없다 — 목업이 그리지 않은 화면을 지어내지 않는다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. Like every spec it signs in as its own stub user (`?as=ac25`).
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { runToAcceptance, signInWithCredentials } from '../support/acceptance';
import { CATEGORIES, traced, type Dependency } from '../support/dependencies';

type Dependent = {
  analysisId: string;
  repoOwner: string;
  repoName: string;
  branch: string;
  featureKey: string;
  featureName: string | null;
  evidence: string | null;
};

async function dependents(page: Page, category: string, name: string): Promise<Dependent[]> {
  const res = await page.request.get(
    `/api/dependencies/features?category=${encodeURIComponent(category)}&name=${encodeURIComponent(name)}`,
  );
  expect(res.status(), `reverse query ${category}/${name}`).toBe(200);
  return (await res.json()).features as Dependent[];
}

test.describe('AC2.5: 의존성은 되물을 수 있는 데이터로 남는다', () => {
  test.describe.configure({ mode: 'serial', timeout: 420_000 });

  test('같은 것에 기대는 feature 를 전부 돌려주고, 분류축이 질의의 일부다', async ({ page }) => {
    try {
      await scaleWorkers(0);
      await signInWithCredentials(page, 'ac25');
      await scaleWorkers(1);

      // 두 기능을 확정하고 둘 다 추적한다 — 「feature 전부」는 하나로는 물을 수 없다.
      const { id, confirmed } = await runToAcceptance(page, 'payments-api', 2);
      expect(confirmed.length).toBe(2);
      const first = await traced(page, id, confirmed[0]);
      const second = await traced(page, id, confirmed[1]);

      const shared = first.find((a: Dependency) =>
        second.some((b) => b.category === a.category && b.name === a.name),
      );
      expect(shared, '두 기능이 공유하는 의존성이 있어야 역방향 질의를 물을 수 있다').toBeTruthy();
      const target = shared as Dependency;

      const found = await dependents(page, target.category, target.name);
      expect(found.length, `${target.category}/${target.name} 를 쓰는 기능`).toBe(2);
      const keys = found.map((f) => f.featureKey).sort();
      expect(keys).toEqual([...confirmed].sort());
      for (const row of found) {
        expect(row.analysisId).toBe(id);
        expect(row.repoOwner).toBe('stub-account');
        expect(row.repoName).toBe('payments-api');
      }

      // 다른 차원 — 같은 이름을 다른 분류로 물으면 그 축의 답이 나온다. 이 이름이 그
      // 분류로도 존재하지 않는 한 답은 비어 있어야 하고, 존재한다면 그 축의 것만 온다.
      const other = (CATEGORIES as readonly string[]).find((c) => c !== target.category) as string;
      const elsewhere = await dependents(page, other, target.name);
      const expected = [first, second].filter((items) =>
        items.some((d) => d.category === other && d.name === target.name),
      ).length;
      expect(elsewhere.length, `${other}/${target.name} 를 쓰는 기능`).toBe(expected);

      // 7종 밖의 분류는 질의 자체가 성립하지 않는다 — 분류축은 자유 텍스트가 아니다.
      const bogus = await page.request.get(
        `/api/dependencies/features?category=wishful&name=${encodeURIComponent(target.name)}`,
      );
      expect(bogus.status()).toBe(400);
    } finally {
      await scaleWorkers(0);
    }
  });
});
