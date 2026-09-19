// 검증 시나리오: 02-feature-representation.md#시나리오 3
//
// 로직과 테스트가 모순되는 경우 — 코드 로직과 테스트가 같은 상황에 다른 결말을 말할 때,
// 그 차이가 **별도 섹션으로 표시되는가**를 본다.
//
// 이 단정들은 `sc02-02-acceptance-from-tests.spec.ts` 안에 있던 것을 옮겨 온 것이다.
// 그 파일은 `02#시나리오 2`(테스트로부터의 보강)를 선언하는데 모순 분리는 시나리오 3 의
// 기대 결과이므로, 시나리오 하나가 자기 전용 파일을 갖는다는 1:1 규약대로 갈라 세웠다.
// 단정 내용은 옮기면서 바꾸지 않았고, 사전 조건 관측 한 줄만 더했다(아래).
//
// 모순의 판정 규칙 자체(같은 given+when 에 다른 then)는 서버 코드이고
// `backend/src/acceptance.rs` 의 단위 테스트가 지킨다. 여기서 보는 것은 그 결과가 문서와
// 화면에서 **본 시나리오에 섞이지 않고 따로 선다**는 성질이다.
//
// **원문의 기대 결과 중 여기서 단정하지 않는 것**: 「사용자는 어느 쪽을 정설로 채택할지
// 결정할 수 있다」. 그 채택 조작은 아직 구현돼 있지 않아(AC3.x · 슬라이스 6, 담당
// `tbm_feature-doc-docs-impl`) e2e 가 관측할 대상 자체가 없다. 이 루프는 화면·엔드포인트를
// 신설하지 않으므로 그 몫은 남겨 두고, 구현이 착지하면 이 spec 이 확장된다.
// 등재는 `docs/doc-tracker.md` 「e2e 매핑」 매핑 표의 이 행 "자동화 밖 잔여" 칸에 있다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. Like every spec it signs in as its own stub user (`?as=sc0203`).
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

      const { id, features } = await runToAcceptance(page, 'payments-api');
      const doc = features[0];

      // ── 사전 조건: 이 저장소는 코드와 테스트가 실제로 다투는 곳이다 ─────────
      // 원문의 사전 조건("코드 로직은 100자, 테스트는 50자를 가정")에 해당하는 상황을
      // stub 더블이 결정적으로 재현한다 — `backend/src/acceptance.rs` 의 단위 테스트가
      // "the stub disagrees exactly once" 로 고정해 둔 성질이라 개수까지 단정할 수 있다.
      // 전용 파일이 되었으므로 이 시나리오의 사전 조건은 여기서 직접 지킨다.
      expect(doc.contradictions.length, '이 저장소는 정확히 한 번 다툰다').toBe(1);

      // ── 모순: 별도 섹션에 서고, 양쪽을 모두 이름 붙인다 ─────────────────
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

      // ── 인수 시나리오 화면이 그 섹션을 그린다 ─────────────────────────────────────────
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
