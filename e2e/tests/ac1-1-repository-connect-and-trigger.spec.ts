// 검증 AC: AC1.1
//
// AC1.1 (저장소 연결 및 분석 트리거) 전용 spec.
//
// Runs against the stub-mode deployment (FEATUREDOC_MODE=stub), whose GitHub App
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
  await expect(page.getByTestId('metrics')).toBeVisible();
  await expect(page.getByTestId('metric-repos')).toHaveText('3');
  await expect(page.getByTestId('metric-analyses')).toHaveText('0');
  const cards = page.getByTestId('repo-card');
  await expect(cards).toHaveCount(3);
  await expect(cards.filter({ hasText: 'stub-account/payments-api' })).toBeVisible();

  // ── Connect Repository: a target outside the App's granted access is refused ──
  await page.getByTestId('new-repository').click();
  await page.getByTestId('repo-url').fill('github.com/someone-else/private-repo');
  await page.getByTestId('check-access').click();
  const noAccess = page.getByTestId('no-access');
  await expect(noAccess).toBeVisible();
  await expect(noAccess).toContainText('someone-else/private-repo');
  // The recovery path (App 설치 범위 관리) is offered, and the trigger is unreachable.
  await expect(page.getByTestId('manage-install')).toBeVisible();
  await expect(page.getByTestId('start-analysis')).toHaveCount(0);

  // Nothing was queued by the refused attempt.
  await page.getByTestId('back').click();
  await expect(page.getByTestId('metric-analyses')).toHaveText('0');

  // ── Connect Repository: an in-scope target shows the pre-flight estimate before triggering ──
  await page.getByTestId('new-repository').click();
  await page.getByTestId('repo-url').fill('stub-account/payments-api');
  await page.getByTestId('check-access').click();
  const estimate = page.getByTestId('estimate');
  await expect(estimate).toBeVisible();
  await expect(estimate).toContainText('stub-account/payments-api');
  await expect(estimate).toContainText('Est. LLM Cost');
  await expect(page.getByTestId('access')).toContainText('has access');

  // ── Trigger: the job lands on the home list as queued ──
  await page.getByTestId('start-analysis').click();
  await expect(page.getByTestId('metric-analyses')).toHaveText('1');
  const analyzed = page.getByTestId('repo-card').filter({ hasText: 'stub-account/payments-api' });
  await expect(analyzed).toContainText('Queued');
});
