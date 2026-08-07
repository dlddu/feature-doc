// 검증 AC: AC4.5
//
// AC4.5 (k8s 배포 및 워크로드 분리) 전용 spec.
//
// AC4.5 is the one AC whose verification lives *below* the browser: its two
// scenarios in docs/test/04-platform.md are stated in terms of pods, not pages.
//
//   시나리오 7 — 워커 파드를 모두 강제 종료해도 API는 정상 응답하고, 신규 분석 요청은
//                큐에 적재되어 워커 복구 후 처리된다.
//   시나리오 8 — 워커 replica를 늘리면 처리량이 늘고, API/워커 간 결합 없이 확장된다.
//
// So this file drives `kubectl` against the same kind cluster scripts/e2e.sh
// created, and observes the effect through the public API. That is deliberate:
// asserting the topology through anything *other* than actually removing the
// workers would only restate the manifest back to itself.
//
// Isolation: the analysis worker is *deployment-wide* state — unlike App installs
// and LLM keys, no per-user handle can isolate it, and a running worker drains the
// queue within seconds, which would race ac1-1's `Queued` assertion. Two things
// make that safe: `deploy/e2e/` starts the worker at **0 replicas**, and
// playwright.config.ts pins `workers: 1` so no sibling spec file is in flight
// while this one scales. This spec puts the count back to 0 in `finally`, so a
// failure here cannot leave a worker running for later specs. The API — what
// every other spec talks to — is never touched.
//
// Like every spec it signs in as its own stub user (`?as=ac45`); App installation
// is per-user state and sharing an identity would let specs clobber each other.
import { execFileSync } from 'node:child_process';
import { expect, test, type APIRequestContext } from '@playwright/test';

const WORKER_DEPLOY = 'deployment/featuredoc-worker';

function kubectl(...args: string[]): string {
  return execFileSync('kubectl', args, { encoding: 'utf8', timeout: 120_000 });
}

/** Pod names currently existing for the worker Deployment, in any phase. */
function workerPods(): string[] {
  const out = kubectl(
    'get',
    'pods',
    '-l',
    'app.kubernetes.io/name=featuredoc-worker',
    '-o',
    'jsonpath={.items[*].metadata.name}',
  ).trim();
  return out ? out.split(/\s+/) : [];
}

/**
 * Scales the worker Deployment and waits until the change has actually taken
 * effect at the *pod* level.
 *
 * `kubectl scale` + `rollout status` is not enough on the way down: both return
 * as soon as the Deployment reports the new desired state, while the old pod is
 * still being told to stop. A worker in that window keeps polling and will drain
 * the very queue the next assertion is about to inspect — which is exactly how
 * the first run of this spec failed (a job read back `awaiting_pipeline` five
 * seconds after the workers were supposedly gone). So wait for the pod list
 * itself.
 */
async function scaleWorkers(replicas: number): Promise<void> {
  kubectl('scale', WORKER_DEPLOY, `--replicas=${replicas}`);
  if (replicas > 0) {
    kubectl('rollout', 'status', WORKER_DEPLOY, '--timeout=120s');
  }
  for (let i = 0; i < 120; i++) {
    if (workerPods().length === replicas) return;
    await new Promise((r) => setTimeout(r, 1_000));
  }
  throw new Error(
    `worker pods did not settle at ${replicas}; still see ${workerPods().join(', ') || '(none)'}`,
  );
}

/**
 * Signs in as this spec's stub user and links a (stub) App installation — the
 * pre-condition for enqueuing anything. Both are the same endpoints the UI drives;
 * this spec calls them directly because its subject is the cluster topology, not
 * the screens (which ac1-1 / ac4-1 own).
 */
async function signInWithApp(request: APIRequestContext): Promise<void> {
  const login = await request.get('/api/auth/login?as=ac45');
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

test.describe('AC4.5: API 워크로드와 분석 워커 워크로드의 분리', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('워커 0개에서도 API는 응답하고 큐는 보존되며, 워커를 늘리면 중복 없이 드레인된다', async ({
    request,
  }) => {
    try {
      // ── 시나리오 7: 워커를 전부 내린다 ──────────────────────────────────
      // The overlay already starts the worker at 0; this makes the precondition
      // explicit (and re-establishes it if the deployment was left running).
      await scaleWorkers(0);
      await signInWithApp(request);

      const queuedIds = [
        await enqueue(request, 'payments-api'),
        await enqueue(request, 'checkout-web'),
      ];

      // The API is unaffected by the worker being gone — credential reads,
      // result reads, and new triggers all still answer.
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

      // ── 시나리오 8: 워커를 2개로 늘린다 ─────────────────────────────────
      const alsoQueued = [
        await enqueue(request, 'notif-worker'),
        await enqueue(request, 'payments-api'),
      ];
      const allIds = [...queuedIds, ...alsoQueued];

      await scaleWorkers(2);

      // Every job the queue held — including the ones enqueued while no worker
      // existed — drains once workers exist again.
      await expect
        .poll(
          async () => {
            const statuses = await Promise.all(allIds.map((id) => statusOf(request, id)));
            return statuses.filter((s) => s === 'awaiting_pipeline').length;
          },
          {
            message: '4 queued analyses should drain once 2 workers run',
            timeout: 120_000,
            intervals: [1_000],
          },
        )
        .toBe(allIds.length);

      // Scaling out is unconditional: two replicas both reach Ready and both poll
      // the same queue. There is no leader election or exclusive resource that
      // would make the second one a no-op — which is what "API/워커 간 결합 없이
      // 확장된다" asks for.
      expect(kubectl('get', WORKER_DEPLOY, '-o', 'jsonpath={.spec.replicas}').trim()).toBe('2');

      const logs = kubectl(
        'logs',
        '-l',
        'app.kubernetes.io/name=featuredoc-worker',
        '--prefix',
        // `-l` defaults to --tail=10; we need every line to count claims.
        '--tail=-1',
      );
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
      // legitimately drains the whole burst before its sibling finishes booting;
      // the disjointness of concurrent claims is gated by
      // backend/tests/worker.rs::many_workers_racing_never_claim_the_same_job_twice.)
      const claimLines = logs.split('\n').filter((l) => l.includes('claimed analysis'));
      for (const id of allIds) {
        const claims = claimLines.filter((l) => l.includes(id));
        expect(claims.length, `analysis ${id} must be claimed exactly once`).toBe(1);
      }

      // The queue drained by *doing work*, not by being marked done: the one
      // implemented stage ran on every job. (Rendering it is S04 / AC1.5.)
      const completed = logs.split('\n').filter((l) => l.includes('fetch stage complete'));
      expect(completed.length).toBe(allIds.length);
    } finally {
      // Back to the overlay's resting state (0), whatever happened above, so a
      // later spec never finds a worker quietly draining its queue.
      await scaleWorkers(0);
    }
  });
});
