import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 30_000,
  expect: { timeout: 10_000 },
  fullyParallel: false,
  // One worker, i.e. one spec *file* at a time. `fullyParallel: false` alone only
  // serialises within a file; files still run concurrently. ac4-5 scales the
  // analysis worker Deployment, which is state no per-user handle can isolate, so
  // overlapping files would see each other's queue drain. The suite is seconds
  // long — determinism is worth more than the concurrency here.
  workers: 1,
  reporter: [['list']],
  use: {
    baseURL: process.env.BASE_URL ?? 'http://localhost:8080',
    trace: 'retain-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
