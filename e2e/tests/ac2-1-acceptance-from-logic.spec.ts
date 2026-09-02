// 검증 AC: AC2.1
//
// AC2.1 (로직 코드로부터 인수 기준 도출) 전용 spec.
//
// AC2.1 의 검증 방법 두 가지를 그대로 따라간다: 확정된 feature 하나에 대해 인수 기준이
// **"주어진 ~ / ~ 했을 때 / ~ 해야 한다"** 형태로 생성되고, **각 인수 기준에 근거가 된
// 코드 위치가 첨부**된다.
//
// 기대값을 상수로 박지 않는다. "이 기준이 이 저장소에서 나왔는가"는 문서 자신이 아니라
// **스캔이 본 저장소**를 기준으로 판정해야 의미가 있으므로, 근거 경로가 분석 대상 안에
// 있는지로 본다. 픽스처를 바꿔도 이 테스트는 여전히 옳고, 5단계가 근거 없는 문장을
// 지어내는 순간에만 깨진다.
//
// 이 spec 이 브라우저에서 관측하는 또 하나: **확정이 5단계를 실제로 연다**. 확정 전에는
// 문서가 아예 없고(404 — "아직 생성 전"과 "생성했는데 비었음"을 구분한다), 확정하면
// 분석이 다시 큐에 들어가 5단계가 돌아간다.
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

      // ── 확정 전: 쓸 문서가 없다 ─────────────────────────────────────────
      const created = await page.request.post('/api/analyses', {
        data: { repoUrl: 'stub-account/payments-api', branch: null },
      });
      expect(created.status()).toBe(201);
      const early = (await created.json()).id as string;
      expect(await acceptanceOf(page, early)).toBeNull();

      // ── 워커를 켜고 확정까지 걸어간다 ───────────────────────────────────
      await scaleWorkers(1);
      const { id, features, confirmed } = await runToAcceptance(page, 'payments-api');

      // 확정한 feature 의 문서다 — 후보 목록이 아니라 확정 목록이 입력이다.
      expect(features).toHaveLength(1);
      expect(features[0].key).toBe(confirmed[0]);
      const named = (await candidatesOf(page, id)).find((c) => c.key === confirmed[0]);
      expect(features[0].name, '문서의 이름은 사용자가 확정한 그 이름이다').toBe(named?.name);

      // ── 형태: 주어진 / 이럴 때 / 이렇게 됩니다 ─────────────────────────
      expect(features[0].scenarios.length, '인수 기준이 없으면 검수할 것이 없다').toBeGreaterThan(0);
      for (const scenario of features[0].scenarios) {
        for (const part of [scenario.given, scenario.when, scenario.then]) {
          expect(part.trim().length, `빈 칸이 있는 인수 기준: ${JSON.stringify(scenario)}`)
            .toBeGreaterThan(0);
        }
        // ── 근거: 각 인수 기준에 코드 위치가 첨부되고, 그 위치는 이 분석이
        //    실제로 본 저장소 안이다.
        expect(scenario.evidence, `분석 대상 밖의 근거: ${scenario.evidence}`).toContain(
          'payments-api/',
        );
      }

      // ── S08 이 그 문서를 그린다 ─────────────────────────────────────────
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
