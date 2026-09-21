// 검증 시나리오: 03-doc-management.md#시나리오 4
//
// 「강제로 만들어내지 않는다」를 **관측**한다 — 트리에 없는 것을 부탁하면 초안이 비어 오고
// 화면은 「근거 없음」을 그대로 보인다. 사람은 그 상태로 추가하거나(출처는 「직접」)
// 취소할 수 있고, 어느 쪽이든 자동 문서는 그대로다.
//
// 워커를 임대하지 않는다 — 근거 찾기는 이 분석의 트리와 모델을 API 프로세스가 직접
// 부르므로 파이프라인이 한 단계도 돌지 않은 분석에서도 성립한다(문서 부재 404 의 뜻,
// 「아직 생성 전」, 이 그대로임을 함께 본다). Like every spec it signs in as its own
// stub user (`?as=…`).
import { expect, test } from '@playwright/test';
import { acceptanceOf, signInWithCredentials } from '../support/acceptance';

/** 스텁 트리의 어느 경로에도 들어 있지 않은 문장 — 실제 모델은 코드 검색이 빈 경우다. */
const UNKNOWN = '관리자 일괄 삭제 추가';

async function additionsOf(page: import('@playwright/test').Page, id: string) {
  const res = await page.request.get(`/api/analyses/${id}/features/additions`);
  expect(res.status(), `additions for ${id}`).toBe(200);
  return (await res.json()) as {
    finalCount: number;
    additions: {
      id: string;
      status: string;
      source: string | null;
      evidenceFound: boolean;
      scenarios: unknown[];
      request: string;
    }[];
  };
}

test.describe('AC3.2: 코드에서 근거를 찾지 못하면 「근거 없음」을 명시하고 지어내지 않는다', () => {
  test('근거 없는 요청은 초안 0건으로 표시되고, 그대로 추가하거나 취소할 수 있다', async ({
    page,
  }) => {
    await signInWithCredentials(page, 'sc0304');

    const created = await page.request.post('/api/analyses', {
      data: { repoUrl: 'stub-account/payments-api', branch: null },
    });
    expect(created.status(), 'enqueue payments-api').toBe(201);
    const id = (await created.json()).id as string;
    expect(await acceptanceOf(page, id), '아직 자동 문서는 없다').toBeNull();

    await page.goto(`/#/analyses/${id}/features/add`);
    await expect(page.getByTestId('final-count')).toHaveText('0');

    await page.getByTestId('new-feature').fill(UNKNOWN);
    await page.getByTestId('find-evidence').click();

    // 근거 없음이 명시되고, 초안은 하나도 그려지지 않는다.
    await expect(page.getByTestId('no-evidence')).toBeVisible();
    await expect(page.getByTestId('no-evidence')).toContainText('코드에서 근거를 찾지 못했어요');
    await expect(page.getByTestId('draft-result')).toHaveCount(0);
    await expect(page.getByTestId('draft-scenario')).toHaveCount(0);

    // 서버도 같은 말을 한다 — 시도는 남되 초안은 비었다.
    const drafted = await additionsOf(page, id);
    expect(drafted.additions.length).toBe(1);
    expect(drafted.additions[0].status).toBe('drafted');
    expect(drafted.additions[0].evidenceFound).toBe(false);
    expect(drafted.additions[0].scenarios).toEqual([]);
    expect(drafted.finalCount, '결정 전에는 목록에 세지 않는다').toBe(0);

    // 「근거 없음」으로 추가 — 출처는 사람이 직접.
    await page.getByTestId('add-anyway').click();
    await expect(page.getByTestId('final-count')).toHaveText('1');
    await expect(page.getByTestId('no-evidence')).toHaveCount(0);
    await expect(page.getByTestId('new-feature')).toHaveValue('');

    const confirmed = await additionsOf(page, id);
    expect(confirmed.additions[0].status).toBe('confirmed');
    expect(confirmed.additions[0].source, 'AC3.4 가 요구하는 출처 — 직접').toBe('user_direct');
    expect(confirmed.additions[0].request).toBe(UNKNOWN);
    expect(confirmed.finalCount).toBe(1);

    // 취소 경로 — 시도는 남지만 목록에 들지 않는다.
    await page.getByTestId('new-feature').fill(UNKNOWN + ' 두 번째');
    await page.getByTestId('find-evidence').click();
    await expect(page.getByTestId('no-evidence')).toBeVisible();
    await page.getByTestId('cancel-add').click();
    await expect(page.getByTestId('no-evidence')).toHaveCount(0);
    await expect(page.getByTestId('final-count')).toHaveText('1');

    const cancelled = await additionsOf(page, id);
    expect(cancelled.additions.length).toBe(2);
    expect(cancelled.additions.map((a) => a.status).sort()).toEqual(['cancelled', 'confirmed']);
    expect(cancelled.finalCount).toBe(1);

    // 자동 문서의 뜻은 그대로다 — 5단계가 쓰지 않았으면 여전히 「아직 생성 전」이다.
    expect(await acceptanceOf(page, id)).toBeNull();

    // 남의 분석에는 닿지 않는다.
    await page.goto('/api/auth/login?as=sc0304-other');
    const foreign = await page.request.get(`/api/analyses/${id}/features/additions`);
    expect(foreign.status(), '남의 분석은 존재조차 확인해 주지 않는다').toBe(404);
  });
});
