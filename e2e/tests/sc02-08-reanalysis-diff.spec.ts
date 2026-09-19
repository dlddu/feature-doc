// 검증 시나리오: 02-feature-representation.md#시나리오 8
//
// Isolation: this spec *leases* two pieces of deployment-wide state — the worker
// replica count and one worker env var (see `e2e/support/cluster.ts`). Both are set
// inside its own block and returned in `finally`; `playwright.config.ts` pins
// `workers: 1`, so no sibling spec file is in flight while it runs. Like every spec
// it signs in as its own stub user (`?as=ac26`).
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers, setWorkerEnv } from '../support/cluster';
import { runToAcceptance, signInWithCredentials } from '../support/acceptance';
import { dependenciesOf, traced } from '../support/dependencies';

/** 두 번째 분석이 보는 트리를 한 걸음 나아가게 하는 스위치(EXT-03 더블 안). */
const REVISION = 'FEATUREDOC_STUB_REPO_REVISION';

type ScenarioLine = { mark: string; text: string };
type DependencyLine = { mark: string; category: string; name: string };
type FeatureDiff = {
  key: string;
  name: string;
  location: string | null;
  scenarios: number;
  scenarioLines: ScenarioLine[];
  dependencyLines: DependencyLine[];
};
type AnalysisDiff = {
  comparedTo: string | null;
  changedLineCount: number;
  features: FeatureDiff[];
};

async function diffOf(page: Page, id: string): Promise<AnalysisDiff> {
  const res = await page.request.get(`/api/analyses/${id}/diff`);
  expect(res.status(), `diff for ${id}`).toBe(200);
  return (await res.json()) as AnalysisDiff;
}

test.describe('AC2.6: 코드가 바뀐 뒤 재분석하면 달라진 것만 diff 로 보인다', () => {
  test.describe.configure({ mode: 'serial', timeout: 600_000 });

  test('같은 타깃의 직전 분석과 견주어, 달라진 기능만 추가·제거 줄과 함께 보인다', async ({
    page,
  }) => {
    try {
      await scaleWorkers(0);
      setWorkerEnv(REVISION, null);
      await signInWithCredentials(page, 'sc0208');
      await scaleWorkers(1);

      const first = await runToAcceptance(page, 'payments-api', 2);
      const key = first.confirmed[0];
      const untouched = first.confirmed[1];
      const before = await traced(page, first.id, key);
      expect(before.length, '첫 분석에서 의존성이 하나도 나오지 않았다').toBeGreaterThan(0);

      const firstDiff = await diffOf(page, first.id);
      expect(firstDiff.comparedTo, '이 저장소의 첫 분석이다').toBeNull();
      expect(firstDiff.features).toHaveLength(0);

      await scaleWorkers(0);
      setWorkerEnv(REVISION, '2');
      await scaleWorkers(1);

      const second = await runToAcceptance(page, 'payments-api', 2);
      expect(
        second.confirmed,
        'feature 의 정체성은 분석을 가로지른다 — 같은 자리는 같은 키다',
      ).toEqual(first.confirmed);
      const after = await traced(page, second.id, key);

      // 무엇이 늘었는지는 이 파일이 아니라 두 추적 결과가 정한다.
      const identity = (d: { category: string; name: string }) => `${d.category}\u0000${d.name}`;
      const seen = new Set(before.map(identity));
      const added = after.filter((d) => !seen.has(identity(d)));
      expect(added.length, '코드가 늘었는데 의존성이 그대로다').toBeGreaterThan(0);

      const diff = await diffOf(page, second.id);
      expect(diff.comparedTo, '직전 분석이 상대다').toBe(first.id);
      expect(
        diff.features.map((f) => f.key),
        '바뀌지 않은 기능은 목록에 서지 않는다',
      ).toEqual([key]);
      expect(diff.features.map((f) => f.key)).not.toContain(untouched);

      const changed = diff.features[0];
      expect(changed.scenarioLines, '시나리오 문장은 그대로다').toHaveLength(0);
      expect(changed.dependencyLines.map((l) => ({ mark: l.mark, name: l.name }))).toEqual(
        added.map((d) => ({ mark: '+', name: d.name })),
      );
      expect(changed.dependencyLines[0].category).toBe(added[0].category);
      expect(diff.changedLineCount).toBe(changed.dependencyLines.length);

      const untouchedView = await dependenciesOf(page, second.id, untouched);
      expect(untouchedView.status, '이 기능은 이번에도 묻지 않았다').toBeNull();

      await page.goto(`/#/analyses/${second.id}/diff`);
      await expect(page.getByTestId('diff-list')).toBeVisible();
      await expect(page.getByTestId('diff-feature')).toHaveCount(1);
      await expect(page.getByTestId('diff-feature-name')).toHaveText(changed.name);
      await expect(page.getByTestId('diff-line')).toHaveCount(changed.dependencyLines.length);
      await expect(page.getByTestId('diff-line-count')).toHaveText(
        String(changed.dependencyLines.length),
      );
      await expect(page.getByTestId('diff-dependency-name').first()).toHaveText(added[0].name);

      await page.getByTestId('diff-filter').selectOption('scenario');
      await expect(page.getByTestId('diff-feature')).toHaveCount(0);
      await expect(page.getByTestId('diff-line-count')).toHaveText('0');
      await page.getByTestId('diff-filter').selectOption('dep');
      await expect(page.getByTestId('diff-feature')).toHaveCount(1);

      await page.reload();
      await expect(page.getByTestId('diff-feature')).toHaveCount(1);
      await expect(page.getByTestId('diff-line-count')).toHaveText(
        String(changed.dependencyLines.length),
      );
    } finally {
      await scaleWorkers(0);
      setWorkerEnv(REVISION, null);
    }
  });
});
