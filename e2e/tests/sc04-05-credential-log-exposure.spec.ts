// 검증 시나리오: 04-platform.md#시나리오 5
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_GITHUB_AUTH=stub) and signs in as its own
// stub identity (`?as=ac43`) so the sentinel key it registers belongs to no other spec.
import { expect, test } from '@playwright/test';

// 이 평문은 레포 어디에도 두 번 나타나지 않아야 관측이 성립한다 — 다른 spec 과 값을
// 공유하면 여기서 잡은 부재가 저 spec 의 부재로 읽힌다.
const SENTINEL = 'sk-ant-api03-PLAINTEXTSENTINEL0002';

test('AC4.3: 등록한 키는 식별자로만 표시되고 평문은 재노출되지 않으며, 사용 이력은 조회 가능하다', async ({
  page,
}) => {
  await page.goto('/api/auth/login?as=ac43');

  await page.getByTestId('connect-app').click();
  await expect(page.getByTestId('connection')).toBeVisible();

  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill(SENTINEL);
  await page.getByTestId('register-key').click();

  const active = page.getByTestId('active-key');
  await expect(active).toBeVisible();
  await expect(active).toContainText('sk-ant-');
  await expect(active).not.toContainText('PLAINTEXTSENTINEL0002');

  // reload 해야 입력 필드가 아직 들고 있는 평문이 아니라 서버가 되돌려 준 문서를 본다.
  await page.reload();
  await expect(page.getByTestId('active-key')).toBeVisible();
  expect(await page.content()).not.toContain('PLAINTEXTSENTINEL0002');

  for (const endpoint of ['/api/llm-keys', '/api/me', '/api/audit']) {
    const body = await (await page.request.get(endpoint)).text();
    expect(body).not.toContain('PLAINTEXTSENTINEL0002');
  }

  const audit = (await (await page.request.get('/api/audit')).json()) as { action: string }[];
  expect(audit.map((entry) => entry.action)).toContain('llm_key.register');
});
