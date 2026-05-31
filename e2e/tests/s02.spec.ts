import { test, expect } from '@playwright/test';

// Runs against the stub-mode deployment (FEATUREDOC_MODE=stub): GitHub and LLM
// boundaries are answered by in-process test doubles — no network. Completes the
// S01 credentials journey (so the setup gate opens), then lands on the S02
// Repositories home and asserts its structure.
test('S02: once credentials are ready, "/" shows the Repositories home', async ({ page }) => {
  // Seed readiness through S01: sign in → connect the App → register a key.
  await page.goto('/'); // not ready yet → gate redirects to /setup
  await page.getByTestId('signin').click();
  await page.getByTestId('connect-app').click();
  await expect(page.getByTestId('connection')).toBeVisible();

  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill('sk-ant-api03-aaaaaaaaaaaaaaaaaaaa');
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toBeVisible();

  // Credentials are ready: "/" now resolves to the home instead of redirecting.
  await page.goto('/');
  const home = page.getByTestId('home');
  await expect(home).toBeVisible();

  // The one display heading.
  await expect(home.getByRole('heading', { name: 'Repositories' })).toBeVisible();

  // Aggregate metrics — no repos connected yet, so the count is 0.
  await expect(page.getByTestId('metrics')).toBeVisible();
  await expect(page.getByTestId('metric-repos')).toHaveText('0');

  // Empty state: connecting a repository is S03 (out of scope), so the list is empty.
  await expect(page.getByTestId('repos-empty')).toBeVisible();

  // "+ New" is inert here; the Keys tab routes back to setup.
  await expect(page.getByTestId('new-repo')).toBeDisabled();
  await expect(page.getByTestId('tab-keys')).toBeVisible();
});
