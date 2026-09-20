// 검증 시나리오: 04-platform.md#시나리오 13
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_LLM_KEY=stub), where key validation
// is a deterministic shape check instead of a provider round-trip. Signs in as its own
// stub identity (`?as=sc0413`); keys are per-user state.
import { expect, test } from '@playwright/test';

// Shape-valid for the stub validator on purpose: the refusal below has to come from
// the supported-provider scope, not from key validation.
const ANTHROPIC_KEY = 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa';
const GOOGLE_KEY = 'AIzaSyCcccccccccccccccccccc';

test('AC4.2: 미지원 제공자의 키는 등록 시점에 거부되고, 등록돼 있던 키의 자리를 빼앗지 않는다', async ({
  page,
}) => {
  await page.goto('/api/auth/login?as=sc0413');

  await page.getByTestId('connect-app').click();
  await expect(page.getByTestId('connection')).toBeVisible();

  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill(ANTHROPIC_KEY);
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toBeVisible();

  // 활성 키 표시는 고른 제공자의 것만 보여준다 — 아래의 0 은 아직 키가 없다는 뜻이지
  // 방금 등록한 Anthropic 키가 사라졌다는 뜻이 아니다.
  await page.getByTestId('provider-google').click();
  await expect(page.getByTestId('active-key')).toHaveCount(0);

  await page.getByTestId('key-input').fill(GOOGLE_KEY);
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('key-error')).toContainText('분석 호출을 지원하지 않아');

  await expect(page.getByTestId('active-key')).toHaveCount(0);

  const stillAnthropic = await page.request.get('/api/llm-keys/preflight');
  expect(stillAnthropic.ok()).toBe(true);
  expect((await stillAnthropic.json()).provider).toBe('anthropic');
  await page.getByTestId('provider-anthropic').click();
  await expect(page.getByTestId('active-key')).toBeVisible();
});
