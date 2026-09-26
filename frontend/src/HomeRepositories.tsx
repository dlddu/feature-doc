// Home — Repositories — the real, stateful screen behind
// docs/mockups/JRN-connect-repo.html#STP-pick-target and its pre-flight area
// docs/mockups/JRN-connect-repo.html#STP-confirm-cost (Home + Connect, AC1.1 · AC4.6 pre-flight).

import { useEffect, useState } from 'react';
import {
  createAnalysis,
  getInstallUrl,
  listAnalyses,
  listRepositories,
  logout,
  preflightAnalysis,
} from './api';
import type { Analysis, Preflight, Repository } from './api';
import { formatAgo, formatCost, formatSize } from './format';

const STATUS_BADGE: Record<string, { tone: string; label: string }> = {
  queued: { tone: 'info', label: 'Queued' },
  running: { tone: 'info', label: 'Analyzing' },
  // The worker drained the queue and ran every stage that exists today; the LLM
  // stages are still unimplemented, so this is deliberately not 'Synced'.
  awaiting_pipeline: { tone: 'info', label: 'Fetched' },
  succeeded: { tone: 'success', label: 'Synced' },
  failed: { tone: 'danger', label: 'Failed' },
};

type Phase = 'idle' | 'checking' | 'starting';

function badgeFor(status: string): { tone: string; label: string } {
  return STATUS_BADGE[status] ?? { tone: '', label: status };
}

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

type Row = {
  key: string;
  fullName: string;
  branch: string;
  latest: Analysis | null;
  accessible: boolean;
};

/**
 * A job whose repository has since left the App's installation scope must not
 * silently vanish from the list, so history is unioned in rather than filtered by
 * what is reachable today.
 */
function buildRows(repos: Repository[], analyses: Analysis[]): Row[] {
  const newestFor = (owner: string, name: string): Analysis | null =>
    analyses.find(
      (a) =>
        a.repoOwner.toLowerCase() === owner.toLowerCase() &&
        a.repoName.toLowerCase() === name.toLowerCase(),
    ) ?? null;

  const rows: Row[] = repos.map((r) => {
    const latest = newestFor(r.owner, r.name);
    return {
      key: r.fullName.toLowerCase(),
      fullName: r.fullName,
      branch: latest?.branch ?? r.defaultBranch,
      latest,
      accessible: true,
    };
  });

  const seen = new Set(rows.map((r) => r.key));
  for (const a of analyses) {
    const fullName = a.repoOwner + '/' + a.repoName;
    const key = fullName.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    rows.push({ key, fullName, branch: a.branch, latest: a, accessible: false });
  }
  return rows;
}

type Props = {
  onOpenCredentials: () => void;
  onOpenAnalysis: (analysisId: string) => void;
  onLoggedOut: () => void;
  onAnalysisQueued: () => void;
};

export function HomeRepositories({
  onOpenCredentials,
  onOpenAnalysis,
  onLoggedOut,
  onAnalysisQueued,
}: Props) {
  const [repos, setRepos] = useState<Repository[] | null>(null);
  const [analyses, setAnalyses] = useState<Analysis[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [signingOut, setSigningOut] = useState(false);

  const [repoUrl, setRepoUrl] = useState('');
  const [branch, setBranch] = useState('');
  const [estimate, setEstimate] = useState<Preflight | null>(null);
  const [phase, setPhase] = useState<Phase>('idle');

  useEffect(() => {
    void (async () => {
      try {
        const [rs, as] = await Promise.all([listRepositories(), listAnalyses()]);
        setRepos(rs);
        setAnalyses(as);
      } catch (e) {
        setRepos([]);
        setError(messageOf(e));
      }
    })();
  }, []);

  const rows = buildRows(repos ?? [], analyses);

  // A failure is shown rather than swallowed — a logout the user believes happened
  // but did not is the one outcome worth interrupting for.
  async function signOut() {
    setSigningOut(true);
    try {
      await logout();
      onLoggedOut();
    } catch (e) {
      setSigningOut(false);
      setError(messageOf(e));
    }
  }

  /** Any edit invalidates the estimate — the trigger must never use a stale one. */
  function edit(setter: (v: string) => void) {
    return (value: string) => {
      setter(value);
      setEstimate(null);
      setError(null);
    };
  }

  function pick(fullName: string) {
    edit(setRepoUrl)('github.com/' + fullName);
  }

  async function check() {
    setPhase('checking');
    setError(null);
    try {
      setEstimate(await preflightAnalysis(repoUrl, branch));
    } catch (e) {
      setEstimate(null);
      setError(messageOf(e));
    } finally {
      setPhase('idle');
    }
  }

  async function start() {
    setPhase('starting');
    setError(null);
    try {
      await createAnalysis(repoUrl, branch);
      onAnalysisQueued();
    } catch (e) {
      setError(messageOf(e));
      setPhase('idle');
    }
  }

  async function openInstallScope() {
    try {
      window.location.href = await getInstallUrl();
    } catch (e) {
      setError(messageOf(e));
    }
  }

  const ready = estimate?.hasAccess === true;
  const busy = phase !== 'idle';

  return (
    <main className="screen has-tabbar">
      <header className="appbar">
        <span className="appbar-title">Repositories</span>
        <button
          className="btn-link"
          type="button"
          onClick={() => void signOut()}
          disabled={signingOut}
          data-testid="logout"
        >
          로그아웃
        </button>
      </header>

      <div className="section-title" style={{ marginTop: 16 }}>
        <span>연결된 저장소</span>
        <span className="count">{rows.length}</span>
      </div>

      <div className="stack-10" style={{ marginTop: 10 }}>
        {repos === null && (
          <p className="body sm" data-testid="home-loading">
            불러오는 중…
          </p>
        )}

        {error && (
          <div className="row" data-testid="home-error">
            <span className="badge danger">
              <span className="dot" />
              Error
            </span>
            <span className="body sm">{error}</span>
          </div>
        )}

        {repos !== null && !error && rows.length === 0 && (
          <p
            className="body sm"
            style={{ marginTop: 10, color: 'var(--text-tertiary)' }}
            data-testid="home-empty"
          >
            아직 분석한 저장소가 없어요. 아래에서 첫 저장소를 연결해 보세요.
          </p>
        )}

        {rows.map((row) => {
          const badge = row.latest ? badgeFor(row.latest.status) : null;
          return (
            <div
              className="card pickable"
              key={row.key}
              data-testid="repo-card"
              role="button"
              tabIndex={0}
              onClick={() => pick(row.fullName)}
              onKeyDown={(e) => {
                if (e.target !== e.currentTarget) return;
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  pick(row.fullName);
                }
              }}
            >
              <div className="row between top">
                <div className="grow">
                  <div className="body" style={{ fontWeight: 600 }}>
                    {row.fullName}
                  </div>
                  <div className="meta" style={{ marginTop: 3 }}>
                    ⎇ {row.branch}
                    {row.latest ? ' · ' + formatAgo(row.latest.createdAt) : ' · not analyzed'}
                  </div>
                </div>
                {badge && (
                  <span className={`badge ${badge.tone}`.trim()}>
                    <span className="dot" />
                    {badge.label}
                  </span>
                )}
              </div>
              {row.latest && (
                <div className="row between" style={{ marginTop: 12 }}>
                  <span className="meta">
                    step {row.latest.stagesDone} of {row.latest.stagesTotal}{' '}
                    <span className="dot-sep">·</span> ~{row.latest.estLlmCalls} LLM calls{' '}
                    <span className="dot-sep">·</span> est {formatCost(row.latest.estCostCents)}
                  </span>
                  <button
                    className="section-action"
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      onOpenAnalysis(row.latest!.id);
                    }}
                    data-testid="open-progress"
                  >
                    진행 상황
                  </button>
                </div>
              )}
              {!row.accessible && (
                <div className="meta" style={{ marginTop: 8 }}>
                  App 설치 범위 밖 — 다시 분석하려면 설치 범위에 추가해 주세요.
                </div>
              )}
            </div>
          );
        })}
      </div>

      <hr className="divider" style={{ marginTop: 22, marginBottom: 22 }} />

      <div>
        <h1 className="page-h1">새 저장소 연결</h1>
        <p className="h-display-sub" style={{ marginTop: 6 }}>
          분석을 시작하면 횡단 관심사 → 탐색 전략 → 기능 후보 순으로 진행돼요.
        </p>
      </div>

      <div className="stack-10" style={{ marginTop: 18 }}>
        <div className="input">
          <span className="lbl">Repository URL</span>
          <input
            className="field-input"
            type="text"
            value={repoUrl}
            placeholder="github.com/owner/repo"
            autoComplete="off"
            autoCapitalize="none"
            spellCheck={false}
            aria-label="Repository URL"
            data-testid="repo-url"
            onChange={(e) => edit(setRepoUrl)(e.target.value)}
          />
        </div>
        <div className="input">
          <span className="lbl">Branch</span>
          <input
            className="field-input"
            type="text"
            value={branch}
            placeholder="기본 브랜치"
            autoComplete="off"
            autoCapitalize="none"
            spellCheck={false}
            aria-label="Branch"
            data-testid="branch"
            onChange={(e) => edit(setBranch)(e.target.value)}
          />
        </div>
      </div>

      {estimate && !estimate.hasAccess && (
        <div className="notice err" style={{ marginTop: 12 }} data-testid="no-access">
          이 저장소에는 접근할 수 없어요 — App 설치 범위 밖입니다.
          <button
            className="btn btn-secondary"
            type="button"
            onClick={openInstallScope}
            data-testid="manage-install"
          >
            설치 범위에 추가하기
          </button>
        </div>
      )}

      {estimate?.hasAccess && (
        <div className="notice ok" style={{ marginTop: 12 }} data-testid="access-ok">
          접근 가능한 저장소예요. 비용을 확인하고 시작할 수 있습니다.
        </div>
      )}

      {error && (
        <div className="row" style={{ marginTop: 12 }} data-testid="connect-error">
          <span className="badge danger">
            <span className="dot" />
            Error
          </span>
          <span className="body sm">{error}</span>
        </div>
      )}

      <div className="row between" style={{ marginTop: 14 }} data-testid="access">
        <span className="body sm" style={{ color: 'var(--text-primary)' }}>
          GitHub App
        </span>
        <span className="meta">
          {estimate ? (estimate.hasAccess ? '✓ has access' : '✕ no access') : '미확인'}
        </span>
      </div>

      {!ready && (
        <div className="stack" style={{ marginTop: 20 }}>
          <button
            className="btn btn-primary block"
            type="button"
            onClick={() => void check()}
            disabled={busy || repoUrl.trim() === ''}
            data-testid="check-access"
          >
            {'비용 확인하기'}
          </button>
        </div>
      )}

      {estimate?.hasAccess && (
        <>
          <div style={{ marginTop: 20 }}>
            <h1 className="h-display">얼마나 들까요</h1>
            <p className="h-display-sub">
              분석은 이 버튼으로만 시작돼요. 자동으로 시작되는 경로는 없습니다.
            </p>
          </div>

          <div className="card stack-14" style={{ marginTop: 20 }} data-testid="estimate">
            <div className="row between">
              <span className="caps">Target</span>
              <span className="badge success">
                <span className="dot" />
                Ready
              </span>
            </div>
            <div className="stack">
              <div className="row between">
                <span className="body sm" style={{ color: 'var(--text-primary)' }}>
                  Repository
                </span>
                <span className="meta" style={{ color: 'var(--text-primary)' }}>
                  {estimate.fullName}
                </span>
              </div>
              <div className="row between">
                <span className="body sm" style={{ color: 'var(--text-primary)' }}>
                  Branch
                </span>
                <span className="meta" style={{ color: 'var(--text-primary)' }}>
                  ⎇ {estimate.branch}
                </span>
              </div>
            </div>
            <hr className="divider" />
            <div className="row" style={{ gap: 0 }}>
              <div className="grow">
                <div className="metric">~{formatCost(estimate.estCostCents)}</div>
                <div className="caps" style={{ marginTop: 6 }}>
                  Est. LLM Cost
                </div>
              </div>
              <div className="grow">
                <div className="metric">~{estimate.estDurationMin} min</div>
                <div className="caps" style={{ marginTop: 6 }}>
                  Est. Duration
                </div>
              </div>
            </div>
            <hr className="divider" />
            <div className="stack">
              <div className="row between">
                <span className="body sm" style={{ color: 'var(--text-primary)' }}>
                  Files to scan
                </span>
                <span className="meta" style={{ color: 'var(--text-primary)' }}>
                  {estimate.filesToScan} <span className="dot-sep">·</span>{' '}
                  {formatSize(estimate.sizeBytes)}
                </span>
              </div>
              <div className="row between">
                <span className="body sm" style={{ color: 'var(--text-primary)' }}>
                  Est. LLM calls
                </span>
                <span className="meta" style={{ color: 'var(--text-primary)' }}>
                  ~{estimate.estLlmCalls}
                </span>
              </div>
            </div>
          </div>

          <div className="stack" style={{ marginTop: 20 }}>
            <button
              className="btn btn-primary block"
              type="button"
              onClick={() => void start()}
              disabled={busy}
              data-testid="start-analysis"
            >
              Start Analysis →
            </button>
          </div>
        </>
      )}

      <nav className="tabbar">
        <button className="tab active" type="button" data-testid="tab-repos">
          <span className="gl" />
          Repos
        </button>
        <button className="tab" type="button" disabled>
          <span className="gl" />
          Activity
        </button>
        <button className="tab" type="button" onClick={onOpenCredentials} data-testid="tab-keys">
          <span className="gl" />
          Keys
        </button>
        <button className="tab" type="button" disabled>
          <span className="gl" />
          Settings
        </button>
      </nav>
    </main>
  );
}
