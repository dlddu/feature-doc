// 검증 시나리오: 01-analysis-pipeline.md#시나리오 2
//
// 「접근 범위 밖 저장소 접근 시도」 전용 spec (AC1.1 의 거부 절반).
//
// 앞부분의 홈 목록 → pre-flight → `Queued` 는 거부를 대비시키기 위한 정상 경로이자
// 셋업이며, 선언 대상이 아니다 — 그 절반의 기대 결과(세 산출물의 단계별 제시)는
// `01-analysis-pipeline.md#시나리오 1` 의 것이고, 그 전용 spec 이 신설될 때
// 이어받는다(doc-tracker 「e2e 매핑」의 미매핑 잔여 표 참조).
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_GITHUB_APP=stub), whose GitHub App
// installation can reach exactly three deterministic repositories
// (stub-account/{payments-api,checkout-web,notif-worker}).
//
// Drives AC1.1's verification end to end in a real browser: repo URL 입력 →
// 분석 시작 → 분석 작업이 큐에 등록됨, and the negative half — a repository outside
// the App's granted access is refused with a clear message and a recovery path, and
// nothing is queued (docs/test/01-analysis-pipeline.md 시나리오 2).
//
// Spec files run in parallel workers against one shared deployment, and App
// installation / key state is per *user* — so this spec signs in as its own stub
// user (`?as=ac11`, the same handle switch smoke.sh uses). Sharing an identity would
// let one spec install the App out from under another. The credential screen it walks
// through on the way in is setup, not this file's verification target:
// ac4-8/ac4-1/ac4-2/ac4-3 own those declarations.
import { expect, test } from '@playwright/test';

test('AC1.1: 홈 → 저장소 연결 → 분석 트리거(queued)', async ({ page }) => {
  await page.goto('/api/auth/login?as=ac11');

  // ── Credentials Setup: install the App and register a key (the Home pre-condition) ──
  await page.getByTestId('connect-app').click();
  await expect(page.getByTestId('connection')).toBeVisible();

  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill('sk-ant-api03-aaaaaaaaaaaaaaaaaaaa');
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toBeVisible();

  // Continue confirms readiness, then carries the user into Home.
  const cont = page.getByTestId('continue');
  await cont.click();
  await expect(page.getByTestId('ready')).toBeVisible();
  await cont.click();

  // ── Home: the repositories the App can reach, and no analyses yet ──
  // 메트릭 그리드는 목업에 없어 슬라이스 ⑥ 이 제거했다. 같은 두 사실을 목록 자체로 단정한다 —
  // 카드 수가 App 이 닿는 저장소 수이고, 실행 이력이 없으면 상태 배지가 하나도 없다.
  const cards = page.getByTestId('repo-card');
  await expect(cards).toHaveCount(3);
  await expect(page.locator('[data-testid="repo-card"] .badge')).toHaveCount(0);
  await expect(cards.filter({ hasText: 'stub-account/payments-api' })).toBeVisible();

  // ── 저장소 연결(같은 화면): a target outside the App's granted access is refused ──
  await page.getByTestId('repo-url').fill('github.com/someone-else/private-repo');
  await page.getByTestId('check-access').click();
  const noAccess = page.getByTestId('no-access');
  await expect(noAccess).toBeVisible();
  // 시나리오가 요구하는 것은 **명확한 사유**이고, 슬라이스 ⑥ 이 그 문면을 목업 카피로 수렴시켰다
  // (대상 저장소 이름을 끼워 넣던 구현 문구 → 목업의 고정 문장). 사유 자체는 그대로 있다.
  await expect(noAccess).toContainText('App 설치 범위 밖입니다');
  // The recovery path is offered, and the trigger is unreachable.
  await expect(page.getByTestId('manage-install')).toBeVisible();
  await expect(page.getByTestId('start-analysis')).toHaveCount(0);

  // Nothing was queued by the refused attempt — 목록에 상태 배지가 여전히 없다.
  await expect(page.locator('[data-testid="repo-card"] .badge')).toHaveCount(0);

  // ── 같은 화면의 pre-flight 영역: an in-scope target shows the estimate before triggering ──
  await page.getByTestId('repo-url').fill('stub-account/payments-api');
  await page.getByTestId('check-access').click();
  const estimate = page.getByTestId('estimate');
  await expect(estimate).toBeVisible();
  await expect(estimate).toContainText('stub-account/payments-api');
  await expect(estimate).toContainText('Est. LLM Cost');
  await expect(page.getByTestId('access')).toContainText('has access');

  // ── Trigger: the job lands on the home list as queued ──
  await page.getByTestId('start-analysis').click();
  await expect(page.locator('[data-testid="repo-card"] .badge')).toHaveCount(1);
  const analyzed = page.getByTestId('repo-card').filter({ hasText: 'stub-account/payments-api' });
  await expect(analyzed).toContainText('Queued');
});
