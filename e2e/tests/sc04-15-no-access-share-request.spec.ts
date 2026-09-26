// 검증 시나리오: 04-platform.md#시나리오 15
//
// 시나리오가 요구하는 대비군이 이 spec 의 구조다 — 실재하는 남의 분석 id 와 한 번도 존재한
// 적 없는 id 를 **같은 절차**에 통과시키고, 두 축이 돌려준 것을 값으로 비교한다. 칸을 하나씩
// 골라 비교하면 나중에 생긴 칸이 검사 없이 새므로, 응답은 (status, body) 쌍으로 화면은
// textContent 전체로 비교한다.
import { expect, test } from '@playwright/test';
import type { APIResponse, Page } from '@playwright/test';

import { installApp } from '../support/github-app';

const KEY = 'sk-proj-dddddddddddddddddddddd';
/** 어떤 분석도 가질 수 없는 id — 시나리오의 「실재하지 않는 임의 id」. */
const ABSENT = '00000000-0000-4000-8000-000000000000';
/** B 의 화면·응답 어디에도 나타나서는 안 되는 것들(소유자 신원과 대상의 정체). */
const OWNER_HANDLE = 'sc0415a';
const OWNER_REPO = 'payments-api';

async function signIn(page: Page, handle: string): Promise<void> {
  await page.goto(`/api/auth/login?as=${handle}`);
  await installApp(page.request);
  const created = await page.request.post('/api/llm-keys', {
    data: { provider: 'openai', key: KEY },
  });
  expect(created.status(), `${handle} 의 키 등록`).toBe(201);
}

async function seen(res: APIResponse): Promise<[number, string]> {
  return [res.status(), await res.text()];
}

/** 막힌 화면을 열고 → 공유 요청을 보내고 → 접수 화면의 문면 전체를 돌려준다. */
async function walk(page: Page, target: string): Promise<string> {
  await page.goto(`/#/analyses/${target}`);
  const notice = page.getByTestId('no-access');
  await expect(notice, '막힌 자리의 안내').toBeVisible();
  await expect(
    page.getByTestId('progress-error'),
    '사유 없는 오류 뱃지로 떨어지지 않는다',
  ).toHaveCount(0);

  const cta = page.getByTestId('btn-request-access');
  await expect(cta, '막힌 그 자리에서 공유 요청 경로가 보인다').toBeVisible();
  await cta.click();

  const done = page.getByTestId('access-requested');
  await expect(done, '요청 후 접수 상태로 바뀐다').toBeVisible();
  await expect(page.getByTestId('no-access'), '막힌 안내는 물러난다').toHaveCount(0);
  await expect(
    page.getByTestId('btn-open-another'),
    '이어지는 경로는 다른 링크 열기뿐이다',
  ).toBeVisible();
  return ((await done.textContent()) ?? '').trim();
}

test('AC4.10: 막힌 링크에서 공유 요청을 보낼 수 있고, 그 요청이 대상의 실재 여부를 말하지 않는다', async ({
  browser,
}) => {
  const ctxA = await browser.newContext();
  const ctxB = await browser.newContext();
  try {
    // ── A: 저장소 하나를 분석해 둔 사람 ──────────────────────────────────
    const pageA = await ctxA.newPage();
    await signIn(pageA, OWNER_HANDLE);
    const created = await pageA.request.post('/api/analyses', {
      data: { repoUrl: `stub-account/${OWNER_REPO}`, branch: null },
    });
    expect(created.status(), 'A 가 분석을 만든다').toBe(201);
    const hidden = (await created.json()).id as string;

    // ── B: 그 저장소에 접근 권한이 없는 사람 ─────────────────────────────
    const pageB = await ctxB.newPage();
    await signIn(pageB, 'sc0415b');

    const mine = (await (await pageB.request.get('/api/analyses')).json()) as { id: string }[];
    expect(mine.map((a) => a.id), '목록에 그 저장소가 없다').not.toContain(hidden);
    for (const target of [hidden, ABSENT]) {
      const detail = await pageB.request.get(`/api/analyses/${target}`);
      expect(detail.status(), `상세에도 ${target} 가 나오지 않는다`).toBe(404);
    }

    // ── 두 축을 같은 절차에 통과시킨다 ───────────────────────────────────
    const realScreen = await walk(pageB, hidden);
    const absentScreen = await walk(pageB, ABSENT);
    expect(absentScreen, '대비군과 화면이 구분된다').toBe(realScreen);

    const realApi = await seen(
      await pageB.request.post(`/api/analyses/${hidden}/access-request`),
    );
    const absentApi = await seen(
      await pageB.request.post(`/api/analyses/${ABSENT}/access-request`),
    );
    expect(realApi[0], '요청은 접수된다').toBe(202);
    expect(absentApi, '대비군과 응답이 구분된다').toEqual(realApi);

    // ── 소유자의 신원도, 대상의 정체도 B 에게 가지 않는다 ────────────────
    for (const leak of [OWNER_HANDLE, OWNER_REPO, 'stub-account']) {
      expect(realScreen, `화면이 ${leak} 를 흘린다`).not.toContain(leak);
      expect(realApi[1], `응답이 ${leak} 를 흘린다`).not.toContain(leak);
    }

    // ── 요청은 접근을 열지 않는다 — 허락 전까지 그대로 안 보인다 ─────────
    const after = await pageB.request.get(`/api/analyses/${hidden}`);
    expect(after.status(), '요청했다고 열리지 않는다').toBe(404);
    const stillMine = (await (await pageA.request.get('/api/analyses')).json()) as {
      id: string;
    }[];
    expect(stillMine.map((a) => a.id), 'A 에게는 그대로 있다').toContain(hidden);
  } finally {
    await ctxA.close();
    await ctxB.close();
  }
});
