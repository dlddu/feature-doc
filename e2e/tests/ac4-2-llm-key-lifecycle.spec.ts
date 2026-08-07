// 검증 AC: AC4.2
//
// AC4.2 (사용자 LLM API Key 등록·교체·폐기 및 호출 위임) 전용 spec.
//
// Runs against the stub-mode deployment (FEATUREDOC_MODE=stub), where key validation
// is a deterministic shape check instead of a provider round-trip. Signs in as its own
// stub identity (`?as=ac42`); keys are per-user state.
//
// 자동화 밖 잔여: 실제 제공자에 대한 키 검증·호출 위임과 누적 호출/실측 비용 표시는
// 외부 LLM 호출과 분석 파이프라인(미구현)에 의존한다(doc-tracker "e2e 매핑" 참조).
import { expect, test } from '@playwright/test';

const ANTHROPIC_KEY = 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa';
const OPENAI_KEY = 'sk-proj-bbbbbbbbbbbbbbbbbbbbbb';

test('AC4.2: 잘못된 키 거부 → 등록 → 교체 → 폐기 후 신규 호출 차단', async ({ page }) => {
  await page.goto('/api/auth/login?as=ac42');

  // 키가 실제로 쓰일 수 있는지 확인하려면 App 연결이 선행돼야 한다(AC4.1의 화면을 경유만 한다).
  await page.getByTestId('connect-app').click();
  await expect(page.getByTestId('connection')).toBeVisible();

  // 검증에 실패하는 키는 등록되지 않고, 이유가 화면에 보인다.
  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill('bad');
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('key-error')).toBeVisible();
  await expect(page.getByTestId('active-key')).toHaveCount(0);

  // 유효한 키는 등록되고, 자격증명이 준비됐음을 pre-flight가 확인해 준다.
  await page.getByTestId('key-input').fill(ANTHROPIC_KEY);
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toBeVisible();

  const cont = page.getByTestId('continue');
  await expect(cont).toBeEnabled();
  await cont.click();
  await expect(page.getByTestId('ready')).toBeVisible();

  // 교체: 다른 제공자의 키를 등록하면 그 제공자의 활성 키가 된다.
  await page.getByTestId('provider-openai').click();
  await expect(page.getByTestId('active-key')).toHaveCount(0);
  await page.getByTestId('key-input').fill(OPENAI_KEY);
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toContainText('openai');

  // 폐기: 등록된 키를 모두 회수하면 신규 LLM 호출이 차단된다.
  await page.getByTestId('remove-key').click();
  await expect(page.getByTestId('key-input')).toBeVisible();
  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('remove-key').click();
  await expect(page.getByTestId('key-input')).toBeVisible();

  const blocked = await page.request.get('/api/llm-keys/preflight');
  expect(blocked.ok()).toBe(false);
  await expect(page.getByTestId('continue')).toBeDisabled();
});
