// 검증 AC: AC2.3
//
// AC2.3 (최종 사용자 관점의 인수 테스트 문서 생성) 전용 spec.
//
// AC2.3 이 요구하는 것은 셋이다: **feature 1개당 1개 문서**, **개발자 용어가 아닌 최종
// 사용자의 언어**, **모든 시나리오에 근거 코드/테스트 위치가 링크**.
//
// "비개발자가 읽고 설명할 수 있는가"는 사람이 하는 판정이라 자동화 밖이다(그 사실은
// doc-tracker 의 「자동화 밖 잔여」에 남는다). 여기서 기계로 지킬 수 있는 것은 그 판정의
// **필요조건** — 문서에 HTTP 메서드·라우트 경로·함수 호출 같은 개발자 어휘가 남아 있지
// 않다는 것이다. 근거 칸은 이 검사에서 뺀다: 거기는 일부러 코드 위치를 적는 자리다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. Like every spec it signs in as its own stub user (`?as=ac23`).
import { expect, test } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { runToAcceptance, signInWithCredentials } from '../support/acceptance';

/** 최종 사용자가 쓰지 않는 어휘. 하나라도 남아 있으면 이 문서는 P2 에게 공유할 수 없다
 *  (`JRN-review-feature` 의 페인포인트: "개발자 용어가 남아 있으면 V3 가 무너진다"). */
const DEVELOPER_VOCABULARY = /\b(GET|POST|PUT|PATCH|DELETE|HTTP|API|SQL|null|undefined)\b|\/api\/|\(\)|=>|;/;

test.describe('AC2.3: 확정된 기능마다 최종 사용자의 언어로 된 문서 하나', () => {
  test.describe.configure({ mode: 'serial', timeout: 420_000 });

  test('확정한 기능 수만큼 문서가 생기고, 각 문서는 사용자의 말로 쓰이며 근거를 모두 단다', async ({
    page,
  }) => {
    try {
      await scaleWorkers(0);
      await signInWithCredentials(page, 'ac23');
      await scaleWorkers(1);

      // 기능 **둘**을 확정한다 — "feature 1개당 1개 문서"는 하나만 확정해서는 관측할 수
      // 없는 성질이다.
      const { id, features, confirmed } = await runToAcceptance(page, 'payments-api', 2);

      expect(features).toHaveLength(confirmed.length);
      expect(new Set(features.map((f) => f.key)).size, '문서가 기능을 공유하면 안 된다').toBe(
        features.length,
      );
      expect(features.map((f) => f.key).sort()).toEqual([...confirmed].sort());

      for (const doc of features) {
        expect(doc.name.trim().length).toBeGreaterThan(0);
        expect(doc.scenarios.length, `${doc.name} 에 시나리오가 없다`).toBeGreaterThan(0);
        for (const scenario of doc.scenarios) {
          // 최종 사용자의 언어: 시나리오 본문에 개발자 어휘가 없다.
          for (const part of [scenario.given, scenario.when, scenario.then]) {
            expect(
              DEVELOPER_VOCABULARY.test(part),
              `개발자 용어가 남아 있다: ${part}`,
            ).toBe(false);
          }
          // 모든 시나리오에 근거 위치가 붙는다.
          expect(scenario.evidence.trim().length, '근거 없는 시나리오').toBeGreaterThan(0);
        }
      }

      // ── S08: 기능을 고르면 그 기능의 문서만 그린다 ──────────────────────
      await page.goto(`/#/analyses/${id}/acceptance`);
      const select = page.getByTestId('feature-select');
      await expect(select.locator('option')).toHaveCount(features.length);

      for (const doc of features) {
        await select.selectOption(doc.key);
        await expect(page.getByTestId('feature-title')).toHaveText(doc.name);
        await expect(page.getByTestId('scenario')).toHaveCount(doc.scenarios.length);
        await expect(page.getByTestId('scenario-list')).toContainText(doc.scenarios[0].then);
        // 다른 기능의 문장이 섞여 들지 않는다.
        const other = features.find((f) => f.key !== doc.key);
        if (other && other.scenarios[0].then !== doc.scenarios[0].then) {
          await expect(page.getByTestId('scenario-list')).not.toContainText(
            other.scenarios[0].then,
          );
        }
      }

      // 표현이 아니라 발견 자체가 틀렸다면, 이 화면은 후보 결정으로 돌려보낸다.
      await page.getByTestId('not-a-feature').click();
      await expect(page.getByTestId('not-a-feature-confirm')).toBeVisible();
      await page.getByTestId('back-to-candidates').click();
      await expect(page.getByTestId('candidate-list')).toBeVisible();
    } finally {
      await scaleWorkers(0);
    }
  });
});
