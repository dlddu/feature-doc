// The walk to "stage 5 has written its acceptance document", shared by the three
// AC2.x specs.
//
// It is here rather than copied into each spec for the same reason `cluster.ts` is:
// the three specs assert *different* properties of the same document, and three
// copies of the same 60-line pipeline walk would drift. What each spec still owns
// itself is its stub identity (`?as=<handle>`) and its assertions — the setup is
// shared, the verification is not.
//
// This file lives outside `testDir` (`e2e/tests`), so it is neither collected as a
// test nor counted as an AC↔spec matching unit.

import { expect, type Page } from '@playwright/test';

export type AcceptanceScenario = {
  given: string;
  when: string;
  then: string;
  evidence: string;
  symbol: string | null;
  source: 'logic' | 'test';
};

export type AcceptanceContradiction = {
  given: string;
  when: string;
  codeSays: string;
  codeEvidence: string;
  testSays: string;
  testEvidence: string;
};

export type FeatureDoc = {
  key: string;
  name: string;
  location: string;
  symbol: string | null;
  scenarios: AcceptanceScenario[];
  contradictions: AcceptanceContradiction[];
};

type Candidate = { key: string; name: string; decision: string; mergedInto: string | null };

/** The acceptance document, or `null` while stage 5 has not written one (404). */
export async function acceptanceOf(page: Page, id: string): Promise<FeatureDoc[] | null> {
  const res = await page.request.get(`/api/analyses/${id}/documents/acceptance-dependencies`);
  if (res.status() === 404) return null;
  expect(res.status(), `acceptance document for ${id}`).toBe(200);
  return (await res.json()).content.features as FeatureDoc[];
}

export async function candidatesOf(page: Page, id: string): Promise<Candidate[]> {
  const res = await page.request.get(`/api/analyses/${id}/candidates`);
  expect(res.status(), `candidates for ${id}`).toBe(200);
  return (await res.json()).candidates as Candidate[];
}

async function statusOf(page: Page, id: string): Promise<string> {
  const res = await page.request.get(`/api/analyses/${id}`);
  expect(res.ok()).toBeTruthy();
  return (await res.json()).status as string;
}

/** Signs in as this spec's own stub user and gives it the App install and LLM key
 *  every analysis needs. Set up through the API, not through S01's screens — walking
 *  another AC's screen is setup, not verification. */
export async function signInWithCredentials(page: Page, handle: string): Promise<void> {
  await page.goto(`/api/auth/login?as=${handle}`);
  expect((await page.request.get('/api/github/setup?installation_id=4242')).ok()).toBeTruthy();
  const key = await page.request.post('/api/llm-keys', {
    data: { provider: 'openai', key: 'sk-proj-dddddddddddddddddddddd' },
  });
  expect(key.ok(), 'an active LLM key is 분석의 진입 조건').toBeTruthy();
}

/**
 * Walks one analysis from "queued" to "stage 5 wrote its document", confirming the
 * first `confirm` candidates on the way.
 *
 * Assumes a worker is already running — the caller leases it (`scaleWorkers`) so the
 * lease and its `finally` stay visible in the spec that owns them.
 */
export async function runToAcceptance(
  page: Page,
  repo: string,
  confirm = 1,
): Promise<{ id: string; features: FeatureDoc[]; confirmed: string[] }> {
  const created = await page.request.post('/api/analyses', {
    data: { repoUrl: `stub-account/${repo}`, branch: null },
  });
  expect(created.status(), `enqueue ${repo}`).toBe(201);
  const id = (await created.json()).id as string;

  await expect
    .poll(() => statusOf(page, id), { timeout: 120_000, intervals: [1_000] })
    .toBe('awaiting_pipeline');

  // Reading materialises the reviewable strategy (AC1.3's lazy seed); approving is
  // what re-queues the job so stage 4 gets a turn.
  expect((await page.request.get(`/api/analyses/${id}/discovery-strategy`)).ok()).toBeTruthy();
  expect(
    (await page.request.post(`/api/analyses/${id}/discovery-strategy/approve`)).ok(),
    'approve the strategy',
  ).toBeTruthy();

  await expect
    .poll(() => candidatesOf(page, id).then((c) => c.length), {
      timeout: 120_000,
      intervals: [1_000],
    })
    .toBeGreaterThan(0);

  // Confirming a feature is what opens stage 5 — AC2.1 is about a *confirmed*
  // feature, so this approval is part of the walk rather than an assertion.
  const open = (await candidatesOf(page, id)).filter((c) => c.mergedInto === null);
  expect(open.length, `${repo} must offer at least ${confirm} candidate(s)`).toBeGreaterThanOrEqual(
    confirm,
  );
  const confirmed = open.slice(0, confirm).map((c) => c.key);
  for (const key of confirmed) {
    const res = await page.request.post(`/api/analyses/${id}/candidates/decision`, {
      data: { key, decision: 'approve' },
    });
    expect(res.ok(), `approve candidate ${key}`).toBeTruthy();
  }

  await expect
    .poll(() => acceptanceOf(page, id).then((f) => f?.length ?? 0), {
      timeout: 120_000,
      intervals: [1_000],
    })
    .toBe(confirm);

  return { id, features: (await acceptanceOf(page, id)) as FeatureDoc[], confirmed };
}
