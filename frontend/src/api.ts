// The SPA and the API share an origin in every deployment — that is what every
// `credentials: 'same-origin'` below relies on.

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

export type LlmLanguage = 'ko' | 'en';

/** Full-page navigation, not fetch — the OAuth redirect chain is the browser's to follow. */
export const LOGIN_URL = '/api/auth/login';

const json = { 'content-type': 'application/json' };

async function errorMessage(res: Response): Promise<string> {
  try {
    const body = (await res.json()) as { error?: string };
    if (body.error) return body.error;
  } catch {
  }
  return `요청에 실패했어요 (${res.status})`;
}

export async function getMe(): Promise<User | null> {
  const res = await fetch('/api/me', { credentials: 'same-origin' });
  if (res.status === 401) return null;
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as User;
}

export async function logout(): Promise<void> {
  const res = await fetch('/api/auth/logout', {
    method: 'POST',
    credentials: 'same-origin',
  });
  if (!res.ok) throw new Error(await errorMessage(res));
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

export async function getLlmLanguage(): Promise<LlmLanguage> {
  const res = await fetch('/api/settings', { credentials: 'same-origin' });
  if (!res.ok) throw new Error(await errorMessage(res));
  return ((await res.json()) as { llmLanguage: LlmLanguage }).llmLanguage;
}

export async function setLlmLanguage(llmLanguage: LlmLanguage): Promise<LlmLanguage> {
  const res = await fetch('/api/settings', {
    method: 'PUT',
    credentials: 'same-origin',
    headers: json,
    body: JSON.stringify({ llmLanguage }),
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return ((await res.json()) as { llmLanguage: LlmLanguage }).llmLanguage;
}

export async function preflight(): Promise<{ provider: string; fingerprint: string }> {
  const res = await fetch('/api/llm-keys/preflight', { credentials: 'same-origin' });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as { provider: string; fingerprint: string };
}

export type Repository = {
  owner: string;
  name: string;
  fullName: string;
  defaultBranch: string;
};

export type Analysis = {
  id: string;
  repoOwner: string;
  repoName: string;
  branch: string;
  status: string;
  estLlmCalls: number;
  estCostCents: number;
  createdAt: number;
  llmLanguage: LlmLanguage | null;
  /** Denormalized onto the list row so a card can show progress without a second fetch. */
  stagesDone: number;
  stagesTotal: number;
};

export type Stage = {
  seq: number;
  key: string;
  title: string;
  status: 'pending' | 'running' | 'succeeded' | 'failed';
  detail: string | null;
  error: string | null;
  startedAt: number | null;
  finishedAt: number | null;
};

export type AnalysisDetail = Analysis & {
  error: string | null;
  startedAt: number | null;
  finishedAt: number | null;
  stages: Stage[];
};

/**
 * `hasAccess: false` is not an error — it is the answer, and the screen renders a
 * recovery path from it.
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

/** Empty — not an error — when the App is not installed yet. */
export async function listRepositories(): Promise<Repository[]> {
  const res = await fetch('/api/repositories', { credentials: 'same-origin' });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as Repository[];
}

/** Newest first. */
export async function listAnalyses(): Promise<Analysis[]> {
  const res = await fetch('/api/analyses', { credentials: 'same-origin' });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as Analysis[];
}

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

export async function getAnalysis(id: string): Promise<AnalysisDetail> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}`, {
    credentials: 'same-origin',
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as AnalysisDetail;
}

/**
 * The job returns to the queue, so the answer already carries the reset progress —
 * the caller does not refetch.
 */
export async function retryStage(id: string, stageKey: string): Promise<AnalysisDetail> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/stages/${encodeURIComponent(stageKey)}/retry`,
    { method: 'POST', credentials: 'same-origin' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as AnalysisDetail;
}

/** An out-of-scope target is rejected without ever being queued. */
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

export type CrossCuttingItem = {
  name: string;
  evidence: string[];
};

export type CrossCuttingCategory = {
  axis: string;
  items: CrossCuttingItem[];
};

export type Reproducibility = {
  verdict: 'first' | 'unchanged' | 'changed';
  comparedTo: string | null;
};

export type CrossCuttingDocument = {
  kind: string;
  content: { categories: CrossCuttingCategory[] };
  model: string;
  createdAt: number;
  reproducibility: Reproducibility;
};

/**
 * A 404 means the stage has not produced the document yet — a distinct state from
 * "ran and found nothing", so it is surfaced rather than flattened to an empty one.
 * The same reading applies to every `documents/*` read below.
 */
export async function getCrossCutting(id: string): Promise<CrossCuttingDocument> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/documents/cross-cutting`, {
    credentials: 'same-origin',
  });
  if (res.status === 404) throw new Error('아직 생성되지 않았어요');
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as CrossCuttingDocument;
}

export type StrategyEntry = {
  pattern: string;
  source: 'generated' | 'user';
};

export type DiscoveryStrategy = {
  entries: StrategyEntry[];
  approved: boolean;
  approvedAt: number | null;
  updatedAt: number;
};

export async function getDiscoveryStrategy(id: string): Promise<DiscoveryStrategy> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/discovery-strategy`, {
    credentials: 'same-origin',
  });
  if (res.status === 404) throw new Error('아직 생성되지 않았어요');
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as DiscoveryStrategy;
}

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

export async function approveDiscoveryStrategy(id: string): Promise<DiscoveryStrategy> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/discovery-strategy/approve`,
    { method: 'POST', credentials: 'same-origin' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as DiscoveryStrategy;
}

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
  previouslyDeleted: PreviousDeletion | null;
};

export type PreviousDeletion = {
  reason: string | null;
  deletedAt: number;
  analysisId: string;
};

export type CandidateList = {
  candidates: FeatureCandidate[];
  /** Counted by the server so the screen and its 「결정 끝」 gate read one number. */
  undecided: number;
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

export function approveCandidate(id: string, key: string): Promise<CandidateList> {
  return candidateAction(id, 'decision', { key, decision: 'approve' });
}

export function rejectCandidate(
  id: string,
  key: string,
  reason: string,
): Promise<CandidateList> {
  return candidateAction(id, 'decision', { key, decision: 'reject', reason });
}

/** The key does not move — a candidate's identity is where it was found. */
export function renameCandidate(
  id: string,
  key: string,
  name: string,
): Promise<CandidateList> {
  return candidateAction(id, 'rename', { key, name });
}

/** Folded rows are kept, marked `mergedInto`. */
export function mergeCandidates(
  id: string,
  into: string,
  keys: string[],
): Promise<CandidateList> {
  return candidateAction(id, 'merge', { into, keys });
}

export type AcceptanceScenario = {
  given: string;
  when: string;
  then: string;
  evidence: string;
  symbol: string | null;
  source: 'logic' | 'test' | 'user_llm' | 'user_direct';
};

export type AcceptanceContradiction = {
  given: string;
  when: string;
  codeSays: string;
  codeEvidence: string;
  testSays: string;
  testEvidence: string;
};

export type FeatureAcceptance = {
  key: string;
  name: string;
  location: string | null;
  symbol: string | null;
  scenarios: AcceptanceScenario[];
  contradictions: AcceptanceContradiction[];
};

export type AcceptanceDocument = {
  kind: string;
  content: { features: FeatureAcceptance[] };
  model: string;
  createdAt: number;
  reproducibility: Reproducibility;
};

/**
 * `null` rather than a thrown 404, unlike the reads above: not-yet-generated is a
 * state this screen draws rather than an error. The distinction from "ran and found
 * nothing" survives anyway — the stage refuses to write an empty feature list.
 */
export async function getAcceptance(id: string): Promise<AcceptanceDocument | null> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/documents/acceptance-dependencies`,
    { credentials: 'same-origin' },
  );
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as AcceptanceDocument;
}

/** `evidence: null` is 「근거 없음」 — recorded, never invented. */
export type Dependency = {
  category: string;
  name: string;
  evidence: string | null;
};

export type DependencyGroup = {
  category: string;
  items: Dependency[];
};

/**
 * `status: null` means nobody has asked yet — a different state from "asked and it
 * found nothing".
 */
export type FeatureDependencies = {
  featureKey: string;
  featureName: string | null;
  status: string | null;
  error: string | null;
  categories: DependencyGroup[];
  total: number;
  withoutEvidence: number;
};

export async function getDependencies(
  id: string,
  key: string,
): Promise<FeatureDependencies> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/features/dependencies?key=${encodeURIComponent(key)}`,
    { credentials: 'same-origin' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as FeatureDependencies;
}

/** Records the request and re-queues the analysis. */
export async function requestDependencies(
  id: string,
  key: string,
): Promise<FeatureDependencies> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/features/dependencies`,
    {
      method: 'POST',
      credentials: 'same-origin',
      headers: json,
      body: JSON.stringify({ key }),
    },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as FeatureDependencies;
}

export type ScenarioLine = {
  mark: string;
  text: string;
};

export type DependencyLine = {
  mark: string;
  category: string;
  name: string;
};

export type FeatureDiff = {
  key: string;
  name: string;
  location: string | null;
  scenarios: number;
  scenarioLines: ScenarioLine[];
  dependencyLines: DependencyLine[];
};

/**
 * `comparedTo: null` 은 견줄 상대가 없다는 뜻이고 — 같은 저장소·브랜치의 첫 분석이다 —
 * 「바뀐 게 없다」(`features: []` 이면서 `comparedTo` 가 있는 경우)와 다른 상태다.
 */
export type AnalysisDiff = {
  comparedTo: string | null;
  comparedToCreatedAt: number | null;
  changedLineCount: number;
  features: FeatureDiff[];
};

export async function getAnalysisDiff(id: string): Promise<AnalysisDiff> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/diff`, {
    credentials: 'same-origin',
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as AnalysisDiff;
}

export type Sentences = {
  given: string;
  when: string;
  then: string;
};

export type EditContext = {
  featureKey: string;
  featureName: string | null;
  scenarioIndex: number;
  scenarioCount: number;
  target: Sentences;
  rejectedCount: number;
  rejectedReason: string | null;
};

export type EditProposal = {
  id: string;
  featureKey: string;
  scenarioIndex: number;
  status: string;
  source: string;
  request: string;
  before: Sentences;
  after: Sentences[];
  removed: string[];
  added: string[];
  changedLines: number;
};

export async function getEditContext(
  id: string,
  key: string,
  scenario: number,
): Promise<EditContext> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/features/edit-context` +
      `?key=${encodeURIComponent(key)}&scenario=${scenario}`,
    { credentials: 'same-origin' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as EditContext;
}

/** 제안을 만든다. 문서는 승인 전까지 그대로다. */
export async function proposeEdit(
  id: string,
  key: string,
  scenarioIndex: number,
  request: string,
): Promise<EditProposal> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/features/edit-proposals`, {
    method: 'POST',
    credentials: 'same-origin',
    headers: json,
    body: JSON.stringify({ key, scenarioIndex, request }),
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as EditProposal;
}

export async function getEditProposal(id: string, proposal: string): Promise<EditProposal> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/features/edit-proposals/${encodeURIComponent(proposal)}`,
    { credentials: 'same-origin' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as EditProposal;
}

/** 승인이면 그 자리에 얹히고, 거부면 다음 제안이 피해야 할 것이 된다. */
export async function decideEdit(
  id: string,
  proposal: string,
  decision: 'approve' | 'reject',
  reason?: string,
): Promise<EditProposal> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/features/edit-proposals/${encodeURIComponent(proposal)}/decision`,
    {
      method: 'POST',
      credentials: 'same-origin',
      headers: json,
      body: JSON.stringify({ decision, reason }),
    },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as EditProposal;
}

export type DraftScenario = {
  given: string;
  when: string;
  then: string;
  evidence: string;
};

export type DraftDependency = {
  category: string;
  name: string;
  evidence: string | null;
};

export type FeatureAddition = {
  id: string;
  key: string;
  name: string;
  request: string;
  status: string;
  evidenceFound: boolean;
  source: string | null;
  scenarios: DraftScenario[];
  dependencies: DraftDependency[];
};

export type FeatureAdditions = {
  approvedCandidates: number;
  confirmedAdditions: number;
  finalCount: number;
  additions: FeatureAddition[];
};

export async function getAdditions(id: string): Promise<FeatureAdditions> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/features/additions`, {
    credentials: 'same-origin',
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as FeatureAdditions;
}

/** 초안을 만든다. 근거를 못 찾으면 초안은 비어 오고, 문서는 확정 전까지 그대로다. */
export async function draftAddition(id: string, request: string): Promise<FeatureAddition> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/features/additions`, {
    method: 'POST',
    credentials: 'same-origin',
    headers: json,
    body: JSON.stringify({ request }),
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as FeatureAddition;
}

export async function getAddition(id: string, addition: string): Promise<FeatureAddition> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/features/additions/${encodeURIComponent(addition)}`,
    { credentials: 'same-origin' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as FeatureAddition;
}

/** 확정이면 feature 가 되어 문서 끝에 얹히고, 취소면 시도로만 남는다. */
export async function decideAddition(
  id: string,
  addition: string,
  decision: 'confirm' | 'cancel',
): Promise<FeatureAddition> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/features/additions/${encodeURIComponent(addition)}/decision`,
    {
      method: 'POST',
      credentials: 'same-origin',
      headers: json,
      body: JSON.stringify({ decision }),
    },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as FeatureAddition;
}

export type FeatureDeletion = {
  id: string;
  key: string;
  name: string;
  reason: string | null;
  deletedAt: number;
  restoreUntil: number;
  restoredAt: number | null;
  restorable: boolean;
};

export type FeatureDeletionList = {
  retentionDays: number;
  deletions: FeatureDeletion[];
};

export async function listDeletions(id: string): Promise<FeatureDeletionList> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/features/deletions`, {
    credentials: 'same-origin',
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as FeatureDeletionList;
}

export async function deleteFeature(
  id: string,
  key: string,
  reason: string,
): Promise<FeatureDeletion> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/features/deletions`, {
    method: 'POST',
    credentials: 'same-origin',
    headers: json,
    body: JSON.stringify({ key, reason: reason.trim() === '' ? null : reason }),
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as FeatureDeletion;
}

export async function restoreFeature(id: string, deletion: string): Promise<FeatureDeletion> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/features/deletions/${encodeURIComponent(deletion)}/restore`,
    { method: 'POST', credentials: 'same-origin', headers: json, body: '{}' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as FeatureDeletion;
}

export type DocConflict = {
  id: string;
  featureKey: string;
  featureName: string;
  scenarioIndex: number;
  status: 'open' | 'auto' | 'mine' | 'merged';
  source: string;
  request: string;
  mine: Sentences[];
  before: Sentences;
  auto: Sentences;
  mineDecidedAt: number;
  previousAnalysisId: string;
  mergeProposal: MergeProposal | null;
  decidedAt: number | null;
};

export type MergeProposal = {
  id: string;
  status: string;
  after: Sentences[];
  removed: string[];
  added: string[];
  changedLines: number;
};

export type DocConflictList = {
  open: number;
  conflicts: DocConflict[];
};

export async function listConflicts(id: string): Promise<DocConflictList> {
  const res = await fetch(`/api/analyses/${encodeURIComponent(id)}/conflicts`, {
    credentials: 'same-origin',
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as DocConflictList;
}

export async function getConflict(id: string, conflict: string): Promise<DocConflict> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/conflicts/${encodeURIComponent(conflict)}`,
    { credentials: 'same-origin' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as DocConflict;
}

export async function decideConflict(
  id: string,
  conflict: string,
  decision: 'auto' | 'mine',
): Promise<DocConflict> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/conflicts/${encodeURIComponent(conflict)}/decision`,
    { method: 'POST', credentials: 'same-origin', headers: json, body: JSON.stringify({ decision }) },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as DocConflict;
}

export async function proposeMerge(
  id: string,
  conflict: string,
  keepMine: boolean,
): Promise<DocConflict> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/conflicts/${encodeURIComponent(conflict)}/merge`,
    { method: 'POST', credentials: 'same-origin', headers: json, body: JSON.stringify({ keepMine }) },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as DocConflict;
}

export async function decideMerge(
  id: string,
  conflict: string,
  decision: 'approve' | 'reject',
): Promise<DocConflict> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/conflicts/${encodeURIComponent(conflict)}/merge-decision`,
    { method: 'POST', credentials: 'same-origin', headers: json, body: JSON.stringify({ decision }) },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as DocConflict;
}

/** 한 feature 문서의 변경 하나(AC3.4). 자동 기준선·승인된 편집·복원이 같은 목록에 선다. */
export type HistoryEntry = {
  id: string;
  kind: 'auto' | 'edit' | 'restore';
  source: string;
  at: number;
  request: string | null;
  before: Sentences | null;
  after: Sentences[];
  carriedFrom: string | null;
  restoredTo: string | null;
  current: boolean;
  standing: boolean;
};

export type FeatureHistory = {
  featureKey: string;
  featureName: string | null;
  entries: HistoryEntry[];
  scenarios: Sentences[];
};

/** 되돌리기 전에 보는 그 시점의 상태. `lines` 가 비어 있으면 이미 그 시점이다. */
export type HistoryPreview = {
  entryId: string;
  scenarios: Sentences[];
  lines: { mark: string; text: string }[];
  isCurrent: boolean;
};

export async function getHistory(id: string, featureKey: string): Promise<FeatureHistory> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/features/${encodeURIComponent(featureKey)}/history`,
    { credentials: 'same-origin' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as FeatureHistory;
}

export async function getHistoryPoint(
  id: string,
  featureKey: string,
  entry: string,
): Promise<HistoryPreview> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/features/${encodeURIComponent(featureKey)}` +
      `/history/${encodeURIComponent(entry)}`,
    { credentials: 'same-origin' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as HistoryPreview;
}

export async function restoreHistoryPoint(
  id: string,
  featureKey: string,
  entry: string,
): Promise<FeatureHistory> {
  const res = await fetch(
    `/api/analyses/${encodeURIComponent(id)}/features/${encodeURIComponent(featureKey)}` +
      `/history/${encodeURIComponent(entry)}/restore`,
    { method: 'POST', credentials: 'same-origin', headers: json, body: '{}' },
  );
  if (!res.ok) throw new Error(await errorMessage(res));
  return (await res.json()) as FeatureHistory;
}
