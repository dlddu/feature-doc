// 검증 시나리오: 04-platform.md#시나리오 12
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_GITHUB_AUTH=stub), so the GitHub
// OAuth boundary is answered by an in-process test double — no network. The *default*
// stub user belongs to sc04-11, so this spec signs in as its own identity
// (`?as=sc0412`).
import { expect, test } from '@playwright/test';

test('AC4.8·AC4.3: 로그아웃하면 직전 세션 쿠키가 즉시 거부되고, 토큰은 오류에 노출되지 않는다', async ({
  page,
}) => {
  await page.goto('/api/auth/login?as=sc0412');
  const me = await page.request.get('/api/me');
  expect(me.ok()).toBeTruthy();

  // 직전 세션 쿠키를 포착해 둔다 — 로그아웃 뒤 이 값을 그대로 재사용하는 것이
  // 시나리오의 요구다(컨텍스트가 지워주는 것과는 다른 관측).
  const cookies = await page.context().cookies();
  const session = cookies.find((c) => c.name === 'fd_session');
  expect(session, '로그인이 세션 쿠키를 발급했다').toBeTruthy();
  const oldCookie = `${session!.name}=${session!.value}`;

  const loggedOut = await page.request.post('/api/auth/logout');
  expect(loggedOut.status()).toBe(204);

  const jarNow = await page.context().cookies();
  expect(
    jarNow.find((c) => c.name === 'fd_session'),
    '로그아웃이 컨텍스트의 세션 쿠키를 지웠다',
  ).toBeUndefined();

  const replay = await page.request.get('/api/me', {
    headers: { Cookie: oldCookie },
  });
  expect(replay.status(), '이전 세션 쿠키의 요청은 거부된다').toBe(401);

  const body = await replay.text();
  expect(body, `오류 응답이 토큰 값을 노출하지 않아야 한다: ${body}`).not.toContain(
    session!.value,
  );
});
