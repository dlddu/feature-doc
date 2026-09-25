import { expect, type APIRequestContext } from '@playwright/test';

/** Gives the signed-in user the App installation by walking the URL the product
 *  itself hands out. The stub install flow issues one id per user and the Setup
 *  callback now refuses any other, so the id cannot be chosen by the caller. */
export async function installApp(request: APIRequestContext): Promise<void> {
  const offered = await request.get('/api/github/install-url');
  expect(offered.ok(), 'install URL').toBeTruthy();
  const { url } = (await offered.json()) as { url: string };
  // The Setup URL callback is a GET that redirects back into the SPA.
  expect((await request.get(url)).ok(), 'App Setup URL callback').toBeTruthy();
}
