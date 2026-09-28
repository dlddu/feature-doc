// 검증 시나리오: 01-analysis-pipeline.md#시나리오 9
//
// Leases one API env var — lease rules in `e2e/support/cluster.ts`.
import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import { setApiEnv } from '../support/cluster';
import { installApp } from '../support/github-app';

const KEY = 'sk-ant-api03-aaaaaaaaaaaaaaaaaaaa';
const PUBLIC = 'github.com/stub-public/oss-lib';
const PRIVATE = 'github.com/someone-else/private-repo';
const REVOKED = 'FEATUREDOC_STUB_USER_AUTH_REVOKED';
const HANDLE = 'sc0109a';
const UNINSTALLED = 'sc0109b';

async function signIn(page: Page, handle: string, install: boolean): Promise<void> {
  await page.goto(`/api/auth/login?as=${handle}`);
  if (install) await installApp(page.request);
  const created = await page.request.post('/api/llm-keys', {
    data: { provider: 'anthropic', key: KEY },
  });
  expect(created.status(), `${handle} 의 키 등록`).toBe(201);
}

async function enterHome(page: Page): Promise<void> {
  await page.goto('/');
  const enter = page.getByTestId('register-key');
  await expect(enter).toBeEnabled();
  await enter.click();
  await expect(page.getByTestId('repo-card').first()).toBeVisible();
}

async function check(page: Page, repo: string): Promise<void> {
  await page.getByTestId('repo-url').fill(repo);
  await page.getByTestId('check-access').click();
}

async function analysisCount(request: APIRequestContext): Promise<number> {
  const res = await request.get('/api/analyses');
  expect(res.ok()).toBeTruthy();
  return ((await res.json()) as unknown[]).length;
}

function leaksNoCredential(text: string): void {
  expect(text).not.toMatch(/gh[opsu]_[A-Za-z0-9]/);
}

test.describe('AC1.6: 설치 범위 밖 공개 저장소 분석', () => {
  test.describe.configure({ mode: 'serial', timeout: 300_000 });

  test('App 을 설치한 사용자는 설치 범위 밖 공개 저장소를 로그인 인가로 읽어 분석을 건다', async ({
    page,
  }) => {
    await signIn(page, HANDLE, true);
    await enterHome(page);

    await check(page, PUBLIC);
    const notice = page.getByTestId('repo-public');
    await expect(notice).toBeVisible();
    await expect(notice).toContainText('로그인할 때 맡긴 GitHub 인가로 읽기만 합니다');
    await expect(page.getByTestId('repo-outside')).toHaveCount(0);
    const estimate = page.getByTestId('estimate');
    await expect(estimate).toBeVisible();
    await expect(estimate).toContainText('stub-public/oss-lib');
    await expect(estimate).toContainText('Est. LLM Cost');

    await page.getByTestId('start-analysis').click();
    const card = page.getByTestId('repo-card').filter({ hasText: 'stub-public/oss-lib' });
    await expect(card).toContainText('Queued');
    await expect(card).not.toContainText('App 설치 범위 밖');
    expect(await analysisCount(page.request)).toBe(1);

    await enterHome(page);
    await check(page, PRIVATE);
    await expect(page.getByTestId('repo-outside')).toBeVisible();
    await expect(page.getByTestId('repo-public')).toHaveCount(0);
    await expect(page.getByTestId('start-analysis')).toHaveCount(0);
    const refused = await page.request.post('/api/analyses', { data: { repoUrl: PRIVATE } });
    expect(refused.status()).toBe(400);
    const refusedText = await refused.text();
    expect(refusedText).toContain('이 저장소에 접근할 수 없습니다');
    leaksNoCredential(refusedText);
    expect(await analysisCount(page.request)).toBe(1);

    try {
      await setApiEnv(REVOKED, HANDLE);
      await enterHome(page);
      await check(page, PUBLIC);
      const expired = page.getByTestId('repo-authexpired');
      await expect(expired).toBeVisible();
      await expect(expired).toContainText('다시 로그인하면 공개 저장소를 이어서 분석할 수 있습니다');
      await expect(page.getByTestId('relogin')).toBeVisible();
      await expect(page.getByTestId('repo-outside')).toHaveCount(0);
      await expect(page.getByTestId('start-analysis')).toHaveCount(0);

      const blocked = await page.request.post('/api/analyses', { data: { repoUrl: PUBLIC } });
      expect(blocked.status()).toBe(400);
      const blockedText = await blocked.text();
      expect(blockedText).toContain('큐에 등록하지 않았습니다');
      leaksNoCredential(blockedText);
      expect(await analysisCount(page.request)).toBe(1);
    } finally {
      await setApiEnv(REVOKED, null);
    }

    await signIn(page, UNINSTALLED, false);
    const uninstalled = await page.request.post('/api/analyses', { data: { repoUrl: PUBLIC } });
    expect(uninstalled.status()).toBe(400);
    const uninstalledText = await uninstalled.text();
    expect(uninstalledText).toContain('GitHub App이 아직 설치되지 않았습니다');
    leaksNoCredential(uninstalledText);
    expect(await analysisCount(page.request)).toBe(0);
  });
});
