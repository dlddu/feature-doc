// 검증 시나리오: 04-platform.md#시나리오 1
//
// Runs against the e2e deployment (FEATUREDOC_DOUBLE_GITHUB_APP=stub), where the
// installation resolves to the deterministic `stub-account` with three repositories.
// Signs in as its own stub identity (`?as=ac41`) so the installation it creates
// cannot disturb — or be disturbed by — the specs running in sibling workers.
import { expect, test } from '@playwright/test';

test('AC4.1: 설치 전 최소 권한 안내 → App 연결 → 설치 상태와 접근 가능한 저장소 수', async ({
  page,
}) => {
  await page.goto('/api/auth/login?as=ac41');

  const requested = page.getByTestId('requested-permissions');
  await expect(requested).toBeVisible();
  await expect(requested).toContainText('contents:read');
  await expect(requested).toContainText('metadata:read');

  await expect(page.getByTestId('connection')).toHaveCount(0);

  await page.getByTestId('connect-app').click();
  const connection = page.getByTestId('connection');
  await expect(connection).toBeVisible();
  await expect(connection).toContainText('stub-account');
  await expect(connection).toContainText('3 repositories');

  await expect(page.getByText('contents:read')).toBeVisible();
  await expect(page.getByText('metadata:read')).toBeVisible();
});
