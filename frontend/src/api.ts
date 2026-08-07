// Typed client for the FeatureDoc backend. Cookies (the session) ride along on
// same-origin requests; the SPA and API share an origin in every deployment.

export type User = {
  id: string;
  login: string;
  name: string | null;
  avatarUrl: string | null;
};

export type Account = {
  login: string | null;
  accountType: string | null;
};

export type Connection = {
  installed: boolean;
  account: Account | null;
  repositorySelection: string | null;
  repositoryCount: number | null;
  permissions: string[];
};

export type LlmKey = {
  id: string;
  provider: string;
  fingerprint: string;
  masked: string;
  status: string;
  createdAt: number;
};

export type ProviderId = 'anthropic' | 'openai' | 'google';

/** Where the "Sign in with GitHub" button navigates (full-page, to follow redirects). */
export const LOGIN_URL = '/api/auth/login';

const json = { 'content-type': 'application/json' };

async function errorMessage(res: Response): Promise<string> {
  try {
    const body = (await res.json()) as { error?: string };
    if (body.error) return body.error;
  } catch {
    /* fall through */
  }
  return `요청에 실패했어요 (${res.status})`;
}

/** Current user, or null when unauthenticated (401). */
export async function getMe(): Promise<User | null> {
  const res = await fetch('/api/me', { credentials: 'same-origin' });
  if (res.status === 401) return null;
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as User;
}

export async function getConnection(): Promise<Connection> {
  const res = await fetch('/api/github/connection', { credentials: 'same-origin' });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as Connection;
}

export async function getInstallUrl(): Promise<string> {
  const res = await fetch('/api/github/install-url', { credentials: 'same-origin' });
  if (!res.ok) throw new Error(await errorMessage(res));
  return ((await res.json()) as { url: string }).url;
}

export async function listKeys(): Promise<LlmKey[]> {
  const res = await fetch('/api/llm-keys', { credentials: 'same-origin' });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as LlmKey[];
}

export async function registerKey(provider: ProviderId, key: string): Promise<LlmKey> {
  const res = await fetch('/api/llm-keys', {
    method: 'POST',
    credentials: 'same-origin',
    headers: json,
    body: JSON.stringify({ provider, key }),
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as LlmKey;
}

export async function deleteKey(id: string): Promise<void> {
  const res = await fetch(`/api/llm-keys/${encodeURIComponent(id)}`, {
    method: 'DELETE',
    credentials: 'same-origin',
  });
  if (!res.ok && res.status !== 204) throw new Error(await errorMessage(res));
}

/** Confirms a usable key exists before continuing; throws the block message if not. */
export async function preflight(): Promise<{ provider: string; fingerprint: string }> {
  const res = await fetch('/api/llm-keys/preflight', { credentials: 'same-origin' });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as { provider: string; fingerprint: string };
}

// ── analyses (AC1.1) ─────────────────────────────────────────────────────────

/** A repository the installation can access — a candidate to analyze (S02 · S03). */
export type Repository = {
  owner: string;
  name: string;
  fullName: string;
  defaultBranch: string;
};

/** An analysis job as the home list shows it (S02). */
export type Analysis = {
  id: string;
  repoOwner: string;
  repoName: string;
  branch: string;
  status: string;
  estLlmCalls: number;
  estCostCents: number;
  createdAt: number;
};

/**
 * Pre-flight estimate for a typed target (S03). `hasAccess: false` is not an error —
 * the screen renders the "add this repo to the App" recovery path instead.
 */
export type Preflight = {
  hasAccess: boolean;
  owner: string;
  name: string;
  fullName: string;
  branch: string;
  filesToScan: number;
  sizeBytes: number;
  estLlmCalls: number;
  estCostCents: number;
  estDurationMin: number;
};

/** Repositories the App can reach. Empty when the App is not installed yet. */
export async function listRepositories(): Promise<Repository[]> {
  const res = await fetch('/api/repositories', { credentials: 'same-origin' });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as Repository[];
}

/** The user's analysis jobs, newest first. */
export async function listAnalyses(): Promise<Analysis[]> {
  const res = await fetch('/api/analyses', { credentials: 'same-origin' });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as Analysis[];
}

/** Resolves a typed target and estimates the analysis scale before triggering. */
export async function preflightAnalysis(repoUrl: string, branch: string): Promise<Preflight> {
  const res = await fetch('/api/analyses/preflight', {
    method: 'POST',
    credentials: 'same-origin',
    headers: json,
    body: JSON.stringify({ repoUrl, branch: branch.trim() || null }),
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as Preflight;
}

/** Explicitly enqueues an analysis. Rejects out-of-scope targets without queueing. */
export async function createAnalysis(repoUrl: string, branch: string): Promise<Analysis> {
  const res = await fetch('/api/analyses', {
    method: 'POST',
    credentials: 'same-origin',
    headers: json,
    body: JSON.stringify({ repoUrl, branch: branch.trim() || null }),
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as Analysis;
}
