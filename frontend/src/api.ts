// Thin typed client for the FeatureDoc backend. Cookies (the session) ride
// along automatically as same-origin requests.

export type Me = { id: string; login: string };

export type Connection =
  | { installed: false; permissions: string[] }
  | {
      installed: true;
      installation_id: number;
      account: string;
      repo_count: number;
      permissions: string[];
    };

export type LlmKey = {
  id: string;
  provider: string;
  fingerprint: string;
  masked: string;
  status: string;
  created_at: number;
};

export type Provider = 'anthropic' | 'openai' | 'google';

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function json<T>(res: Response): Promise<T> {
  const text = await res.text();
  const data = text ? JSON.parse(text) : null;
  if (!res.ok) {
    const message =
      data && typeof data.error === 'string' ? data.error : `HTTP ${res.status}`;
    throw new ApiError(res.status, message);
  }
  return data as T;
}

export async function getMe(): Promise<Me | null> {
  const res = await fetch('/api/me');
  if (res.status === 401) return null;
  return json<Me>(res);
}

export async function getConnection(): Promise<Connection> {
  return json<Connection>(await fetch('/api/github/connection'));
}

export async function getInstallUrl(): Promise<{ url: string; permissions: string[] }> {
  return json<{ url: string; permissions: string[] }>(await fetch('/api/github/install-url'));
}

export async function listKeys(): Promise<LlmKey[]> {
  return json<LlmKey[]>(await fetch('/api/llm-keys'));
}

export async function registerKey(provider: Provider, key: string): Promise<LlmKey> {
  return json<LlmKey>(
    await fetch('/api/llm-keys', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ provider, key }),
    }),
  );
}

export async function revokeKey(id: string): Promise<unknown> {
  return json<unknown>(await fetch(`/api/llm-keys/${id}`, { method: 'DELETE' }));
}

// Full-page navigations: the OAuth and install flows are browser redirects.
export const loginHref = '/api/auth/login';
