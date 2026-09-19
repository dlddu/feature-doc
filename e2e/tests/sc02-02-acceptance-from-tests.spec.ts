// 검증 시나리오: 02-feature-representation.md#시나리오 2
//
// AC2.2 (테스트 코드로부터 인수 기준 보강) 전용 spec.
//
// 시나리오 2 의 기대 결과를 그대로 따라간다: **보강 후의 시나리오 수가 보강 전보다 같거나
// 많고**, **에러 케이스/경계 조건이 추가된 시나리오에는 근거 테스트 위치가 첨부된다**.
//
// "보강 전"은 문서 자신이 말해 준다 — 각 시나리오는 자기가 어느 패스에서 나왔는지
// (`source`) 를 달고 있고, 테스트 패스가 더한 것은 전부 **테스트 파일**을 근거로 든다.
// 그러므로 이 spec 은 고정 개수를 세지 않고, 두 출처의 관계와 근거의 종류를 본다.
//
// 모순 케이스가 **별도 섹션으로 서는가**(시나리오 3 의 기대 결과)는 이 파일의 몫이 아니다 —
// `sc02-03-contradiction-separation.spec.ts` 가 전용으로 지킨다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. Like every spec it signs in as its own stub user (`?as=ac22`).
import { expect, test } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { runToAcceptance, signInWithCredentials } from '../support/acceptance';

/** 이 저장소에서 "테스트 코드"로 세는 경로 — 백엔드 `acceptance::is_test_path` 의 규약과
 *  같은 관례다. 문서가 스스로 테스트라고 주장하는 값이 아니라 **경로**로 판정한다. */
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

      // ── 보강: 두 패스가 모두 기여했다 ───────────────────────────────────
      const fromLogic = doc.scenarios.filter((s) => s.source === 'logic');
      const fromTests = doc.scenarios.filter((s) => s.source === 'test');
      expect(fromLogic.length, '로직 패스가 없으면 보강할 대상이 없다').toBeGreaterThan(0);
      expect(fromTests.length, '테스트가 있는 저장소인데 보강이 없다').toBeGreaterThan(0);
      // "보강 후의 시나리오 수가 보강 전보다 같거나 많다" — 로직 패스의 결과는 하나도
      // 지워지지 않는다.
      expect(doc.scenarios.length).toBeGreaterThanOrEqual(fromLogic.length);

      // ── 보강의 근거는 테스트 코드다 ─────────────────────────────────────
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
