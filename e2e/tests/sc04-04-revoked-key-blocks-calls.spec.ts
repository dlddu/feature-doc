// 검증 시나리오: 04-platform.md#시나리오 4
//
// 「키 폐기 후 신규 호출 차단」 전용 spec (AC4.2).
//
// docs/test/04-platform.md 시나리오 4를 그대로 따라간다:
//   키 등록 상태에서 키를 폐기한 뒤 새 분석 트리거를 시도하면, 신규 LLM 호출이
//   차단되고 「키가 없거나 폐기되었습니다」 메시지가 표시된다.
//
// 이 단정은 `sc04-03-llm-key-registration.spec.ts`와 한 파일에 있었다(선언은
// 시나리오 3 하나) — 규칙 2 상 분리 대상으로 등재됐다가 이 파일로 옮겨왔다.
// 분할 경계 원칙에 따라 옮기면서 하나만 보강했다: 제품의 거부 메시지는
// `backend/src/llmkey.rs`가 시나리오 4(test#4)로 귀속해 둔 응답이므로, 응답 본문에서
// 그 메시지를 실측한다(기존 단정은 실패 여부와 트리거 차단만 봤다).
//
// Runs against the stub-mode deployment (FEATUREDOC_MODE=stub). Signs in as its own
// stub identity (`?as=sc0404`); keys are per-user state.
//
// 자동화 밖 잔여: 실제 제공자에 대한 호출 차단은 stub 경로 밖이다(doc-tracker "e2e 매핑").
import { expect, test } from '@playwright/test';

const ANTHROPIC_KEY = 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa';

test('AC4.2: 폐기한 뒤에는 신규 호출이 차단된다 — 사유 메시지와 함께', async ({ page }) => {
  await page.goto('/api/auth/login?as=sc0404');

  // 키가 실제로 쓰일 수 있는지 확인하려면 App 연결이 선행돼야 한다(sc04-01의 화면을 경유만 한다).
  await page.getByTestId('connect-app').click();
  await expect(page.getByTestId('connection')).toBeVisible();

  // 키 등록 상태를 만든다.
  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill(ANTHROPIC_KEY);
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toBeVisible();
  await expect(page.getByTestId('continue')).toBeEnabled();

  // 키를 폐기한다.
  await page.getByTestId('remove-key').click();
  await expect(page.getByTestId('key-input')).toBeVisible();

  // 신규 LLM 호출은 차단된다 — 사유 메시지(제품이 시나리오 4로 귀속해 둔 응답)가
  // 응답 본문에 실린다.
  const blocked = await page.request.get('/api/llm-keys/preflight');
  expect(blocked.ok()).toBe(false);
  const body = await blocked.text();
  expect(body, `거부 사유가 응답에 실려야 한다: ${body}`).toContain('폐기');

  // 그리고 화면의 새 분석 트리거도 차단된다.
  await expect(page.getByTestId('continue')).toBeDisabled();
});
