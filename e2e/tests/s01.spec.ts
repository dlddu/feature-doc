import { test, expect } from '@playwright/test';

// S01 Credentials Setup, driven against the real backend in mock mode:
// sign in → connect the GitHub App → register an LLM key → reach a ready
// state. No external GitHub/LLM contact happens (GITHUB_MODE/LLM_MODE=mock).

test('S01: sign in, connect GitHub App, register an LLM key', async ({ page }) => {
  await page.goto('/');

  // Signed out: the screen offers GitHub sign-in.
  const signIn = page.getByTestId('signin-btn');
  await expect(signIn).toBeVisible();
  await expect(page.locator('.h-display')).toHaveText('Bring your own keys');

  // Sign in (mock OAuth round-trip lands us back on the SPA, authenticated).
  await signIn.click();
  await expect(page.getByTestId('github-section')).toBeVisible();

  // Not connected yet: the minimum permissions are advertised before install.
  await expect(page.getByTestId('github-status')).toHaveText('Not connected');
  await expect(page.locator('.tag', { hasText: 'contents:read' })).toBeVisible();

  // Connect the App (mock setup callback redirects back to the SPA).
  await page.getByTestId('connect-btn').click();
  await expect(page.getByTestId('github-status')).toHaveText('Installed');
  await expect(page.locator('.input .val', { hasText: 'repositories' })).toBeVisible();

  // Continue is gated until a key exists.
  await expect(page.getByTestId('continue-btn')).toBeDisabled();

  // An obviously invalid key is rejected with an error, and nothing is stored.
  await page.getByTestId('key-input').fill('sk-ant-invalid-xxxxxxxx');
  await page.getByTestId('key-submit').click();
  await expect(page.getByTestId('key-error')).toBeVisible();

  // A well-formed key validates, is stored, and comes back masked.
  await page.getByTestId('key-input').fill('sk-ant-realkey-abcd12345678');
  await page.getByTestId('key-submit').click();
  await expect(page.getByTestId('key-status')).toHaveText('Active');
  const masked = await page.getByTestId('key-masked').textContent();
  expect(masked).toContain('•');
  expect(masked).not.toContain('realkey');

  // With both credentials in place, Continue is enabled.
  await expect(page.getByTestId('continue-btn')).toBeEnabled();

  // Revocation returns to the entry state (AC4.2).
  await page.getByTestId('revoke-btn').click();
  await expect(page.getByTestId('key-input')).toBeVisible();
  await expect(page.getByTestId('continue-btn')).toBeDisabled();
});
