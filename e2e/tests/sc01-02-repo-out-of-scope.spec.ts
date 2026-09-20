// 검증 시나리오: 01-analysis-pipeline.md#시나리오 2
//
// 앞부분의 홈 목록 → pre-flight → `Queued` 는 거부를 대비시키는 셋업이지 이 파일의
// 선언 대상이 아니다 — 그 절반은 시나리오 1 전용 spec 이 신설될 때 이어받는다
// (doc-tracker 「e2e 매핑」의 미매핑 잔여 표).
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_GITHUB_APP=stub), whose GitHub App
// installation can reach exactly three deterministic repositories
// (stub-account/{payments-api,checkout-web,notif-worker}).
//
// App installation / key state is per *user*, so this spec signs in as its own stub
// user: sharing an identity would let one spec install the App out from under another.
import { expect, test } from '@playwright/test';

test('AC1.1: 홈 → 저장소 연결 → 분석 트리거(queued)', async ({ page }) => {
  await page.goto('/api/auth/login?as=ac11');

  await page.getByTestId('connect-app').click();
  await expect(page.getByTestId('connection')).toBeVisible();

  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill('sk-ant-api03-aaaaaaaaaaaaaaaaaaaa');
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toBeVisible();

  // 같은 버튼을 다시 누른다 — 입력이 빈 두 번째 클릭은 등록이 아니라 pre-flight 다.
  await page.getByTestId('register-key').click();

  const cards = page.getByTestId('repo-card');
  await expect(cards).toHaveCount(3);
  await expect(page.locator('[data-testid="repo-card"] .badge')).toHaveCount(0);
  await expect(cards.filter({ hasText: 'stub-account/payments-api' })).toBeVisible();

  await page.getByTestId('repo-url').fill('github.com/someone-else/private-repo');
  await page.getByTestId('check-access').click();
  const noAccess = page.getByTestId('no-access');
  await expect(noAccess).toBeVisible();
  await expect(noAccess).toContainText('App 설치 범위 밖입니다');
  await expect(page.getByTestId('manage-install')).toBeVisible();
  await expect(page.getByTestId('start-analysis')).toHaveCount(0);

  // Nothing was queued by the refused attempt.
  await expect(page.locator('[data-testid="repo-card"] .badge')).toHaveCount(0);

  await cards.filter({ hasText: 'stub-account/payments-api' }).click();
  await expect(page.getByTestId('repo-url')).toHaveValue('github.com/stub-account/payments-api');
  await expect(page.getByTestId('no-access')).toHaveCount(0);
  await page.getByTestId('check-access').click();
  const estimate = page.getByTestId('estimate');
  await expect(estimate).toBeVisible();
  await expect(estimate).toContainText('stub-account/payments-api');
  await expect(estimate).toContainText('Est. LLM Cost');
  await expect(page.getByTestId('access')).toContainText('has access');

  await page.getByTestId('start-analysis').click();
  await expect(page.locator('[data-testid="repo-card"] .badge')).toHaveCount(1);
  const analyzed = page.getByTestId('repo-card').filter({ hasText: 'stub-account/payments-api' });
  await expect(analyzed).toContainText('Queued');
});
