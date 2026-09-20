// 검증 시나리오: 04-platform.md#시나리오 3
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_LLM_KEY=stub), where key validation
// is a deterministic shape check instead of a provider round-trip. Signs in as its own
// stub identity (`?as=ac42`); keys are per-user state.
import { expect, test } from '@playwright/test';

const ANTHROPIC_KEY = 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa';
const OPENAI_KEY = 'sk-proj-bbbbbbbbbbbbbbbbbbbbbb';

test('AC4.2: 잘못된 키 거부 → 등록 → 교체 → 등록된 키가 분석 호출의 위임 대상이 된다', async ({
  page,
}) => {
  await page.goto('/api/auth/login?as=ac42');

  // 키 등록 화면은 설치 뒤에 서고, 설치가 서면 권한 부여 화면이 스스로 넘긴다 — 그래서
  // 여기에 키 등록 화면으로 가는 조작이 없다.
  await page.getByTestId('connect-app').click();
  await expect(page.getByTestId('connection')).toBeVisible();

  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill('bad');
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('key-error')).toBeVisible();
  await expect(page.getByTestId('active-key')).toHaveCount(0);

  await page.getByTestId('key-input').fill(ANTHROPIC_KEY);
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toBeVisible();

  await page.getByTestId('provider-openai').click();
  await expect(page.getByTestId('active-key')).toHaveCount(0);
  await page.getByTestId('key-input').fill(OPENAI_KEY);
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toContainText('openai');

  // 같은 버튼이 등록과 진행을 겸한다: 입력이 빈 채로 누르면 등록 대신 pre-flight 를 거쳐
  // Home 으로 넘어간다. 아래 두 줄은 그래서 세 번째 등록이 아니라 분석 트리거다.
  const save = page.getByTestId('register-key');
  await expect(save).toBeEnabled();
  await save.click();
  await expect(page.getByTestId('repo-card').first()).toBeVisible();

  const delegated = await page.request.get('/api/llm-keys/preflight');
  expect(delegated.ok()).toBe(true);
  expect((await delegated.json()).provider).toBe('openai');
});
