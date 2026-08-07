// 검증 AC: AC4.1
//
// AC4.1 (GitHub App 설치를 통한 저장소 접근 연결) 전용 spec.
//
// Runs against the stub-mode deployment (FEATUREDOC_MODE=stub): the App install
// round-trip bounces through our own setup callback instead of GitHub, and the
// installation resolves to the deterministic `stub-account` with three repositories.
// Signs in as its own stub identity (`?as=ac41`) so the installation it creates
// cannot disturb — or be disturbed by — the specs running in sibling workers.
//
// 자동화 밖 잔여: 실제 GitHub에서의 설치·저장소 범위 변경·설치 해제 라운드트립은
// 외부 계정 상태에 의존해 stub e2e로 도달할 수 없다(doc-tracker "e2e 매핑" 참조).
import { expect, test } from '@playwright/test';

test('AC4.1: 설치 전 최소 권한 안내 → App 연결 → 설치 상태와 접근 가능한 저장소 수', async ({
  page,
}) => {
  await page.goto('/api/auth/login?as=ac41');

  // 설치 전: 요청할 읽기 전용 최소 권한을 먼저 보여준다(여정 F1).
  const requested = page.getByTestId('requested-permissions');
  await expect(requested).toBeVisible();
  await expect(requested).toContainText('contents:read');
  await expect(requested).toContainText('metadata:read');

  // 아직 설치되지 않았으므로 설치 상태 카드는 없다.
  await expect(page.getByTestId('connection')).toHaveCount(0);

  // App 연결(스텁 setup 왕복) → 설치 상태 + 계정 + 접근 가능한 저장소 수.
  await page.getByTestId('connect-app').click();
  const connection = page.getByTestId('connection');
  await expect(connection).toBeVisible();
  await expect(connection).toContainText('stub-account');
  await expect(connection).toContainText('3 repositories');

  // 설치 후에도 부여된 권한 범위는 계속 보인다.
  await expect(page.getByText('contents:read')).toBeVisible();
  await expect(page.getByText('metadata:read')).toBeVisible();
});
