import { test, expect } from '@playwright/test';

test('frontend renders backend hello message', async ({ page }) => {
  await page.goto('/');
  const heading = page.locator('.h-display');
  await expect(heading).toHaveText('Hello from FeatureDoc backend');
});
