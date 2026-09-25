// 검증 시나리오: 03-doc-management.md#시나리오 1
//
// 3탭을 **세는 것이 이 spec 의 일**이다 — 시나리오가 「3탭 이내」를 기대 결과로 들고
// 있으므로, 흐름을 API 로 질러가면 그 단정이 사라진다. 그래서 진입부터 승인까지는
// 화면의 버튼만 누르고(타이핑은 탭이 아니다), 결과 확인만 저장된 문서를 읽는다.
//
// Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.
import { expect, test } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { acceptanceOf, runToAcceptance, signInWithCredentials } from '../support/acceptance';

const PHONE = { width: 390, height: 844 };

test.describe('AC3.1·AC3.4: 한 줄로 부탁한 수정이 3탭 안에 문서와 이력에 남는다', () => {
  test.describe.configure({ mode: 'serial', timeout: 600_000 });

  test('수정 요청 → 제안 확인 → 승인 세 번으로 시나리오가 하나 늘고, 출처가 남는다', async ({
    page,
  }) => {
    try {
      await page.setViewportSize(PHONE);
      await signInWithCredentials(page, 'sc0301');
      await scaleWorkers(1);

      const run = await runToAcceptance(page, 'payments-api', 1);
      const key = run.confirmed[0];
      const before = (await acceptanceOf(page, run.id)) ?? [];
      const target = before.find((f) => f.key === key);
      expect(target, '검수할 feature 의 인수 문서가 없다').toBeDefined();
      const scenariosBefore = target?.scenarios.length ?? 0;
      expect(scenariosBefore, '고칠 시나리오가 하나는 있어야 한다').toBeGreaterThan(0);

      await page.goto(`/#/analyses/${run.id}/features/${encodeURIComponent(key)}/dependencies`);

      // ── 탭 1 ─────────────────────────────────────────────────────────
      await page.getByTestId('request-edit').click();
      await expect(page.getByTestId('edit-target')).toContainText(
        target?.scenarios[0].when ?? '',
      );
      await expect(page.getByTestId('request-empty')).toBeVisible();
      await expect(page.getByTestId('send-request')).toBeDisabled();

      await page.getByTestId('edit-request').fill('만료된 카드 에러 케이스 1개 더 추가');

      // ── 탭 2 ─────────────────────────────────────────────────────────
      await page.getByTestId('send-request').click();

      await expect(page.getByTestId('diff-added')).toHaveCount(1);
      await expect(page.getByTestId('edit-source')).toHaveText('사용자 · 도움받아 작성');
      await expect(page.getByTestId('edit-request-text')).toHaveText(
        '만료된 카드 에러 케이스 1개 더 추가',
      );
      const staged = (await acceptanceOf(page, run.id)) ?? [];
      expect(
        staged.find((f) => f.key === key)?.scenarios.length,
        '승인하지 않았는데 문서가 이미 바뀌었다',
      ).toBe(scenariosBefore);

      const proposed = await page.getByTestId('diff-added').innerText();
      const proposalUrl = page.url();
      const proposalId = /\/proposals\/([^/?#]+)/.exec(proposalUrl)?.[1];
      expect(proposalId, `제안 주소를 지나지 않았다: ${proposalUrl}`).toBeTruthy();

      // ── 탭 3 ─────────────────────────────────────────────────────────
      await page.getByTestId('approve-diff').click();

      // 아래 펼침은 결과 확인이라 탭 계수 밖이다 — 세는 탭 셋은 「탭 3」 에서 끝났다.
      await expect(page.getByTestId('scenario-count')).toBeVisible();
      await page.getByTestId('scenarios-disclosure').locator('summary').click();
      await expect(page.getByTestId('scenario-list')).toBeVisible();

      const after = (await acceptanceOf(page, run.id)) ?? [];
      const edited = after.find((f) => f.key === key);
      expect(edited?.scenarios.length, '승인했는데 시나리오가 늘지 않았다').toBe(
        scenariosBefore + 1,
      );
      const added = proposed.replace(/^\+\s*/, '');
      expect(
        edited?.scenarios.some((s) => s.then === added),
        '제안한 문장이 문서에 없다',
      ).toBeTruthy();
      expect(
        edited?.scenarios.some((s) => s.then === target?.scenarios[0].then),
        '원래 시나리오가 사라졌다',
      ).toBeTruthy();

      const proposals = await page.request.get(
        `/api/analyses/${run.id}/features/edit-proposals/${proposalId}`,
      );
      expect(proposals.status(), '승인된 제안을 다시 읽을 수 있어야 한다').toBe(200);
      const record = await proposals.json();
      expect(record.status).toBe('approved');
      expect(record.source, 'AC3.4 가 요구하는 출처').toBe('user_llm');
      expect(record.request, '무엇을 부탁했는지도 이력이다').toBe(
        '만료된 카드 에러 케이스 1개 더 추가',
      );
    } finally {
      await scaleWorkers(0);
    }
  });
});
