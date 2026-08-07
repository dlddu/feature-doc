// 검증 AC: AC4.3
//
// AC4.3 (자격증명의 안전한 사용 정책) 전용 spec — 사용자에게 도달하는 절반:
// 등록한 키의 평문은 화면·API 어디에도 다시 나타나지 않고(식별자만), 자격증명을
// 다루는 경로는 사용자가 조회할 수 있는 감사 이력으로 남는다.
//
// Runs against the stub-mode deployment (FEATUREDOC_MODE=stub) and signs in as its own
// stub identity (`?as=ac43`) so the sentinel key it registers belongs to no other spec.
//
// 자동화 밖 잔여: 운영 로그·오류 메시지의 평문 노출 점검은 클러스터 로그 수집에
// 의존한다. 응답 본문 수준의 평문 미노출은 `e2e/smoke.sh`가 별도로 지킨다
// (doc-tracker "e2e 매핑" 참조).
import { expect, test } from '@playwright/test';

// 이 spec에서만 쓰는 유일한 평문. 어디에도 다시 나타나면 안 된다.
const SENTINEL = 'sk-ant-api03-PLAINTEXTSENTINEL0002';

test('AC4.3: 등록한 키는 식별자로만 표시되고 평문은 재노출되지 않으며, 사용 이력은 조회 가능하다', async ({
  page,
}) => {
  await page.goto('/api/auth/login?as=ac43');

  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill(SENTINEL);
  await page.getByTestId('register-key').click();

  // 화면에는 제공자 접두사만 남은 마스킹 식별자가 보인다.
  const active = page.getByTestId('active-key');
  await expect(active).toBeVisible();
  await expect(active).toContainText('sk-ant-');
  await expect(active).not.toContainText('PLAINTEXTSENTINEL0002');

  // 렌더된 문서 전체(입력값이 지워진 뒤)에도 평문은 남지 않는다.
  await page.reload();
  await expect(page.getByTestId('active-key')).toBeVisible();
  expect(await page.content()).not.toContain('PLAINTEXTSENTINEL0002');

  // 자격증명을 노출할 수 있는 응답 어디에도 평문이 없다.
  for (const endpoint of ['/api/llm-keys', '/api/me', '/api/audit']) {
    const body = await (await page.request.get(endpoint)).text();
    expect(body).not.toContain('PLAINTEXTSENTINEL0002');
  }

  // 자격증명을 다룬 경로는 사용자가 볼 수 있는 감사 이력으로 남는다.
  const audit = (await (await page.request.get('/api/audit')).json()) as { action: string }[];
  expect(audit.map((entry) => entry.action)).toContain('llm_key.register');
});
