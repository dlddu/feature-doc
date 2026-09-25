// 검증 시나리오: 04-platform.md#시나리오 2
//
// 재는 것은 "해제를 알아챘는가"가 아니라 채택된 정책이 그대로 일어났는가다.
//
// Leases the analysis worker *and* one API env var — lease rules in
// `e2e/support/cluster.ts`.
import { expect, test, type Page } from '@playwright/test';
import { scaleWorkers, setApiEnv } from '../support/cluster';
import { installApp } from '../support/github-app';

/** 더블이 그리는 설치의 전체 범위(`backend/src/github_app.rs`의 stub 세 저장소). */
const ALL_REPOS = 'payments-api,checkout-web,notif-worker';
const ACCESS = 'FEATUREDOC_STUB_REPO_ACCESS';

async function analysisOf(page: Page, id: string) {
  const res = await page.request.get(`/api/analyses/${id}`);
  expect(res.ok()).toBeTruthy();
  return (await res.json()) as {
    status: string;
    error: string | null;
    accessRevoked: boolean;
    stages: { key: string; status: string }[];
  };
}

test.describe('AC4.1: 진행 중 분석 도중 App 설치 해제', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('접근이 해제되면 분석은 현재 호출까지만 마무리하고 멈추며, 그 사유가 통지된다', async ({
    page,
  }) => {
    try {
      // 큐를 비운 채로 분석을 세워 둔다 — "진행 중"이 워커 타이밍에 흔들리지 않게.
      await scaleWorkers(0);

      await page.goto('/api/auth/login?as=ac41b');
      await installApp(page.request);
      const key = await page.request.post('/api/llm-keys', {
        data: { provider: 'anthropic', key: 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa' },
      });
      expect(key.ok(), 'an active LLM key is 홈 화면의 진입 조건').toBeTruthy();

      const started = await page.request.post('/api/analyses', {
        data: { repoUrl: 'stub-account/payments-api', branch: null },
      });
      expect(started.status(), 'enqueue payments-api').toBe(201);
      const id = (await started.json()).id as string;

      expect((await analysisOf(page, id)).status).toBe('queued');
      await page.goto(`/#/analyses/${id}`);
      await expect(page.getByTestId('access-revoked')).toHaveCount(0);

      await setApiEnv(ACCESS, 'checkout-web,notif-worker');

      await scaleWorkers(1);

      await expect
        .poll(() => analysisOf(page, id).then((a) => a.status), {
          timeout: 120_000,
          intervals: [1_000],
        })
        .toBe('failed');

      const stopped = await analysisOf(page, id);
      expect(stopped.accessRevoked, '해제로 멈춘 분석임이 서버 판정으로 온다').toBe(true);
      expect(stopped.error).toContain('저장소 접근이 해제되어');
      expect(
        stopped.stages.every((s) => s.status !== 'running'),
        '멈춘 분석에 실행 중인 단계가 남지 않는다',
      ).toBe(true);

      await page.goto(`/#/analyses/${id}`);
      const notice = page.getByTestId('access-revoked');
      await expect(notice).toBeVisible();
      await expect(notice).toHaveText(stopped.error!);
      await expect(page.getByTestId('retry')).toHaveCount(0);
      await expect(page.getByTestId('stage-failed')).toHaveCount(0);

      await setApiEnv(ACCESS, ALL_REPOS);
      const again = await page.request.post('/api/analyses', {
        data: { repoUrl: 'stub-account/payments-api', branch: null },
      });
      expect(again.status(), '접근을 되돌린 뒤에는 다시 시작된다').toBe(201);
    } finally {
      await scaleWorkers(0);
      await setApiEnv(ACCESS, null);
    }
  });
});
