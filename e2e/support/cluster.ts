// Cluster handles shared by the specs that need the analysis worker to actually
// run (AC4.5 topology, AC1.5 progress).
//
// The worker replica count is *deployment-wide* state: unlike App installs and LLM
// keys, no per-user handle isolates it, and a running worker drains the global
// queue within seconds. It is therefore **leased, not owned**: `deploy/e2e/` keeps
// it at 0 at rest, a spec that needs it scales it up inside its own block and puts
// it back to 0 in `finally`, and `playwright.config.ts` pins `workers: 1` so no
// sibling spec file is ever in flight while it is up. The residual effect a lessee
// must accept: while the worker runs it drains *every* queued job, including ones
// other specs left behind — so no spec may assert on a job it did not create.
// (docs/doc-tracker.md "e2e 매핑" states the same rule in prose.)
//
// This file lives outside `testDir` (`e2e/tests`), so it is neither collected as a
// test nor counted as an AC↔spec matching unit.

import { execFileSync } from 'node:child_process';

const WORKER_DEPLOY = 'deployment/featuredoc-worker';

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
