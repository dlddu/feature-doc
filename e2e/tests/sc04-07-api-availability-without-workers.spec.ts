// 검증 시나리오: 04-platform.md#시나리오 7
//
// 「워커 다운 시 API 가용성」 전용 spec (AC4.5).
//
// AC4.5 is the one AC whose verification lives *below* the browser: its two
// scenarios in docs/test/04-platform.md are stated in terms of pods, not pages.
//
// 시나리오 8(워커의 수평 확장)의 단정은
// `sc04-08-worker-horizontal-scale.spec.ts`가 지킨다 — 한 파일에 있던 두 시나리오를
// 분리한 것은 `rct_20260916-0002`가 닫았다.
//
//   시나리오 7 — 워커 파드를 모두 강제 종료해도 API는 정상 응답하고, 신규 분석 요청은
//                큐에 적재되어 워커 복구 후 처리된다.
//
// So this file drives `kubectl` against the same kind cluster scripts/e2e.sh
// created, and observes the effect through the public API. That is deliberate:
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

/**
 * Signs in as this spec's stub user, links a (stub) App installation and registers
 * an LLM key — the pre-conditions for enqueuing anything that will actually run.
 * All three are the same endpoints the UI drives; this spec calls them directly
 * because its subject is the cluster topology, not the screens (which ac1-1 /
 * ac4-1 / sc04-03 own).
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

  // Same entry condition every other analysis-running spec sets up. This file used
  // to skip it and still drain once the worker came back, because the worker let a
  // keyless job fall back to a default provider whenever the LLM double was on — a
  // stub path more permissive than the real one. That leniency is gone
  // (backend/src/bin/worker.rs `provider_for`), so the recovery this spec asserts
  // now runs the same jobs production would.
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

      // ── 복구: 워커가 돌아오면 대기 job이 처리된다 ────────────────────────
      // (확장의 성질 — 두 replica가 같은 큐를 청구하고 아무 job도 두 번 처리되지
      // 않음 — 은 sc04-08-worker-horizontal-scale.spec.ts의 몫이다.)
      await scaleWorkers(1);

      // The jobs that waited while no worker existed drain once the worker is back.
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
