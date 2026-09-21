// 검증 시나리오: 03-doc-management.md#시나리오 2
//
// 회피를 **관측**한다. 「다음 제안에서 피했다」는 모델의 선의가 아니라 규칙이어야 하므로,
// 같은 요청을 두 번 보내 두 번째 제안이 거부된 문면과 다른지를 본다(스텁도 같은 규칙을
// 따른다 — `backend/src/doc_edit.rs` 의 `stub_edit`). 사람에게 알리는 쪽도 같이 본다:
// 요청 화면이 거부 건수와 사유를 먼저 보여 준다.
//
// Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.
import { expect, test } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { acceptanceOf, runToAcceptance, signInWithCredentials } from '../support/acceptance';

const REQUEST = '만료 안내를 더 분명하게';
const REASON = '오류 코드를 덧붙인 표현은 읽는 사람에게 필요 없어요';

test.describe('AC3.1: 거부한 제안은 기록되고 다음 제안이 그 방향을 피한다', () => {
  test.describe.configure({ mode: 'serial', timeout: 600_000 });

  test('사유와 함께 거부하면 문서는 그대로이고, 다시 부탁한 제안은 거부한 문면과 다르다', async ({
    page,
  }) => {
    try {
      await signInWithCredentials(page, 'sc0302');
      await scaleWorkers(1);

      const run = await runToAcceptance(page, 'payments-api', 1);
      const key = run.confirmed[0];
      const before = (await acceptanceOf(page, run.id)) ?? [];
      const scenariosBefore = before.find((f) => f.key === key)?.scenarios.length ?? 0;

      await page.goto(`/#/analyses/${run.id}/features/${encodeURIComponent(key)}/dependencies`);
      await page.getByTestId('request-edit').click();
      await expect(page.getByTestId('rejected-note')).toHaveCount(0);

      await page.getByTestId('edit-request').fill(REQUEST);
      await page.getByTestId('send-request').click();
      const first = await page.getByTestId('diff-added').innerText();

      await page.getByTestId('reject-diff').click();
      await expect(page.getByTestId('reject-reason')).toBeVisible();
      await page.getByTestId('reject-reason').fill(REASON);
      await page.getByTestId('reject-diff').click();
      await expect(page.getByTestId('scenario-list')).toBeVisible();

      const afterReject = (await acceptanceOf(page, run.id)) ?? [];
      expect(
        afterReject.find((f) => f.key === key)?.scenarios.length,
        '거부했는데 문서가 바뀌었다',
      ).toBe(scenariosBefore);

      await page.goto(`/#/analyses/${run.id}/features/${encodeURIComponent(key)}/dependencies`);
      await page.getByTestId('request-edit').click();

      await expect(page.getByTestId('rejected-count')).toHaveText('1');
      await expect(page.getByTestId('rejected-reason')).toHaveText(REASON);

      await page.getByTestId('edit-request').fill(REQUEST);
      await page.getByTestId('send-request').click();
      const second = await page.getByTestId('diff-added').innerText();
      expect(second, '거부한 제안이 그대로 다시 왔다').not.toBe(first);
    } finally {
      await scaleWorkers(0);
    }
  });
});
