// 검증 시나리오: 03-doc-management.md#시나리오 3
//
// Leases the analysis worker — lease rules in `e2e/support/cluster.ts`.
import { expect, test } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import {
  acceptanceOf,
  candidatesOf,
  runToAcceptance,
  signInWithCredentials,
} from '../support/acceptance';
import { CATEGORIES, dependenciesOf, flatten } from '../support/dependencies';

/** 문장이 가리키는 곳 — 스텁 트리의 두 경로에 든 낱말이다. 실제 모델은 문장 전체를 읽는다. */
const REQUEST = 'routes 와 auth 미들웨어를 거치는 회원 탈퇴 기능';

test.describe('AC3.2: 자동 추출이 놓친 feature 를 근거와 함께 초안으로 받아 확정한다', () => {
  test.describe.configure({ mode: 'serial', timeout: 600_000 });

  test('초안과 의존성 후보가 트리의 경로를 근거로 들고, 확정하면 새 feature 로 등록된다', async ({
    page,
  }) => {
    try {
      await signInWithCredentials(page, 'sc0303');
      await scaleWorkers(1);

      const run = await runToAcceptance(page, 'payments-api', 1);
      const before = (await acceptanceOf(page, run.id)) ?? [];
      expect(before.length, '자동 문서가 한 feature 로 서 있어야 한다').toBe(1);

      for (const candidate of await candidatesOf(page, run.id)) {
        if (candidate.mergedInto !== null || candidate.decision !== 'undecided') continue;
        const res = await page.request.post(`/api/analyses/${run.id}/candidates/decision`, {
          data: { key: candidate.key, decision: 'reject', reason: '이 spec 의 셋업 — 사용자 기능이 아님' },
        });
        expect(res.ok(), `reject ${candidate.key}`).toBeTruthy();
      }

      await page.goto(`/#/analyses/${run.id}/candidates`);
      await expect(page.getByTestId('undecided-count')).toHaveText('0');
      await page.getByTestId('finish-sift').click();
      await expect(page.getByTestId('new-feature')).toBeVisible();
      await expect(page.getByTestId('final-count')).toHaveText('1');

      await expect(page.getByTestId('find-evidence')).toBeDisabled();
      await page.getByTestId('new-feature').fill(REQUEST);
      await page.getByTestId('find-evidence').click();

      await expect(page.getByTestId('draft-result')).toBeVisible();
      await expect(page.getByTestId('evidence-badge')).toHaveText(/근거 있음/);
      await expect(page.getByTestId('no-evidence')).toHaveCount(0);

      // `.legend` 가 대문자로 그리므로(text-transform) 렌더 텍스트가 아니라 DOM 텍스트를 읽는다.
      const evidence = await page.getByTestId('draft-evidence').allTextContents();
      expect(evidence.length, '초안이 근거를 하나는 들어야 한다').toBeGreaterThan(0);
      for (const path of evidence) {
        expect(path, '트리 밖의 경로를 근거로 들었다').toMatch(/^payments-api\//);
        expect(path, '문장이 가리키지 않은 곳을 근거로 들었다').toMatch(/routes|auth/);
      }
      const scenarioCount = await page.getByTestId('draft-scenario').count();
      expect(scenarioCount).toBe(evidence.length);

      const dependencies = page.getByTestId('draft-dependency');
      const dependencyCount = await dependencies.count();
      expect(dependencyCount, '의존성 후보가 하나는 있어야 한다').toBeGreaterThan(0);
      for (const category of await dependencies.evaluateAll((els) =>
        els.map((el) => el.getAttribute('data-category')),
      )) {
        expect(CATEGORIES as readonly string[]).toContain(category);
      }
      for (const path of await page.getByTestId('dependency-evidence').allTextContents()) {
        expect(path).toMatch(/^payments-api\//);
      }

      expect(((await acceptanceOf(page, run.id)) ?? []).length, '확정하지 않았는데 문서가 늘었다').toBe(1);

      await page.getByTestId('confirm-draft').click();
      await expect(page.getByTestId('final-count')).toHaveText('2');
      await expect(page.getByTestId('draft-result')).toHaveCount(0);

      const after = (await acceptanceOf(page, run.id)) ?? [];
      expect(after.length, '확정했는데 feature 가 늘지 않았다').toBe(2);
      const added = after.find((f) => f.key.startsWith('added:'));
      expect(added, '직접 더한 feature 가 문서에 없다').toBeDefined();
      expect(added?.name).toBe(REQUEST);
      expect(added?.scenarios.length).toBe(scenarioCount);
      for (const scenario of added?.scenarios ?? []) {
        expect(scenario.source, 'AC3.4 가 요구하는 출처 — 도움받아 작성').toBe('user_llm');
        expect(evidence).toContain(scenario.evidence);
      }
      expect(after.some((f) => f.key === run.confirmed[0]), '자동 feature 가 사라졌다').toBeTruthy();

      const traced = await dependenciesOf(page, run.id, added?.key ?? '');
      expect(traced.status).toBe('succeeded');
      expect(traced.featureName).toBe(REQUEST);
      expect(flatten(traced).length, '확정한 후보 수만큼 행이 실려야 한다').toBe(dependencyCount);
      for (const dependency of flatten(traced)) {
        expect(CATEGORIES as readonly string[]).toContain(dependency.category);
      }

      const additions = await page.request.get(`/api/analyses/${run.id}/features/additions`);
      expect(additions.status()).toBe(200);
      const record = (await additions.json()).additions.find(
        (a: { key: string }) => a.key === added?.key,
      );
      expect(record.status).toBe('confirmed');
      expect(record.source).toBe('user_llm');
      expect(record.evidenceFound).toBe(true);
      expect(record.request).toBe(REQUEST);
    } finally {
      await scaleWorkers(0);
    }
  });
});
