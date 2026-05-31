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

/** A connected repository (S02). Analysis-derived fields are reserved and stay
 * `null` until the analysis pipeline produces them. */
export type Repository = {
  id: string;
  owner: string;
  name: string;
  branch: string;
  status: string;
  featureCount: number | null;
  conflictCount: number | null;
  spendCents: number | null;
  progress: number | null;
  step: string | null;
  lastAnalyzedAt: number | null;
  createdAt: number;
};

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

export async function listRepositories(): Promise<Repository[]> {
  const res = await fetch('/api/repositories', { credentials: 'same-origin' });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as Repository[];
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
