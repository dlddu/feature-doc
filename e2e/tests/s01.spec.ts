import { test, expect } from '@playwright/test';

// GitHub is answered by a standalone mock GitHub server (deployed as `mock-github`),
// so the app runs its real OAuth/App HTTP path while staying offline. The LLM-key
// boundary is still a deterministic shape check (FEATUREDOC_MODE=stub). Drives the
// whole S01 journey in a real browser.
test('S01: sign in → connect GitHub App → register key → continue', async ({ page }) => {
  await page.goto('/');

  // Anonymous: a single primary "Sign in with GitHub".
  const signin = page.getByTestId('signin');
  await expect(signin).toBeVisible();
  await signin.click();

  // Back from the (stub) OAuth round-trip, authenticated but not yet installed.
  // The requested read-only scopes are shown before connecting (journey F1).
  const connect = page.getByTestId('connect-app');
  await expect(connect).toBeVisible();
  await expect(page.getByText('contents:read')).toBeVisible();
  await expect(page.getByText('metadata:read')).toBeVisible();

  // Connect the App (stub setup round-trip) → installed state with repo count.
  await connect.click();
  const connection = page.getByTestId('connection');
  await expect(connection).toBeVisible();
  await expect(connection).toContainText('repositories');

  // An invalid key is rejected with a visible message.
  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill('bad');
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('key-error')).toBeVisible();

  // A valid key registers; only masked identifiers are shown, never the secret.
  const secret = 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa';
  await page.getByTestId('key-input').fill(secret);
  await page.getByTestId('register-key').click();
  const active = page.getByTestId('active-key');
  await expect(active).toBeVisible();
  await expect(active).toContainText('sk-ant-');
  await expect(active).not.toContainText('aaaaaaaaaaaaaaaaaaaa');

  // Continue is now enabled; preflight confirms readiness.
  const cont = page.getByTestId('continue');
  await expect(cont).toBeEnabled();
  await cont.click();
  await expect(page.getByTestId('ready')).toBeVisible();
});
