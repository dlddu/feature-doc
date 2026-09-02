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
  /** Pipeline progress, so a card can read "step 1 of 5" without a second fetch. */
  stagesDone: number;
  stagesTotal: number;
};

/** One pipeline step of an analysis (S04). `pending` until a worker runs it. */
export type Stage = {
  seq: number;
  key: string;
  title: string;
  status: 'pending' | 'running' | 'succeeded' | 'failed';
  /** What the stage measured, e.g. `766 files · 2.2 MB`. */
  detail: string | null;
  error: string | null;
  startedAt: number | null;
  finishedAt: number | null;
};

/** Everything S04 draws — all of it persisted server-side (AC1.5). */
export type AnalysisDetail = Analysis & {
  error: string | null;
  startedAt: number | null;
  finishedAt: number | null;
  stages: Stage[];
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

/** One analysis with its pipeline stages — the S04 read (AC1.5). */
export async function getAnalysis(id: string): Promise<AnalysisDetail> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}`, {
    credentials: 'same-origin',
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as AnalysisDetail;
}

/**
 * Re-runs one failed stage and nothing else (AC1.5). The job returns to the queue,
 * so the answer already carries the reset progress the screen should render.
 */
export async function retryStage(id: string, stageKey: string): Promise<AnalysisDetail> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/stages/${encodeURIComponent(stageKey)}/retry`,
    { method: 'POST', credentials: 'same-origin' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as AnalysisDetail;
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

// ── pipeline documents (AC1.2) ───────────────────────────────────────────────

/** One extracted concern, with the file paths it was inferred from. */
export type CrossCuttingItem = {
  name: string;
  /** Repository paths that support this item — AC1.2's "근거가 된 파일 경로". */
  evidence: string[];
};

/** One of AC1.2's five axes and what was found on it. */
export type CrossCuttingCategory = {
  axis: string;
  items: CrossCuttingItem[];
};

/**
 * Whether this document reproduced the previous analysis of the same target.
 * `first` — nothing earlier to compare against.
 */
export type Reproducibility = {
  verdict: 'first' | 'unchanged' | 'changed';
  comparedTo: string | null;
};

/** Everything S05 draws (AC1.2). */
export type CrossCuttingDocument = {
  kind: string;
  content: { categories: CrossCuttingCategory[] };
  model: string;
  createdAt: number;
  reproducibility: Reproducibility;
};

/**
 * The cross-cutting concerns document one analysis produced.
 *
 * A 404 means the stage has not produced it yet — that is a distinct state from
 * "ran and found nothing", so it is surfaced rather than flattened to an empty
 * document.
 */
export async function getCrossCutting(id: string): Promise<CrossCuttingDocument> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/documents/cross-cutting`, {
    credentials: 'same-origin',
  });
  if (res.status === 404) throw new Error('아직 생성되지 않았어요');
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as CrossCuttingDocument;
}

// ── discovery strategy (AC1.3) ───────────────────────────────────────────────

/** One place the next stage will look. `user` entries are the reviewer's own. */
export type StrategyEntry = {
  pattern: string;
  source: 'generated' | 'user';
};

/** The reviewable strategy for one analysis — draft until `approved`. */
export type DiscoveryStrategy = {
  entries: StrategyEntry[];
  approved: boolean;
  approvedAt: number | null;
  updatedAt: number;
};

/**
 * The strategy stage 3 proposed, as the user has it so far.
 *
 * A 404 means stage 3 has not produced a proposal yet — a different state from
 * "proposed nothing", so it is surfaced rather than flattened to an empty list.
 */
export async function getDiscoveryStrategy(id: string): Promise<DiscoveryStrategy> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/discovery-strategy`, {
    credentials: 'same-origin',
  });
  if (res.status === 404) throw new Error('아직 생성되지 않았어요');
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as DiscoveryStrategy;
}

/** Replaces the list — deleting and adding are both this call (AC1.3 "수정"). */
export async function putDiscoveryStrategy(
  id: string,
  patterns: string[],
): Promise<DiscoveryStrategy> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/discovery-strategy/entries`,
    {
      method: 'PUT',
      credentials: 'same-origin',
      headers: json,
      body: JSON.stringify({ patterns }),
    },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as DiscoveryStrategy;
}

/** Approves it. Only an approved strategy becomes the next stage's input (AC1.3). */
export async function approveDiscoveryStrategy(id: string): Promise<DiscoveryStrategy> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/discovery-strategy/approve`,
    { method: 'POST', credentials: 'same-origin' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as DiscoveryStrategy;
}

/** An earlier analysis of the same target rejected this same candidate (AC1.4). */
export type PreviousRejection = {
  reason: string;
  rejectedAt: number;
  analysisId: string;
};

export type FeatureCandidate = {
  key: string;
  name: string;
  location: string;
  symbol: string | null;
  rationale: string;
  decision: 'undecided' | 'approved' | 'rejected';
  rejectReason: string | null;
  mergedInto: string | null;
  previouslyRejected: PreviousRejection | null;
};

/** The reviewable candidate list for one analysis (AC1.4). */
export type CandidateList = {
  candidates: FeatureCandidate[];
  /** Counted by the server, so the screen and the "결정 끝" gate read one number. */
  undecided: number;
  /** Whether stage 4 has produced its document yet. `false` is a state, not an error. */
  extracted: boolean;
};

export async function getCandidates(id: string): Promise<CandidateList> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/candidates`, {
    credentials: 'same-origin',
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as CandidateList;
}

async function candidateAction(
  id: string,
  action: string,
  body: Record<string, unknown>,
): Promise<CandidateList> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/candidates/${action}`, {
    method: 'POST',
    credentials: 'same-origin',
    headers: json,
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as CandidateList;
}

/** Approve one candidate (AC1.4). */
export function approveCandidate(id: string, key: string): Promise<CandidateList> {
  return candidateAction(id, 'decision', { key, decision: 'approve' });
}

/** Reject one candidate with the reason AC1.4 requires to be recorded. */
export function rejectCandidate(
  id: string,
  key: string,
  reason: string,
): Promise<CandidateList> {
  return candidateAction(id, 'decision', { key, decision: 'reject', reason });
}

/** Rename one candidate. Its key does not move — identity is where it was found. */
export function renameCandidate(
  id: string,
  key: string,
  name: string,
): Promise<CandidateList> {
  return candidateAction(id, 'rename', { key, name });
}

/** Fold candidates into one. The folded rows are kept, marked `mergedInto`. */
export function mergeCandidates(
  id: string,
  into: string,
  keys: string[],
): Promise<CandidateList> {
  return candidateAction(id, 'merge', { into, keys });
}
