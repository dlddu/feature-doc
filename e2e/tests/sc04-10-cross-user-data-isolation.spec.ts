// 검증 시나리오: 04-platform.md#시나리오 10
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_GITHUB_AUTH=stub), so each
// `?as=<handle>` is its own stub GitHub account — distinct user id, and therefore a
// distinct App installation (`github_app::stub_installation_id`). Two browser
// contexts keep the two cookie jars apart, which is what lets A hold a live session
// while B's resources exist.
//
// 시나리오의 실행 단계는 "A 인증으로 B 의 자원에 **직접** API 호출" 이므로 이 spec 은
// 화면을 걷지 않고 주소를 직접 때린다 — 목록에서 안 보이는 것과 id 를 알고도 못 읽는 것은
// 다른 관측이고, 시나리오가 요구하는 것은 후자다.
//
// 거부는 403 이 아니라 **404** 다: 남의 id 에 403 을 주면 "그 id 는 존재한다"를 확인해
// 주는 셈이라, `analysis::owned_analysis` 가 의도적으로 404 로 닫는다.
import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';

import { installApp } from '../support/github-app';

const KEY = 'sk-proj-dddddddddddddddddddddd';

/** 한 사용자를 세우고 그 사용자의 LLM 키 id 를 돌려준다. */
async function signIn(page: Page, handle: string): Promise<string> {
  await page.goto(`/api/auth/login?as=${handle}`);
  await installApp(page.request);
  const created = await page.request.post('/api/llm-keys', {
    data: { provider: 'openai', key: KEY },
  });
  expect(created.status(), `${handle} 의 키 등록`).toBe(201);
  return (await created.json()).id as string;
}

test('AC4.7: 다른 사용자의 분석·자격증명은 id 를 알고 직접 호출해도 닿지 않는다', async ({
  browser,
}) => {
  const ctxB = await browser.newContext();
  const ctxA = await browser.newContext();
  try {
    // ── 사용자 B: 분석 1건과 키 1개를 가진 상태 ───────────────────────────
    const pageB = await ctxB.newPage();
    const keyB = await signIn(pageB, 'sc0410b');
    const created = await pageB.request.post('/api/analyses', {
      data: { repoUrl: 'stub-account/payments-api', branch: null },
    });
    expect(created.status(), 'B 가 분석을 만든다').toBe(201);
    const idB = (await created.json()).id as string;

    // ── 사용자 A: 다른 계정 ───────────────────────────────────────────────
    const pageA = await ctxA.newPage();
    const keyA = await signIn(pageA, 'sc0410a');
    expect(keyA, '두 사용자의 키는 서로 다른 자원이다').not.toBe(keyB);

    // ⑴ 목록에는 남의 것이 섞이지 않는다.
    const mine = (await (await pageA.request.get('/api/analyses')).json()) as { id: string }[];
    expect(mine.map((a) => a.id), 'A 의 분석 목록').not.toContain(idB);
    const myKeys = (await (await pageA.request.get('/api/llm-keys')).json()) as { id: string }[];
    expect(myKeys.map((k) => k.id), 'A 의 키 목록은 A 의 것뿐').toEqual([keyA]);
    const conn = await (await pageA.request.get('/api/github/connection')).json();
    expect(conn.installed, 'A 는 자기 설치만 본다').toBe(true);

    // ⑵ id 를 알고 직접 읽어도 거부된다 — 읽기 경로 전수.
    const reads = [
      `/api/analyses/${idB}`,
      `/api/analyses/${idB}/candidates`,
      `/api/analyses/${idB}/discovery-strategy`,
      `/api/analyses/${idB}/documents/cross-cutting`,
      `/api/analyses/${idB}/diff`,
      `/api/analyses/${idB}/conflicts`,
      `/api/analyses/${idB}/features/additions`,
      `/api/analyses/${idB}/features/deletions`,
      `/api/analyses/${idB}/dependencies/export`,
    ];
    for (const url of reads) {
      const res = await pageA.request.get(url);
      expect(res.status(), `A 가 B 의 ${url} 를 읽을 수 없다`).toBe(404);
    }

    // ⑶ 쓰기 경로도 같다 — 거부가 조회에만 걸려 있으면 격리가 아니다.
    const approve = await pageA.request.post(
      `/api/analyses/${idB}/discovery-strategy/approve`,
    );
    expect(approve.status(), 'A 가 B 의 전략을 승인할 수 없다').toBe(404);
    const decide = await pageA.request.post(`/api/analyses/${idB}/candidates/decision`, {
      data: { key: 'payments-api/src/api/routes.rs', decision: 'confirmed' },
    });
    expect(decide.status(), 'A 가 B 의 후보를 결정할 수 없다').toBe(404);
    const revoked = await pageA.request.delete(`/api/llm-keys/${keyB}`);
    expect(revoked.status(), 'A 가 B 의 키를 폐기할 수 없다').toBe(404);

    // ⑷ 그리고 B 자신은 여전히 자기 분석을 읽는다 — 위의 404 가 "없는 자원"이 아니라
    //    "남의 자원"이라는 것을 이 한 줄이 못박는다.
    const stillThere = await pageB.request.get(`/api/analyses/${idB}`);
    expect(stillThere.status(), 'B 에게는 그대로 있다').toBe(200);
    expect((await stillThere.json()).id).toBe(idB);
  } finally {
    await ctxA.close();
    await ctxB.close();
  }
});
