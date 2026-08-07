// 검증 AC: AC4.8
//
// AC4.8 (GitHub OAuth 기반 사용자 본인 인증과 세션) 전용 spec.
//
// Runs against the stub-mode deployment (FEATUREDOC_MODE=stub), so the GitHub OAuth
// boundary is answered by an in-process test double — no network. This spec owns the
// *default* stub user, because it is the only one that drives the real UI entry point
// ("Sign in with GitHub"); every other spec signs in through `?as=<handle>` with its
// own identity so that per-user state (App installation, LLM keys) never collides
// across the parallel workers that share one deployment.
import { expect, test } from '@playwright/test';

test('AC4.8: 미인증 진입 → GitHub 로그인 → 동일 사용자 세션 → 로그아웃 시 즉시 무효화', async ({
  page,
}) => {
  // 미인증: 보호된 화면 대신 단 하나의 주요 행동(로그인)만 제시된다.
  await page.goto('/');
  const signin = page.getByTestId('signin');
  await expect(signin).toBeVisible();

  // 미인증 요청은 보호 API에서 거부된다.
  const anonymous = await page.request.get('/api/me');
  expect(anonymous.status()).toBe(401);

  // 로그인(스텁 OAuth 왕복) 후 인증 상태로 돌아온다.
  await signin.click();
  await expect(page.getByTestId('connect-app')).toBeVisible();

  // 본인 조회로 GitHub 계정 기준의 사용자가 식별된다.
  const first = await (await page.request.get('/api/me')).json();
  expect(first.login).toBeTruthy();

  // 재로그인해도 같은 사용자로 해석된다(계정 중복 생성 없음).
  await page.goto('/api/auth/login');
  await expect(page.getByTestId('connect-app')).toBeVisible();
  const second = await (await page.request.get('/api/me')).json();
  expect(second.id).toBe(first.id);
  expect(second.login).toBe(first.login);

  // 로그아웃하면 그 세션은 즉시 무효화되고, 화면은 로그인으로 유도된다.
  const loggedOut = await page.request.post('/api/auth/logout');
  expect(loggedOut.status()).toBe(204);
  expect((await page.request.get('/api/me')).status()).toBe(401);

  await page.goto('/');
  await expect(page.getByTestId('signin')).toBeVisible();
});
