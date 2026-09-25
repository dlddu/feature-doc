// 검증 시나리오: 04-platform.md#시나리오 6
//
// 목업은 어느 목록도 접힌 상태로 그리지 않는다. 접힘은 목업보다 위인 AC4.4 의 「긴 분석
// 결과는 단계적으로 노출(요약 → 상세)된다」에서 오고, 그 어긋남은 편차 원장 3행으로
// 등재돼 있다. 그래서 이 spec 의 관측 기준은 목업 카피가 아니라 **폭에 따라 접힘이
// 갈리는가**다 — compact 에서 요약만 서고, 첫 확장 브레이크포인트에서 같은 화면이
// 펼쳐지는 것.
//
// 이 파일에 쓰인 수는 전부 문서에서 왔다: 390px 과 5분은 시나리오 6 의 실행 단계·기대
// 결과, 600px 은 `frontend/src/index.css` 가 레이아웃을 확장하는 첫 브레이크포인트다.
//
// 네 단계를 걷는 것 자체가 시나리오의 단정이라 화면을 질러가지 않는다. 다만 각 단계
// **안쪽**의 정합성은 그 단계를 소유한 spec 의 몫이다 — 보조 수정 3탭은 `sc03-01`,
// 의존성 분류·필터는 `sc02-05` 가 세므로 여기서 다시 세지 않는다.
//
// Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { acceptanceOf, runToAcceptance, signInWithCredentials } from '../support/acceptance';

const PHONE = { width: 390, height: 844 };
const DESKTOP = { width: 1280, height: 800 };
/** 시나리오 6 의 기대 결과가 든 예산. 한 세션 전체에 걸리는 상한이다. */
const SESSION_BUDGET_MS = 300_000;

/** 한 손 조작의 관측 가능한 형태 — 이 폭에서 가로로 밀리는 것이 없다. */
async function fitsOneHandedWidth(page: Page): Promise<boolean> {
  return await page.evaluate(() => {
    const root = document.scrollingElement ?? document.body;
    return root.scrollWidth <= window.innerWidth;
  });
}

test.describe('AC4.4: 모바일 폭에서 요약 → 상세로 훑는 5분 세션', () => {
  test.describe.configure({ mode: 'serial', timeout: 600_000 });

  test('390px 에서 긴 목록은 요약만 서고, 네 단계를 5분 예산 안에 걷는다', async ({ page }) => {
    try {
      await page.setViewportSize(PHONE);
      await signInWithCredentials(page, 'sc0406');
      await scaleWorkers(1);

      const run = await runToAcceptance(page, 'payments-api', 1);
      const key = run.confirmed[0];
      const docs = (await acceptanceOf(page, run.id)) ?? [];
      const target = docs.find((f) => f.key === key);
      expect(target, '검수할 feature 의 인수 문서가 없다').toBeDefined();
      expect(target?.scenarios.length ?? 0, '접을 시나리오가 하나는 있어야 한다').toBeGreaterThan(
        0,
      );

      // 세션은 여기서 시작한다 — 위는 시나리오의 사전 조건("분석이 완료되어 …")이다.
      const startedAt = Date.now();

      // (a) 분석 진행 확인 — 요약은 `N of M`, 상세는 단계 목록.
      await page.goto(`/#/analyses/${run.id}`);
      const pipeline = page.getByTestId('pipeline-disclosure');
      await expect(pipeline).toBeVisible();
      await expect(page.getByTestId('pipeline-count')).toBeVisible();
      await expect(pipeline).not.toHaveAttribute('open', '');
      await expect(page.getByTestId('stage').first()).toBeHidden();
      expect(await fitsOneHandedWidth(page), '진행 화면이 390px 에서 가로로 밀린다').toBe(true);

      await pipeline.locator('summary').click();
      await expect(pipeline).toHaveAttribute('open', '');
      await expect(page.getByTestId('stage').first()).toBeVisible();

      // (b) feature 1개 검토 — 같은 규칙이 인수 시나리오 목록에 걸린다.
      await page.goto(`/#/analyses/${run.id}/acceptance`);
      const scenarios = page.getByTestId('scenarios-disclosure');
      await expect(scenarios).toBeVisible();
      await expect(page.getByTestId('scenario-count')).toBeVisible();
      await expect(scenarios).not.toHaveAttribute('open', '');
      await expect(page.getByTestId('scenario-list')).toBeHidden();
      expect(await fitsOneHandedWidth(page), '검수 화면이 390px 에서 가로로 밀린다').toBe(true);

      await scenarios.locator('summary').click();
      await expect(scenarios).toHaveAttribute('open', '');
      await expect(page.getByTestId('scenario-list')).toBeVisible();

      // (c) LLM 보조 수정 1회 — 3탭 규약 자체는 sc03-01 이 센다.
      await page.getByTestId('request-edit').click();
      await page.getByTestId('edit-request').fill('만료된 카드 에러 케이스 1개 더 추가');
      await page.getByTestId('send-request').click();
      await expect(page.getByTestId('diff-added')).toHaveCount(1);
      await page.getByTestId('approve-diff').click();
      await expect(page.getByTestId('scenario-list')).toBeVisible();

      // (d) 의존성 1건 조회.
      await page.goto(`/#/analyses/${run.id}/features/${encodeURIComponent(key)}/dependencies`);
      await expect(page.getByTestId('dependency-list')).toBeVisible();
      await expect(page.getByTestId('dependency').first()).toBeVisible();
      expect(await fitsOneHandedWidth(page), '의존성 화면이 390px 에서 가로로 밀린다').toBe(true);

      const elapsed = Date.now() - startedAt;
      expect(elapsed, `네 단계가 5분 예산을 넘었다 (${elapsed}ms)`).toBeLessThan(SESSION_BUDGET_MS);

      // 데스크톱은 같은 화면을 확장 적용한다 — 접힘은 compact 전용이고,
      // 600px 을 넘기면 같은 경로가 펼쳐진 채로 선다.
      await page.setViewportSize(DESKTOP);
      await page.goto(`/#/analyses/${run.id}/acceptance`);
      await expect(page.getByTestId('scenarios-disclosure')).toHaveAttribute('open', '');
      await expect(page.getByTestId('scenario-list')).toBeVisible();
      await page.goto(`/#/analyses/${run.id}`);
      await expect(page.getByTestId('pipeline-disclosure')).toHaveAttribute('open', '');
      await expect(page.getByTestId('stage').first()).toBeVisible();
    } finally {
      await scaleWorkers(0);
    }
  });
});
