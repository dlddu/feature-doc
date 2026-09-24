// 검증 시나리오: 04-platform.md#시나리오 7
//
// This scenario's verification lives *below* the browser: it is stated in terms of
// pods, not pages. So this file drives `kubectl` against the same kind cluster
// scripts/e2e.sh created, and observes the effect through the public API. That is
// deliberate:
// asserting the topology through anything *other* than actually removing the
// workers would only restate the manifest back to itself.
//
// Isolation: the analysis worker is *deployment-wide* state — unlike App installs
// and LLM keys, no per-user handle can isolate it, and a running worker drains the
// queue within seconds, which would race ac1-1's `Queued` assertion. It is leased,
// not owned: `deploy/e2e/` starts the worker at **0 replicas**, this spec scales it
// up inside its own block and puts the count back to 0 in `finally`, and
// playwright.config.ts pins `workers: 1` so no sibling spec file is in flight while
// it is up. The scale/settle handles live in `e2e/support/cluster.ts`, shared with
// the other lessee (ac1-5). The API — what every other spec talks to — is never
// touched.
//
// Like every spec it signs in as its own stub user (`?as=ac45`); App installation
// is per-user state and sharing an identity would let specs clobber each other.
import { expect, test, type APIRequestContext } from '@playwright/test';
import { scaleWorkers } from '../support/cluster';
import { installApp } from '../support/github-app';

/**
 * The pre-conditions for enqueuing anything that will actually run. Driven through
 * the same endpoints the UI drives, but directly — this spec's subject is the
 * cluster topology, not the screens.
 */
async function signInWithApp(request: APIRequestContext): Promise<void> {
  const login = await request.get('/api/auth/login?as=ac45');
  expect(login.ok()).toBeTruthy();
  await installApp(request);

  const connection = await request.get('/api/github/connection');
  expect(connection.ok()).toBeTruthy();
  expect((await connection.json()).installed, 'App must be linked before enqueuing').toBe(true);

  // An active key is the entry condition for an analysis on the stubbed path too, so
  // the recovery asserted below runs the same jobs production would.
  const key = await request.post('/api/llm-keys', {
    data: { provider: 'openai', key: 'sk-proj-7777777777777777777777' },
  });
  expect(key.ok(), 'an active LLM key is 분석의 진입 조건').toBeTruthy();
}

async function enqueue(request: APIRequestContext, repo: string): Promise<string> {
  const res = await request.post('/api/analyses', {
    data: { repoUrl: `stub-account/${repo}` },
  });
  expect(res.status()).toBe(201);
  return (await res.json()).id as string;
}

async function statusOf(request: APIRequestContext, id: string): Promise<string> {
  const res = await request.get('/api/analyses');
  expect(res.ok()).toBeTruthy();
  const rows = (await res.json()) as Array<{ id: string; status: string }>;
  const row = rows.find((r) => r.id === id);
  expect(row, `analysis ${id} missing from /api/analyses`).toBeTruthy();
  return row!.status;
}

test.describe('AC4.5: API 워크로드와 분석 워커 워크로드의 분리', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('워커가 모두 내려가도 API는 응답하고 큐는 보존되며, 복구하면 대기 job이 처리된다', async ({
    request,
  }) => {
    try {
      // The overlay already starts the worker at 0; this makes the precondition
      // explicit (and re-establishes it if the deployment was left running).
      await scaleWorkers(0);
      await signInWithApp(request);

      const queuedIds = [
        await enqueue(request, 'payments-api'),
        await enqueue(request, 'checkout-web'),
      ];

      for (const path of ['/api/me', '/api/llm-keys', '/api/analyses', '/api/repositories']) {
        const res = await request.get(path);
        expect(res.status(), `${path} while no worker runs`).toBe(200);
      }

      // Give the queue a window in which a (nonexistent) worker could have taken
      // it, then assert nothing moved: the work is waiting, not lost.
      await new Promise((r) => setTimeout(r, 5_000));
      for (const id of queuedIds) {
        expect(await statusOf(request, id), 'no worker ⇒ the job waits').toBe('queued');
      }

      await scaleWorkers(1);

      await expect
        .poll(
          async () => {
            const statuses = await Promise.all(queuedIds.map((id) => statusOf(request, id)));
            return statuses.filter((s) => s === 'awaiting_pipeline').length;
          },
          {
            message: 'queued analyses should drain once the worker recovers',
            timeout: 120_000,
            intervals: [1_000],
          },
        )
        .toBe(queuedIds.length);
    } finally {
      // Back to the overlay's resting state (0), whatever happened above, so a
      // later spec never finds a worker quietly draining its queue.
      await scaleWorkers(0);
    }
  });
});
