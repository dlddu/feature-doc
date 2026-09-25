import { expect, type APIRequestContext } from '@playwright/test';

/** Gives the signed-in user the App installation by walking the URL the product itself hands out. */
export async function installApp(request: APIRequestContext): Promise<void> {
  const offered = await request.get('/api/github/install-url');
  expect(offered.ok(), 'install URL').toBeTruthy();
  const { url } = (await offered.json()) as { url: string };
  expect((await request.get(url)).ok(), 'App Setup URL callback').toBeTruthy();
}
