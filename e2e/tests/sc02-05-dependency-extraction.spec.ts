// 검증 시나리오: 02-feature-representation.md#시나리오 5
//
// AC2.4 (feature 단위 종단 의존성 추출) 전용 spec.
//
// 시나리오 5 의 실행 단계가 이 파일의 형태를 정한다 — 「해당 feature 선택 → "의존성 분석"
// 트리거」. 즉 추출은 파이프라인이 알아서 하는 일이 아니라 **사람이 feature 하나에 대해
// 누르는 행동**이고, 그래서 이 spec 은 먼저 「아무도 누르지 않았으면 아무것도 없다」를
// 단정한 뒤에 누른다.
//
// 기대 결과의 「각 카테고리별로 분리되어 표시된다 · 각 항목에 코드 근거가 있다」는
// **상수로 박지 않는다**. 문서의 예시(결제 PG · 주문 엔티티)는 시나리오가 그리는 그림이지
// 이 저장소의 사실이 아니므로, 판정은 스캔된 트리를 기준으로 한다 — 분류는 7종 안이고,
// 근거는 이 분석이 실제로 본 경로이거나 **명시적으로 비어 있다**. 뒤쪽은 여정의 예외 표가
// 요구하는 성질이다(「근거 없음으로 명시. 임의로 채우지 않음」).
//
// Isolation: this spec *leases* the analysis worker (see `e2e/support/cluster.ts`).
// It scales the Deployment to 1 inside its own block and returns it to 0 in
// `finally`; `playwright.config.ts` pins `workers: 1`, so no sibling spec file is in
// flight while it runs. Like every spec it signs in as its own stub user (`?as=ac24`).
import { expect, test } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { runToAcceptance, signInWithCredentials } from '../support/acceptance';
import { CATEGORIES, dependenciesOf, flatten, traced } from '../support/dependencies';

test.describe('AC2.4: 확정된 feature 의 종단 의존성을 분류별로 뽑는다', () => {
  test.describe.configure({ mode: 'serial', timeout: 420_000 });

  test('누르기 전에는 아무것도 없고, 누르면 분류별로 갈린 항목이 이 저장소의 경로를 근거로 든다', async ({
    page,
  }) => {
    try {
      await scaleWorkers(0);
      await signInWithCredentials(page, 'ac24');
      await scaleWorkers(1);

      const { id, confirmed } = await runToAcceptance(page, 'payments-api');
      const key = confirmed[0];

      // ── 누르기 전 ───────────────────────────────────────────────────────
      const before = await dependenciesOf(page, id, key);
      expect(before.status, '아무도 요청하지 않았는데 상태가 있다').toBeNull();
      expect(before.total).toBe(0);
      // 일곱 칸은 언제나 그려진다 — 빈 분류도 정보다.
      expect(before.categories.map((c) => c.category)).toEqual([...CATEGORIES]);

      // ── 누른 뒤 ─────────────────────────────────────────────────────────
      const items = await traced(page, id, key);
      expect(items.length, '의존성이 하나도 나오지 않았다').toBeGreaterThan(0);

      const scanned: string[] = [];
      for (const item of items) {
        expect(CATEGORIES as readonly string[]).toContain(item.category);
        if (item.evidence !== null) {
          // 「각 항목에 코드 근거가 있다」 — 근거는 이 분석이 본 저장소의 경로다.
          expect(item.evidence).toContain('payments-api/');
          scanned.push(item.evidence);
        }
      }
      expect(scanned.length, '근거를 든 항목이 하나도 없다').toBeGreaterThan(0);

      // 「인프라 · 데이터 모델 · 미들웨어 · 프레임워크 · 진입점 …」 — 다층 의존성을 가진
      // 저장소는 한 분류에 뭉치지 않는다. 몇 종인지는 트리가 정하지 이 파일이 정하지 않는다.
      const kinds = new Set(items.map((i) => i.category));
      expect(kinds.size, `분류가 갈리지 않았다: ${[...kinds].join(' · ')}`).toBeGreaterThanOrEqual(
        5,
      );

      // ── 종단 의존성 화면이 그 행들을 그린다 ─────────────────────────────
      await page.goto(`/#/analyses/${id}/features/${encodeURIComponent(key)}/dependencies`);
      await expect(page.getByTestId('dependency-list')).toBeVisible();
      await expect(page.getByTestId('dependency')).toHaveCount(items.length);
      await expect(page.getByTestId('dependency-count')).toHaveText(String(items.length));
      await expect(page.getByTestId('dependency-name').first()).toHaveText(items[0].name);

      // 분류 필터가 그 축으로 실제로 자른다.
      const first = items[0].category;
      const sameKind = items.filter((i) => i.category === first);
      await page.getByTestId('dependency-filter').selectOption(first);
      await expect(page.getByTestId('dependency')).toHaveCount(sameKind.length);
      await expect(page.getByTestId('dependency-count')).toHaveText(String(sameKind.length));

      // 새로고침 후에도 같다 — 의존성은 서버 상태다.
      await page.reload();
      await expect(page.getByTestId('dependency')).toHaveCount(items.length);
      const again = flatten(await dependenciesOf(page, id, key));
      expect(again).toEqual(items);
    } finally {
      await scaleWorkers(0);
    }
  });
});
