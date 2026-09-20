// 검증 시나리오: 04-platform.md#시나리오 4
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_LLM_KEY=stub). Signs in as its own
// stub identity (`?as=sc0404`); keys are per-user state.
import { expect, test } from '@playwright/test';

const ANTHROPIC_KEY = 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa';

test('AC4.2: 폐기한 뒤에는 신규 호출이 차단된다 — 사유 메시지와 함께', async ({ page }) => {
  await page.goto('/api/auth/login?as=sc0404');

  await page.getByTestId('connect-app').click();
  await expect(page.getByTestId('connection')).toBeVisible();

  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill(ANTHROPIC_KEY);
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toBeVisible();
  await expect(page.getByTestId('register-key')).toBeEnabled();

  await page.getByTestId('remove-key').click();
  await expect(page.getByTestId('key-input')).toBeVisible();

  const blocked = await page.request.get('/api/llm-keys/preflight');
  expect(blocked.ok()).toBe(false);
  const body = await blocked.text();
  expect(body, `거부 사유가 응답에 실려야 한다: ${body}`).toContain('폐기');

  // 같은 버튼이 등록과 진행을 겸하므로(sc04-03), 키도 입력도 없을 때만 닫힌다.
  await expect(page.getByTestId('register-key')).toBeDisabled();
});
