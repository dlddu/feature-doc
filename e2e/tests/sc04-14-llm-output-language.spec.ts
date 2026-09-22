// 검증 시나리오: 04-platform.md#시나리오 14
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_LLM_KEY=stub). Signs in as its own
// stub identity (`?as=sc0414`); the language setting is per-user state. What language
// the stub model "writes" in is not observable — the stub answers a fixed document —
// so the spec asserts what the product records: the setting, and the language each
// analysis was fixed to when it started. The prompt text itself is unit-tested in
// `backend/src/llm.rs`.
import { expect, test } from '@playwright/test';

const OPENAI_KEY = 'sk-proj-dddddddddddddddddddddd';

test('AC4.9: 출력 언어는 사용자별로 저장되고, 분석은 시작 시점의 언어로 고정된다', async ({
  page,
}) => {
  await page.goto('/api/auth/login?as=sc0414');

  await page.getByTestId('connect-app').click();
  await expect(page.getByTestId('connection')).toBeVisible();

  await expect(page.getByTestId('lang-ko').locator('input')).toBeChecked();
  await expect(page.getByTestId('lang-en').locator('input')).not.toBeChecked();

  await page.getByTestId('lang-en').click();
  await expect(page.getByTestId('lang-en').locator('input')).toBeChecked();
  await expect
    .poll(async () => (await (await page.request.get('/api/settings')).json()).llmLanguage)
    .toBe('en');

  await page.getByTestId('provider-openai').click();
  await page.getByTestId('key-input').fill(OPENAI_KEY);
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toBeVisible();

  await page.reload();
  await expect(page.getByTestId('lang-en').locator('input')).toBeChecked();

  const first = await page.request.post('/api/analyses', {
    data: { repoUrl: 'stub-account/payments-api', branch: null },
  });
  expect(first.status()).toBe(201);
  const firstRun = await first.json();
  expect(firstRun.llmLanguage).toBe('en');

  await page.getByTestId('lang-ko').click();
  await expect(page.getByTestId('lang-ko').locator('input')).toBeChecked();
  await expect
    .poll(async () => (await (await page.request.get('/api/settings')).json()).llmLanguage)
    .toBe('ko');

  const stillEn = await page.request.get(`/api/analyses/${firstRun.id}`);
  expect(stillEn.ok()).toBe(true);
  expect((await stillEn.json()).llmLanguage).toBe('en');

  const second = await page.request.post('/api/analyses', {
    data: { repoUrl: 'stub-account/payments-api', branch: null },
  });
  expect(second.status()).toBe(201);
  expect((await second.json()).llmLanguage).toBe('ko');

  const refused = await page.request.put('/api/settings', { data: { llmLanguage: 'ja' } });
  expect(refused.status()).toBe(400);
  expect((await (await page.request.get('/api/settings')).json()).llmLanguage).toBe('ko');
});
