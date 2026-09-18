// 검증 시나리오: 04-platform.md#시나리오 12
//
// 「로그아웃 후 세션 무효화」 전용 spec (AC4.8·AC4.3).
//
// docs/test/04-platform.md 시나리오 12를 그대로 따라간다:
//   로그인 상태에서 로그아웃한 뒤, 직전 세션 쿠키를 그대로 사용해 보호 API를
//   재호출하면 거부된다. 세션 토큰은 로그/오류 메시지에 평문 노출되지 않는다.
//
// 이 단정은 `sc04-11-unauthenticated-block-and-signin.spec.ts`와 한 파일에 있었다
// (선언은 시나리오 11 하나) — 규칙 2 상 분리 대상으로 등재됐다가 이 파일로 옮겨왔다.
// 옮기면서 원문이 요구하는 관측으로 바로잡았다: 기존 단정은 로그아웃 응답이 쿠키를
// 지운 **같은 컨텍스트**의 401이라 「이전 세션 쿠키」를 재사용한 것이 아니었다.
// 서버측 무효화는 제품의 실제 동작이다(backend/src/session.rs `delete()` = `DELETE
// FROM sessions`; auth.rs logout 핸들러가 호출) — 로그아웃 전에 쿠키를 포착해 두고,
// 로그아웃 뒤 그 값을 명시적 Cookie 헤더로 재사용하는 401로 관측한다.
//
// 오류 메시지 절반만 검증한다: 401 응답 본문이 토큰 값을 평문으로 담지 않는다.
// 운영 로그 절반은 클러스터 로그 수집이 필요해 선례(sc04-05)와 같이 등재 SSOT의
// 「자동화 밖 잔여」에 남는다.
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_GITHUB_AUTH=stub), so the GitHub OAuth
// boundary is answered by an in-process test double — no network. The *default*
// stub user belongs to sc04-11 (it is the only one driving the real UI entry
// point); this spec signs in as its own identity (`?as=sc0412`) so that per-user
// state never collides across the parallel workers that share one deployment.
import { expect, test } from '@playwright/test';

test('AC4.8·AC4.3: 로그아웃하면 직전 세션 쿠키가 즉시 거부되고, 토큰은 오류에 노출되지 않는다', async ({
  page,
}) => {
  // 로그인 상태(세션 쿠키 보유)를 만든다.
  await page.goto('/api/auth/login?as=sc0412');
  const me = await page.request.get('/api/me');
  expect(me.ok()).toBeTruthy();

  // 직전 세션 쿠키를 포착해 둔다 — 로그아웃 뒤 이 값을 그대로 재사용하는 것이
  // 시나리오의 요구다(컨텍스트가 지워주는 것과는 다른 관측).
  const cookies = await page.context().cookies();
  const session = cookies.find((c) => c.name === 'fd_session');
  expect(session, '로그인이 세션 쿠키를 발급했다').toBeTruthy();
  const oldCookie = `${session!.name}=${session!.value}`;

  // 로그아웃한다.
  const loggedOut = await page.request.post('/api/auth/logout');
  expect(loggedOut.status()).toBe(204);

  // 로그아웃 응답이 컨텍스트의 쿠키를 지웠는지 먼저 확인한다 — 지워진 컨텍스트에서
  // 401이 나와도 그건 시나리오의 관측이 아니다. 재사용은 명시적 헤더로 한다.
  const jarNow = await page.context().cookies();
  expect(
    jarNow.find((c) => c.name === 'fd_session'),
    '로그아웃이 컨텍스트의 세션 쿠키를 지웠다',
  ).toBeUndefined();

  // 직전 세션 쿠키를 그대로 사용해 보호 API를 재호출한다 — 서버측 무효화라서
  // 거부된다.
  const replay = await page.request.get('/api/me', {
    headers: { Cookie: oldCookie },
  });
  expect(replay.status(), '이전 세션 쿠키의 요청은 거부된다').toBe(401);

  // 오류 메시지가 토큰을 평문으로 돌려주지 않는다.
  const body = await replay.text();
  expect(body, `오류 응답이 토큰 값을 노출하지 않아야 한다: ${body}`).not.toContain(
    session!.value,
  );
});
