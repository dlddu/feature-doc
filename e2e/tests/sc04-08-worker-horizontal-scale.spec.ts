// 검증 시나리오: 04-platform.md#시나리오 8
//
// 「워커의 수평 확장」 전용 spec (AC4.5).
//
// docs/test/04-platform.md 시나리오 8을 그대로 따라간다:
//   다수 분석 요청이 큐에 적재된 상태에서 워커 replica를 늘리면, 처리가 비례적으로
//   이어지고 API/워커 간 결합 없이 확장된다.
//
// 이 단정은 `sc04-07-api-availability-without-workers.spec.ts`와 한 파일에 있었다
// (선언은 시나리오 7 하나) — 규칙 2 상 분리 대상으로 등재됐다가 이 파일로 옮겨왔다.
// 여기서 시나리오 7의 몫(워커 다운 시 API 가용성·큐 보존·복구)은 sc04-07이 지킨다;
// 이 파일은 확장 자체 — 두 replica가 같은 큐를 청구하고, 아무 job도 두 번 처리되지
// 않는 성질 — 을 검증한다. exactly-once의 계약은
// backend/tests/worker.rs::many_workers_racing_never_claim_the_same_job_twice가
// 먼저 지키고, 이 파일은 클러스터 절반을 더한다.
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

/**
 * Signs in as this spec's stub user and links a (stub) App installation — the
 * pre-condition for enqueuing anything. Both are the same endpoints the UI drives;
 * this spec calls them directly because its subject is the cluster topology, not
 * the screens (which sc01-02 / sc04-01 own).
 */
async function signInWithApp(request: APIRequestContext): Promise<void> {
  const login = await request.get('/api/auth/login?as=sc0408');
  expect(login.ok()).toBeTruthy();
  // The App "Setup URL" callback is a GET that redirects back into the SPA.
  const setup = await request.get('/api/github/setup?installation_id=4242');
  expect(setup.ok()).toBeTruthy();

  const connection = await request.get('/api/github/connection');
  expect(connection.ok()).toBeTruthy();
  expect((await connection.json()).installed, 'App must be linked before enqueuing').toBe(true);
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
      // The overlay already rests at 0; start the burst from there, as the
      // scenario's precondition states (다수 분석 요청이 큐에 적재된 상태).
      await scaleWorkers(0);
      await signInWithApp(request);

      const burst = [await enqueue(request, 'checkout-web'), await enqueue(request, 'notif-worker')];

      // ── 워커를 2개로 늘린다 ──────────────────────────────────────────────
      await scaleWorkers(2);

      // Every job the burst enqueued drains once the two replicas run.
      await expect
        .poll(
          async () => {
            const statuses = await Promise.all(burst.map((id) => statusOf(request, id)));
            return statuses.filter((s) => s === 'awaiting_pipeline').length;
          },
          {
            message: '2 queued analyses should drain once 2 workers run',
            timeout: 120_000,
            intervals: [1_000],
          },
        )
        .toBe(burst.length);

      // Scaling out is unconditional: two replicas both reach Ready and both poll
      // the same queue. There is no leader election or exclusive resource that
      // would make the second one a no-op — which is what 「API/워커 간 결합 없이
      // 확장된다」 asks for.
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

      // No job is processed twice. This is the property that makes scaling *safe*
      // — and unlike "each pod claimed at least one", it does not depend on which
      // pod happened to win the race, so it is deterministic in CI. (A fast pod
      // legitimately drains the whole burst before its sibling finishes booting.)
      // Note the checks match on the analysis id rather than counting lines. The
      // queue is global — a job another spec left `queued` is drained by these same
      // workers, so a bare line count is not this spec's to assert.
      const lines = logs.split('\n');
      for (const id of burst) {
        expect(
          lines.filter((l) => l.includes('claimed analysis') && l.includes(id)).length,
          `analysis ${id} must be claimed exactly once`,
        ).toBe(1);

        // And it drained by *doing work*, not by being marked done: the one
        // implemented stage ran. (Rendering it is Analysis Progress / sc01-05.)
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
