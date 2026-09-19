// The walk to "this feature's dependencies have been traced", shared by the three
// AC2.4/AC2.5 specs.
//
// Here rather than copied into each spec for the same reason `acceptance.ts` is: the
// three specs assert *different* properties (the seven-way split, the reverse query,
// the export) of the same rows, and three copies of the same request-and-wait would
// drift. What each spec still owns is its stub identity and its assertions.
//
// This file lives outside `testDir` (`e2e/tests`), so it is neither collected as a
// test nor counted as a scenario↔spec matching unit.

import { expect, type Page } from '@playwright/test';

/** The seven categories AC2.4 lists. The server's `dependencies::CATEGORIES` is the
 *  definition; this is the e2e's copy of the same seven, and a spec that sees an
 *  eighth kind should fail rather than quietly widen. */
export const CATEGORIES = [
  'infrastructure',
  'data',
  'architecture',
  'framework',
  'middleware',
  'logic',
  'interface',
] as const;

export type Dependency = { category: string; name: string; evidence: string | null };

export type FeatureDependencies = {
  featureKey: string;
  featureName: string | null;
  status: string | null;
  error: string | null;
  categories: { category: string; items: Dependency[] }[];
  total: number;
  withoutEvidence: number;
};

export async function dependenciesOf(
  page: Page,
  id: string,
  key: string,
): Promise<FeatureDependencies> {
  const res = await page.request.get(
    `/api/analyses/${id}/features/dependencies?key=${encodeURIComponent(key)}`,
  );
  expect(res.status(), `dependencies for ${key}`).toBe(200);
  return (await res.json()) as FeatureDependencies;
}

/** 「의존성 분석」 — the request that re-queues the analysis so the worker traces it. */
export async function requestTrace(page: Page, id: string, key: string): Promise<void> {
  const res = await page.request.post(`/api/analyses/${id}/features/dependencies`, {
    data: { key },
  });
  expect(res.status(), `request a trace of ${key}`).toBe(200);
}

/** Asks for one feature's dependencies and waits for the worker to answer. */
export async function traced(page: Page, id: string, key: string): Promise<Dependency[]> {
  await requestTrace(page, id, key);
  await expect
    .poll(() => dependenciesOf(page, id, key).then((d) => d.status), {
      timeout: 120_000,
      intervals: [1_000],
    })
    .toBe('succeeded');
  return flatten(await dependenciesOf(page, id, key));
}

export function flatten(view: FeatureDependencies): Dependency[] {
  return view.categories.flatMap((group) => group.items);
}
