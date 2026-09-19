// 검증 시나리오: 04-platform.md#시나리오 11
//
// 「미인증 접근 차단과 GitHub 로그인」 전용 spec (AC4.8).
//
// 시나리오 12(로그아웃 후 세션 무효화)의 단정은
// `sc04-12-logout-session-invalidation.spec.ts`가 지킨다 — 한 파일에 있던 두
// 시나리오를 분리한 것은 `rct_20260916-0002`가 닫았다.
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_GITHUB_AUTH=stub), so the GitHub OAuth
// boundary is answered by an in-process test double — no network. This spec owns the
// *default* stub user, because it is the only one that drives the real UI entry point
// (the Sign In screen's primary button); every other spec signs in through `?as=<handle>` with its
// own identity so that per-user state (App installation, LLM keys) never collides
// across the parallel workers that share one deployment.
import { expect, test } from '@playwright/test';

test('AC4.8: 미인증 진입 → GitHub 로그인 → 동일 사용자 세션', async ({
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
});
