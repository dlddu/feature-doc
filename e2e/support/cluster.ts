// Cluster handles shared by the specs that need the analysis worker to actually
// run.
//
// The worker replica count is *deployment-wide* state: unlike App installs and LLM
// keys, no per-user handle isolates it, and a running worker drains the global
// queue within seconds. It is therefore **leased, not owned**: `deploy/e2e/` keeps
// it at 0 at rest, a spec that needs it scales it up inside its own block and puts
// it back to 0 in `finally`, and `playwright.config.ts` pins `workers: 1` so no
// sibling spec file is ever in flight while it is up. The residual effect a lessee
// must accept: while the worker runs it drains *every* queued job, including ones
// other specs left behind — so no spec may assert on a job it did not create.

import { execFileSync } from 'node:child_process';

const WORKER_DEPLOY = 'deployment/featuredoc-worker';
const API_DEPLOY = 'deployment/featuredoc';

export function kubectl(...args: string[]): string {
  return execFileSync('kubectl', args, { encoding: 'utf8', timeout: 120_000 });
}

/** Pod names currently existing for the worker Deployment, in any phase. */
export function workerPods(): string[] {
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

/** The Deployment's currently desired replica count, as a string. */
export function desiredWorkerReplicas(): string {
  return kubectl('get', WORKER_DEPLOY, '-o', 'jsonpath={.spec.replicas}').trim();
}

/** Every line the worker pods have logged (`-l` defaults to --tail=10). */
export function workerLogs(): string {
  return kubectl(
    'logs',
    '-l',
    'app.kubernetes.io/name=featuredoc-worker',
    '--prefix',
    '--tail=-1',
  );
}

/**
 * Scales the worker Deployment and waits until the change has actually taken
 * effect at the *pod* level.
 *
 * `kubectl scale` + `rollout status` is not enough on the way down: both return as
 * soon as the Deployment reports the new desired state, while the old pod is still
 * being told to stop. A worker in that window keeps polling and will drain the very
 * queue the next assertion is about to inspect — which is exactly how the first run
 * of ac4-5 failed (a job read back `awaiting_pipeline` five seconds after the
 * workers were supposedly gone). So wait for the pod list itself.
 */
export async function scaleWorkers(replicas: number): Promise<void> {
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
 * Sets (or clears, with `value = null`) one env var on the worker Deployment.
 *
 * Like the replica count this is deployment-wide state, so the same lease rule
 * applies: a spec sets the var *after* `scaleWorkers(0)` and clears it *after*
 * scaling back down. The pods that start inside the lease window carry it, and
 * nothing outside that window ever sees it. Env edits rewrite the pod template,
 * which restarts pods on their own — keeping the ordering above means that
 * restart happens at a moment no pod is running.
 *
 * Used by sc01-06's LLM failure arc (blocker-ledger R1 in
 * docs/e2e-mocking-policy.md — `FEATUREDOC_STUB_LLM_FAIL`).
 */
export function setWorkerEnv(key: string, value: string | null): void {
  if (value === null) {
    kubectl('set', 'env', WORKER_DEPLOY, `${key}-`);
  } else {
    kubectl('set', 'env', WORKER_DEPLOY, `${key}=${value}`);
  }
}

/** Pod names currently existing for the API Deployment, in any phase. */
function apiPods(): string[] {
  const out = kubectl(
    'get',
    'pods',
    '-l',
    'app.kubernetes.io/name=featuredoc',
    '-o',
    'jsonpath={.items[*].metadata.name}',
  ).trim();
  return out ? out.split(/\s+/) : [];
}

/** Bumped by the API server whenever an edit actually rewrites the pod template. */
function apiGeneration(): string {
  return kubectl('get', API_DEPLOY, '-o', 'jsonpath={.metadata.generation}').trim();
}

/**
 * Blocks until `BASE_URL` answers again.
 *
 * This is the assertion that matters, and it is deliberately *not* `rollout status`:
 * the pod is not what the specs talk to. Every request goes through the single
 * `kubectl port-forward` that `scripts/e2e.sh` owns, and that forward is bound to one
 * pod — when the pod goes, the forward exits and the harness has to respawn it.
 * Waiting on the Deployment instead let sc04-02 continue into that window, and the
 * eleven sc04-* specs queued behind it went down with it on `ECONNREFUSED :8080`.
 */
async function waitForApi(): Promise<void> {
  const base = process.env.BASE_URL ?? 'http://localhost:8080';
  let last = 'no attempt made';
  for (let i = 0; i < 240; i++) {
    try {
      const res = await fetch(`${base}/hello`);
      if (res.ok) return;
      last = `HTTP ${res.status}`;
    } catch (e) {
      last = e instanceof Error ? e.message : String(e);
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`${base}/hello did not come back within 120s (last: ${last})`);
}

/**
 * Sets (or clears, with `value = null`) one env var on the **API** Deployment and
 * returns only once `BASE_URL` is served by a pod that carries the new value.
 *
 * Deployment-wide state with the same lease rule as `setWorkerEnv`: set it inside
 * the spec's own block, clear it in `finally`. Used by sc04-02 to take repository
 * access away the way a user does on GitHub (`FEATUREDOC_STUB_REPO_ACCESS`,
 * `backend/src/github_app.rs`).
 *
 * Unlike the worker, the API cannot be restarted quietly: it is `strategy: Recreate`
 * (SQLite on a ReadWriteOnce volume), so the old pod is gone *before* the new one
 * starts and there is a window with nothing serving at all. Hence the three waits
 * below — new pod, old pod gone, port actually answering — and hence the caller must
 * `await` this. A rollout that nobody waited through is what broke the first run of
 * sc04-02 and every sc04-* spec after it.
 */
export async function setApiEnv(key: string, value: string | null): Promise<void> {
  const generationBefore = apiGeneration();
  const podsBefore = new Set(apiPods());
  if (value === null) {
    kubectl('set', 'env', API_DEPLOY, `${key}-`);
  } else {
    kubectl('set', 'env', API_DEPLOY, `${key}=${value}`);
  }
  // A no-op edit (clearing a var that was never set) rewrites nothing, so there is no
  // rollout to wait for — and waiting for a pod swap that will never come would just
  // burn the spec's timeout.
  if (apiGeneration() !== generationBefore) {
    kubectl('rollout', 'status', API_DEPLOY, '--timeout=180s');
    for (let i = 0; i < 240; i++) {
      const pods = apiPods();
      if (pods.length > 0 && pods.every((name) => !podsBefore.has(name))) break;
      await new Promise((r) => setTimeout(r, 500));
    }
  }
  await waitForApi();
}
