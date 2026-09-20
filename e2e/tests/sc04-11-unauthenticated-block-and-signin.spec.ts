// 검증 시나리오: 04-platform.md#시나리오 11
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_GITHUB_AUTH=stub), so the GitHub
// OAuth boundary is answered by an in-process test double — no network. This spec owns
// the *default* stub user: it is the only one that drives the real UI entry point (the
// Sign In screen's primary button).
import { expect, test } from '@playwright/test';

test('AC4.8: 미인증 진입 → GitHub 로그인 → 동일 사용자 세션', async ({
  page,
}) => {
  await page.goto('/');
  const signin = page.getByTestId('signin');
  await expect(signin).toBeVisible();

  const anonymous = await page.request.get('/api/me');
  expect(anonymous.status()).toBe(401);

  await signin.click();
  await expect(page.getByTestId('connect-app')).toBeVisible();

  const first = await (await page.request.get('/api/me')).json();
  expect(first.login).toBeTruthy();

  await page.goto('/api/auth/login');
  await expect(page.getByTestId('connect-app')).toBeVisible();
  const second = await (await page.request.get('/api/me')).json();
  expect(second.id).toBe(first.id);
  expect(second.login).toBe(first.login);
});
