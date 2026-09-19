// 검증 시나리오: 02-feature-representation.md#시나리오 7
//
// AC2.5 (의존성 데이터의 외부 재사용) 전용 spec.
//
// 시나리오 7 의 기대 결과는 「구조화된 형식(JSON/그래프 형식 등)으로 **외부에서 재사용
// 가능한 형태**로 내려받을 수 있다」이다. 그래서 이 파일이 보는 것은 두 가지다 — 내려받는
// 것이 정말 **파일로 나가는가**(`Content-Disposition: attachment`), 그리고 그 파일이 추적한
// 행들을 **그대로 담고 있는가**. 화면이 JSON 을 보여 주는 것은 export 가 아니다.
//
// 「외부에서 재사용 가능」의 판정은 내려받은 문서를 다시 파싱해 API 가 돌려준 행과 맞춰
// 보는 것으로 한다 — 형식이 스스로를 설명한다(분류 축이 문서 안에 있고, feature 별로
// 묶여 있다).
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. Like every spec it signs in as its own stub user (`?as=ac26`).
import { expect, test } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { runToAcceptance, signInWithCredentials } from '../support/acceptance';
import { CATEGORIES, traced } from '../support/dependencies';

type Exported = {
  analysisId: string;
  repository: string;
  branch: string;
  categories: string[];
  features: { key: string; name: string | null; dependencies: { category: string; name: string; evidence: string | null }[] }[];
};

test.describe('AC2.5: 의존성 데이터는 제품 밖으로 나갈 수 있다', () => {
  test.describe.configure({ mode: 'serial', timeout: 420_000 });

  test('첨부 파일로 내려받히고, 그 안에 추적한 행이 그대로 들어 있다', async ({ page }) => {
    try {
      await scaleWorkers(0);
      await signInWithCredentials(page, 'ac26');
      await scaleWorkers(1);

      const { id, confirmed } = await runToAcceptance(page, 'payments-api');
      const key = confirmed[0];
      const items = await traced(page, id, key);
      expect(items.length).toBeGreaterThan(0);

      const res = await page.request.get(`/api/analyses/${id}/dependencies/export`);
      expect(res.status()).toBe(200);
      // 화면에 그리는 것이 아니라 **파일로 나간다**.
      expect(res.headers()['content-disposition'] ?? '').toContain('attachment;');

      const exported = (await res.json()) as Exported;
      expect(exported.analysisId).toBe(id);
      expect(exported.repository).toBe('stub-account/payments-api');
      // 형식이 스스로를 설명한다 — 분류 축이 문서 안에 있다.
      expect(exported.categories).toEqual([...CATEGORIES]);

      const feature = exported.features.find((f) => f.key === key);
      expect(feature, '추적한 기능이 export 에 없다').toBeTruthy();
      expect((feature as Exported['features'][number]).dependencies).toEqual(items);

      // 아직 추적하지 않은 기능은 들어 있지 않다 — export 는 있는 것만 내보낸다.
      for (const entry of exported.features) {
        expect(entry.dependencies.length).toBeGreaterThan(0);
      }

      // 남의 분석은 존재하지 않는다 (AC4.7 과 같은 판정).
      await page.goto('/api/auth/login?as=ac26-stranger');
      const stranger = await page.request.get(`/api/analyses/${id}/dependencies/export`);
      expect(stranger.status()).toBe(404);
    } finally {
      await scaleWorkers(0);
    }
  });
});
