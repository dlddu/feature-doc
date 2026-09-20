// 검증 시나리오: 02-feature-representation.md#시나리오 2
//
// Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.
import { expect, test } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { runToAcceptance, signInWithCredentials } from '../support/acceptance';

/** 백엔드 `acceptance::is_test_path` 와 같은 관례를 복제한 것 — 한쪽만 고치면 갈라진다. */
function isTestPath(path: string): boolean {
  const file = path.split('/').pop() ?? path;
  const dirs = path.split('/').slice(0, -1);
  if (dirs.some((d) => ['test', 'tests', 'spec', 'specs', '__tests__'].includes(d))) return true;
  const stem = file.split('.')[0];
  return (
    file.includes('.test.') ||
    file.includes('.spec.') ||
    stem.endsWith('_test') ||
    stem.endsWith('-test') ||
    stem.startsWith('test_')
  );
}

test.describe('AC2.2: 테스트 코드가 인수 기준을 보강한다', () => {
  test.describe.configure({ mode: 'serial', timeout: 420_000 });

  test('테스트 출처 시나리오는 테스트 파일을 근거로 들고, 로직 패스의 결과는 지워지지 않는다', async ({
    page,
  }) => {
    try {
      await scaleWorkers(0);
      await signInWithCredentials(page, 'ac22');
      await scaleWorkers(1);

      const { features } = await runToAcceptance(page, 'payments-api');
      const doc = features[0];

      const fromLogic = doc.scenarios.filter((s) => s.source === 'logic');
      const fromTests = doc.scenarios.filter((s) => s.source === 'test');
      expect(fromLogic.length, '로직 패스가 없으면 보강할 대상이 없다').toBeGreaterThan(0);
      expect(fromTests.length, '테스트가 있는 저장소인데 보강이 없다').toBeGreaterThan(0);
      expect(doc.scenarios.length).toBeGreaterThanOrEqual(fromLogic.length);

      for (const scenario of fromTests) {
        expect(
          isTestPath(scenario.evidence),
          `테스트 패스가 테스트가 아닌 파일을 근거로 들었다: ${scenario.evidence}`,
        ).toBe(true);
        expect(scenario.evidence).toContain('payments-api/');
      }
    } finally {
      await scaleWorkers(0);
    }
  });
});
