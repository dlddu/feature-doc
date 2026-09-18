// 검증 시나리오: 04-platform.md#시나리오 13
//
// 「미지원 제공자의 키 등록 거부」 전용 spec (AC4.2).
//
// docs/test/04-platform.md 시나리오 13을 그대로 따라간다:
//   등록 화면에서 아직 호출이 구현되지 않은 제공자(현재: Google)를 선택해 형식이
//   올바른 키로 등록을 시도하면 거부되고(HTTP 400) 사유와 대안이 표시된다; 키는
//   저장되지 않고, 이미 등록돼 있던 지원 제공자의 키가 계속 「이 분석이 쓸 키」로
//   남는다. 지원 여부의 판정은 실제 호출 구현에서 파생되므로, 호출이 없는 제공자가
//   등록 경로로 들어올 수 없다.
//
// 이 단정은 `sc04-03-llm-key-registration.spec.ts`와 한 파일에 있었다(선언은
// 시나리오 3 하나) — 규칙 2 상 분리 대상으로 등재됐다가 이 파일로 옮겨왔다.
//
// 시나리오 3·4와 같은 AC를 보지만 뒤에 붙은 시나리오다(문서 자신의 주석 참조 —
// 기존 번호가 다른 AC의 자동화·문서에 인용돼 재배치하지 않기로 했다).
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_LLM_KEY=stub), where key validation
// is a deterministic shape check instead of a provider round-trip. Signs in as its own
// stub identity (`?as=sc0413`); keys are per-user state.
//
// 자동화 밖 잔여: 실제 제공자 측 상태와의 왕복은 stub 경로 밖이다(doc-tracker "e2e 매핑").
import { expect, test } from '@playwright/test';

// Shape-valid for the stub validator on purpose: the refusal below has to come from
// the supported-provider scope, not from key validation.
const ANTHROPIC_KEY = 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa';
const GOOGLE_KEY = 'AIzaSyCcccccccccccccccccccc';

test('AC4.2: 미지원 제공자의 키는 등록 시점에 거부되고, 등록돼 있던 키의 자리를 빼앗지 않는다', async ({
  page,
}) => {
  await page.goto('/api/auth/login?as=sc0413');

  // 키가 실제로 쓰일 수 있는지 확인하려면 App 연결이 선행돼야 한다(sc04-01의 화면을 경유만 한다).
  await page.getByTestId('connect-app').click();
  await expect(page.getByTestId('connection')).toBeVisible();

  // 비교 기준: 지원 제공자의 키가 먼저 등록돼 있다.
  await page.getByTestId('provider-anthropic').click();
  await page.getByTestId('key-input').fill(ANTHROPIC_KEY);
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('active-key')).toBeVisible();

  // 미지원 제공자를 선택한다 — 지원 제공자의 활성 키는 이 화면에서 이 제공자 이름으로
  // 표시되므로, 선택 전환 시 활성 키 표시가 바뀐다(아직 이 제공자에는 키가 없다).
  await page.getByTestId('provider-google').click();
  await expect(page.getByTestId('active-key')).toHaveCount(0);

  // 형식이 올바른 키로 등록을 시도한다 — 거부는 키 검증이 아니라 지원 범위에서 나온다.
  await page.getByTestId('key-input').fill(GOOGLE_KEY);
  await page.getByTestId('register-key').click();
  await expect(page.getByTestId('key-error')).toContainText('분석 호출을 지원하지 않아');

  // 거부됐으므로 저장되지 않는다 — 이 제공자에 활성 키가 생기지 않는다.
  await expect(page.getByTestId('active-key')).toHaveCount(0);

  // 그리고 이미 쓰던 키의 자리를 빼앗지 않는다: 분석이 쓸 키는 그대로 Anthropic이다.
  const stillAnthropic = await page.request.get('/api/llm-keys/preflight');
  expect(stillAnthropic.ok()).toBe(true);
  expect((await stillAnthropic.json()).provider).toBe('anthropic');
  await page.getByTestId('provider-anthropic').click();
  await expect(page.getByTestId('active-key')).toBeVisible();
});
