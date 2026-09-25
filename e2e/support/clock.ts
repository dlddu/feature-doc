/**
 * Waits until the wall clock is past `unixSeconds`.
 *
 * Stage `startedAt` is stored in whole unix seconds, and the specs that prove a
 * re-run do it by seeing `startedAt` change. With the stub LLM and a fast-polling
 * worker, the first run and the re-run can both start inside one second and read
 * back equal — the re-run happened, but the spec cannot see it. Triggering the
 * re-run only after that second has passed makes the comparison sound. The kind
 * cluster runs on this machine, so the two share one clock.
 */
export async function afterSecond(unixSeconds: number): Promise<void> {
  const waitMs = (unixSeconds + 1) * 1_000 - Date.now();
  if (waitMs > 0) await new Promise((r) => setTimeout(r, waitMs));
}
