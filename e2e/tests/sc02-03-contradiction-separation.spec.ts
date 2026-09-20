// 검증 시나리오: 02-feature-representation.md#시나리오 3
//
// 모순의 판정 규칙 자체(같은 given+when 에 다른 then)는 서버 코드이고
// `backend/src/acceptance.rs` 의 단위 테스트가 지킨다. 여기서 보는 것은 그 결과가
// 문서와 화면에서 **본 시나리오에 섞이지 않고 따로 선다**는 성질이다.
//
// **원문의 기대 결과 중 여기서 단정하지 않는 것**: 「사용자는 어느 쪽을 정설로 채택할지
// 결정할 수 있다」 — 그 채택 조작이 아직 구현돼 있지 않아 e2e 가 관측할 대상 자체가
// 없다. 등재는 `docs/doc-tracker.md` 「e2e 매핑」 이 행의 "자동화 밖 잔여" 칸이다.
//
// Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.
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

test.describe('시나리오 3: 로직과 테스트가 모순되면 그 차이가 따로 선다', () => {
  test.describe.configure({ mode: 'serial', timeout: 420_000 });

  test('모순은 별도 섹션으로 표시되고 본 시나리오 목록에 섞이지 않는다', async ({ page }) => {
    try {
      await scaleWorkers(0);
      await signInWithCredentials(page, 'sc0203');
      await scaleWorkers(1);

      // stub 더블은 "the stub disagrees exactly once" 로 고정돼 있어 개수까지 단정할 수 있다.
      const { id, features } = await runToAcceptance(page, 'payments-api');
      const doc = features[0];

      expect(doc.contradictions.length, '이 저장소는 정확히 한 번 다툰다').toBe(1);

      for (const clash of doc.contradictions) {
        expect(clash.codeSays).not.toBe(clash.testSays);
        expect(clash.codeEvidence).toContain('payments-api/');
        expect(isTestPath(clash.testEvidence), `테스트 측 근거가 아니다: ${clash.testEvidence}`)
          .toBe(true);
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
      const list = page.getByTestId('scenario-list');
      await expect(list).not.toContainText(doc.contradictions[0].testSays);
    } finally {
      await scaleWorkers(0);
    }
  });
});
