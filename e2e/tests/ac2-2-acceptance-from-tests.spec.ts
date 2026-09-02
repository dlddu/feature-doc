// 검증 AC: AC2.2
//
// AC2.2 (테스트 코드로부터 인수 기준 보강) 전용 spec.
//
// AC2.2 의 검증 방법 두 가지를 그대로 따라간다: **테스트가 존재하는 feature 는 인수 기준의
// 시나리오 수가 로직만 본 결과보다 같거나 많고**, **모순 케이스는 별도 섹션("코드와 테스트가
// 다르게 말하는 부분")으로 노출된다**.
//
// "로직만 본 결과"는 문서 자신이 말해 준다 — 각 시나리오는 자기가 어느 패스에서 나왔는지
// (`source`) 를 달고 있고, 테스트 패스가 더한 것은 전부 **테스트 파일**을 근거로 든다.
// 그러므로 이 spec 은 고정 개수를 세지 않고, 두 출처의 관계와 근거의 종류를 본다.
//
// 모순의 판정 규칙 자체(같은 given+when 에 다른 then)는 서버 코드이고
// `backend/src/acceptance.rs` 의 단위 테스트가 지킨다. 여기서 보는 것은 그 결과가
// **본 시나리오에 섞이지 않는다**는 성질 — test/02 시나리오 3 이 요구하는 분리다.
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

test.describe('AC2.2: 테스트 코드가 인수 기준을 보강하고, 모순은 따로 선다', () => {
  test.describe.configure({ mode: 'serial', timeout: 420_000 });

  test('테스트 출처 시나리오는 테스트 파일을 근거로 들고, 모순은 본 시나리오에 섞이지 않는다', async ({
    page,
  }) => {
    try {
      await scaleWorkers(0);
      await signInWithCredentials(page, 'ac22');
      await scaleWorkers(1);

      const { id, features } = await runToAcceptance(page, 'payments-api');
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

      // ── 모순: 별도 섹션에 서고, 양쪽을 모두 이름 붙인다 ─────────────────
      expect(doc.contradictions.length, '이 저장소에는 모순이 있어야 한다').toBeGreaterThan(0);
      for (const clash of doc.contradictions) {
        expect(clash.codeSays).not.toBe(clash.testSays);
        expect(clash.codeEvidence).toContain('payments-api/');
        expect(isTestPath(clash.testEvidence), `테스트 측 근거가 아니다: ${clash.testEvidence}`)
          .toBe(true);
        // 분리: 테스트가 말하는 결말이 본 시나리오 목록에 들어가 있으면 안 된다.
        expect(
          doc.scenarios.some((s) => s.then === clash.testSays),
          '모순의 테스트 측 문장이 본 시나리오로 읽히고 있다',
        ).toBe(false);
        // 그리고 그 상황은 본 시나리오에서 찾을 수 있어야 한다 — 어느 문장이 다투는지
        // 독자가 짚을 수 없으면 "따로 두었다"가 아니라 "숨겼다"가 된다.
        expect(
          doc.scenarios.some((s) => s.given === clash.given && s.when === clash.when),
        ).toBe(true);
      }

      // ── S08 이 그 섹션을 그린다 ─────────────────────────────────────────
      await page.goto(`/#/analyses/${id}/acceptance`);
      const box = page.getByTestId('contradictions');
      await expect(box).toBeVisible();
      await expect(box).toContainText('코드와 테스트가 다르게 말하는 부분이 있어요.');
      await expect(page.getByTestId('contradiction')).toHaveCount(doc.contradictions.length);
      const first = page.getByTestId('contradiction').first();
      await expect(first).toContainText(doc.contradictions[0].codeSays);
      await expect(first).toContainText(doc.contradictions[0].testSays);
      await expect(first.getByTestId('contradiction-test')).toContainText(
        doc.contradictions[0].testEvidence,
      );
      // 본 시나리오 목록에는 그 문장이 없다 — 화면에서도 분리돼 있다.
      const list = page.getByTestId('scenario-list');
      await expect(list).not.toContainText(doc.contradictions[0].testSays);
    } finally {
      await scaleWorkers(0);
    }
  });
});
