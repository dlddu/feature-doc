// 검증 시나리오: 04-platform.md#시나리오 8
//
// exactly-once의 계약은 backend/tests/worker.rs의
// many_workers_racing_never_claim_the_same_job_twice가 먼저 지키고, 이 파일은 그 위에
// 클러스터 절반을 더한다.
//
// So this file drives `kubectl` against the same kind cluster scripts/e2e.sh
// created, and observes the effect through the public API. That is deliberate:
// asserting the topology through anything *other* than actually scaling the
// workers would only restate the manifest back to itself.
//
// Isolation: the analysis worker is *deployment-wide* state — unlike App installs
// and LLM keys, no per-user handle can isolate it, and a running worker drains the
// queue within seconds. It is leased, not owned: `deploy/e2e/` starts the worker at
// **0 replicas**, this spec scales it up inside its own block and puts the count
// back to 0 in `finally`, and playwright.config.ts pins `workers: 1` so no sibling
// spec file is in flight while it is up. The scale/settle handles live in
// `e2e/support/cluster.ts`, shared with the other lessees. The API — what every
// other spec talks to — is never touched.
//
// Like every spec it signs in as its own stub user (`?as=sc0408`); App installation
// is per-user state and sharing an identity would let specs clobber each other.
import { expect, test, type APIRequestContext } from '@playwright/test';
import { desiredWorkerReplicas, scaleWorkers, workerLogs } from '../support/cluster';
import { installApp } from '../support/github-app';

/**
 * The pre-conditions for enqueuing anything that will actually run. Driven through
 * the same endpoints the UI drives, but directly — this spec's subject is the
 * cluster topology, not the screens.
 */
async function signInWithApp(request: APIRequestContext): Promise<void> {
  const login = await request.get('/api/auth/login?as=sc0408');
  expect(login.ok()).toBeTruthy();
  await installApp(request);

  const connection = await request.get('/api/github/connection');
  expect(connection.ok()).toBeTruthy();
  expect((await connection.json()).installed, 'App must be linked before enqueuing').toBe(true);

  // An active key is the entry condition for an analysis on the stubbed path too, so
  // the topology under test drains the same jobs production would.
  const key = await request.post('/api/llm-keys', {
    data: { provider: 'openai', key: 'sk-proj-8888888888888888888888' },
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

test.describe('시나리오 8: 워커의 수평 확장', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('replica를 늘리면 두 replica가 같은 큐를 나눠 청구하고, 아무 job도 두 번 처리되지 않는다', async ({
    request,
  }) => {
    try {
      // The overlay already rests at 0; the burst has to be enqueued from there so
      // that nothing drains before the replicas are raised.
      await scaleWorkers(0);
      await signInWithApp(request);

      const burst = [await enqueue(request, 'checkout-web'), await enqueue(request, 'notif-worker')];

      await scaleWorkers(2);

      await expect
        .poll(
          async () => {
            const statuses = await Promise.all(burst.map((id) => statusOf(request, id)));
            return statuses.filter((s) => s === 'awaiting_pipeline').length;
          },
          {
            message: '2 queued analyses should drain once 2 workers run',
            timeout: 120_000,
            intervals: [250],
          },
        )
        .toBe(burst.length);

      // Scaling out is unconditional: there is no leader election and no exclusive
      // resource that would make the second replica a no-op.
      expect(desiredWorkerReplicas()).toBe('2');

      // Every line, not the default --tail=10: the claims are counted below.
      const logs = workerLogs();
      const podNames = new Set(
        logs
          .split('\n')
          .map((l) => l.match(/^\[pod\/([^/]+)\//)?.[1])
          .filter(Boolean),
      );
      expect(podNames.size, `two worker pods should be live; saw ${[...podNames]}`).toBe(2);
      const started = logs.split('\n').filter((l) => l.includes('featuredoc worker started'));
      expect(started.length, 'both replicas start and poll, neither crash-loops').toBe(2);

      // Asserted this way rather than as "each pod claimed at least one" because that
      // would depend on which pod won the race: a fast pod legitimately drains the
      // whole burst before its sibling finishes booting. The checks also match on the
      // analysis id rather than counting lines — the queue is global, so a job another
      // spec left `queued` is drained by these same workers.
      const lines = logs.split('\n');
      for (const id of burst) {
        expect(
          lines.filter((l) => l.includes('claimed analysis') && l.includes(id)).length,
          `analysis ${id} must be claimed exactly once`,
        ).toBe(1);

        // And it drained by *doing work*, not by being marked done.
        expect(
          lines.filter((l) => l.includes('fetch stage complete') && l.includes(id)).length,
          `analysis ${id} must have run its fetch stage exactly once`,
        ).toBe(1);
      }
    } finally {
      // Back to the overlay's resting state (0), whatever happened above, so a
      // later spec never finds a worker quietly draining its queue.
      await scaleWorkers(0);
    }
  });
});
