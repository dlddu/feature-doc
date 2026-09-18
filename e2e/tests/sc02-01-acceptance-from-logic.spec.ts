// 검증 시나리오: 02-feature-representation.md#시나리오 1
//
// 기대값을 상수로 박지 않는다. "이 기준이 이 저장소에서 나왔는가"는 문서 자신이 아니라
// **스캔이 본 저장소**를 기준으로 판정해야 의미가 있으므로, 근거 경로가 분석 대상 안에
// 있는지로 본다. 픽스처를 바꿔도 이 테스트는 여전히 옳고, 5단계가 근거 없는 문장을
// 지어내는 순간에만 깨진다.
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. Like every spec it signs in as its own stub user (`?as=ac21`).
import { expect, test } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import {
  acceptanceOf,
  candidatesOf,
  runToAcceptance,
  signInWithCredentials,
} from '../support/acceptance';

test.describe('AC2.1: 확정된 feature 의 로직에서 인수 기준을 뽑는다', () => {
  test.describe.configure({ mode: 'serial', timeout: 420_000 });

  test('확정 전에는 문서가 없고, 확정하면 모든 인수 기준이 이 저장소의 코드 위치를 근거로 든다', async ({
    page,
  }) => {
    try {
      await scaleWorkers(0);
      await signInWithCredentials(page, 'ac21');

      const created = await page.request.post('/api/analyses', {
        data: { repoUrl: 'stub-account/payments-api', branch: null },
      });
      expect(created.status()).toBe(201);
      const early = (await created.json()).id as string;
      expect(await acceptanceOf(page, early)).toBeNull();

      await scaleWorkers(1);
      const { id, features, confirmed } = await runToAcceptance(page, 'payments-api');

      expect(features).toHaveLength(1);
      expect(features[0].key).toBe(confirmed[0]);
      const named = (await candidatesOf(page, id)).find((c) => c.key === confirmed[0]);
      expect(features[0].name, '문서의 이름은 사용자가 확정한 그 이름이다').toBe(named?.name);

      expect(features[0].scenarios.length, '인수 기준이 없으면 검수할 것이 없다').toBeGreaterThan(0);
      for (const scenario of features[0].scenarios) {
        for (const part of [scenario.given, scenario.when, scenario.then]) {
          expect(part.trim().length, `빈 칸이 있는 인수 기준: ${JSON.stringify(scenario)}`)
            .toBeGreaterThan(0);
        }
        expect(scenario.evidence, `분석 대상 밖의 근거: ${scenario.evidence}`).toContain(
          'payments-api/',
        );
      }

      await page.goto(`/#/analyses/${id}/acceptance`);
      await expect(page.getByTestId('feature-title')).toHaveText(features[0].name);
      await expect(page.getByTestId('scenario')).toHaveCount(features[0].scenarios.length);
      await expect(page.getByTestId('scenario-count')).toHaveText(
        String(features[0].scenarios.length),
      );
      const first = page.getByTestId('scenario').first();
      await expect(first).toContainText('주어진 상황');
      await expect(first).toContainText('이럴 때');
      await expect(first).toContainText('이렇게 됩니다');
      await expect(first).toContainText(features[0].scenarios[0].given);
      await expect(first.getByTestId('scenario-evidence')).toContainText(
        features[0].scenarios[0].evidence,
      );

      // 화면이 아니라 서버가 기억한다 — 새로고침이 곧 그 증거다.
      await page.reload();
      await expect(page.getByTestId('scenario')).toHaveCount(features[0].scenarios.length);
    } finally {
      await scaleWorkers(0);
    }
  });
});
