// S02 · Home — Repositories — the real, stateful screen behind
// docs/mockups/s02-home-repositories.html (AC1.1).
//
// Reads the slice-2a enqueue contract: the repositories the GitHub App can reach
// (`GET /api/repositories`) and the analysis jobs the user has triggered
// (`GET /api/analyses`). Per-stage progress and the pipeline outputs that the
// mockup's "features / conflicts / spend" figures imply arrive with later slices
// (AC1.5 · AC1.2~1.4 · AC4.6) — this screen only shows what the API really knows.

import { useEffect, useState } from 'react';
import { listAnalyses, listRepositories } from './api';
import type { Analysis, Repository } from './api';
import { formatAgo, formatCost } from './format';

/** How an analysis status renders as a status badge (design-system §4.2 tag). */
const STATUS_BADGE: Record<string, { tone: string; label: string }> = {
  queued: { tone: 'info', label: 'Queued' },
  running: { tone: 'info', label: 'Analyzing' },
  // The worker drained the queue and ran every stage that exists today; the LLM
  // stages are still unimplemented, so this is deliberately not 'Synced'.
  awaiting_pipeline: { tone: 'info', label: 'Fetched' },
  succeeded: { tone: 'success', label: 'Synced' },
  failed: { tone: 'danger', label: 'Failed' },
};

function badgeFor(status: string): { tone: string; label: string } {
  return STATUS_BADGE[status] ?? { tone: '', label: status };
}

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** A repository row: the repo itself plus its most recent analysis, if any. */
type Row = {
  key: string;
  fullName: string;
  branch: string;
  latest: Analysis | null;
  accessible: boolean;
};

/**
 * Accessible repositories first (each carrying its newest analysis), then any
 * repository that only appears in the analysis history — a job whose repo has since
 * left the App's installation scope must not silently vanish from the home list.
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
    const key = `${a.repoOwner}/${a.repoName}`.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    rows.push({
      key,
      fullName: `${a.repoOwner}/${a.repoName}`,
      branch: a.branch,
      latest: a,
      accessible: false,
    });
  }
  return rows;
}

type Props = {
  /** S02 → S03 ("+ New"). */
  onConnectRepository: () => void;
  /** S02 → S01 (settings / Keys tab). */
  onOpenCredentials: () => void;
};

export function HomeRepositories({ onConnectRepository, onOpenCredentials }: Props) {
  const [repos, setRepos] = useState<Repository[] | null>(null);
  const [analyses, setAnalyses] = useState<Analysis[]>([]);
  const [error, setError] = useState<string | null>(null);

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
  const estimatedCents = analyses.reduce((sum, a) => sum + a.estCostCents, 0);

  return (
    <main className="screen has-tabbar">
      <div className="row between" style={{ paddingTop: 14 }}>
        <div>
          <span className="brand">
            <span className="mk">●</span> FeatureDoc
          </span>
          <h1 className="h-display" style={{ marginTop: 6 }}>
            Repositories
          </h1>
        </div>
        <button
          className="icon-btn"
          type="button"
          aria-label="settings"
          onClick={onOpenCredentials}
          data-testid="open-credentials"
        >
          <GearIcon />
        </button>
      </div>

      {/* The mockup's Features / Spend cells need pipeline output (slice 4+) and real
          per-call accounting (AC4.6); until those exist the grid shows what the API
          actually reports — see docs/doc-tracker.md "알려진 목업↔구현 편차". */}
      <div className="metric-grid" style={{ marginTop: 18 }} data-testid="metrics">
        <div className="cell">
          <div className="k">Repos</div>
          <div className="v" data-testid="metric-repos">
            {repos === null ? '—' : repos.length}
          </div>
        </div>
        <div className="cell">
          <div className="k">Analyses</div>
          <div className="v" data-testid="metric-analyses">
            {analyses.length}
          </div>
        </div>
        <div className="cell">
          <div className="k">Est. cost</div>
          <div className="v">{formatCost(estimatedCents)}</div>
        </div>
      </div>

      <div className="section-title" style={{ marginTop: 28, alignItems: 'center' }}>
        <span>Repositories</span>
        <span className="row" style={{ gap: 12 }}>
          <span className="count">{rows.length}</span>
          <button
            className="section-action"
            type="button"
            onClick={onConnectRepository}
            data-testid="new-repository"
          >
            <PlusIcon />
            New
          </button>
        </span>
      </div>

      <div className="stack-10" style={{ marginTop: 12 }}>
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
          <div className="card stack" data-testid="home-empty">
            <p className="body sm">
              아직 접근할 수 있는 저장소가 없어요. GitHub App을 연결하고 분석할 저장소를 고르면
              여기에 나타납니다.
            </p>
            <button
              className="btn btn-secondary block"
              type="button"
              onClick={onOpenCredentials}
              data-testid="empty-open-credentials"
            >
              GitHub App 연결하기
            </button>
          </div>
        )}

        {rows.map((row) => {
          const badge = row.latest ? badgeFor(row.latest.status) : null;
          return (
            <div className="card" key={row.key} data-testid="repo-card">
              <div className="row between top">
                <div className="grow">
                  <div className="body" style={{ fontWeight: 600 }}>
                    {row.fullName}
                  </div>
                  <div className="meta" style={{ marginTop: 3 }}>
                    ⎇ {row.branch}
                    {row.latest ? ` · ${formatAgo(row.latest.createdAt)}` : ' · not analyzed'}
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
                <div className="meta" style={{ marginTop: 12 }}>
                  ~{row.latest.estLlmCalls} LLM calls <span className="dot-sep">·</span> est{' '}
                  {formatCost(row.latest.estCostCents)}
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

      <p className="legend" style={{ marginTop: 24 }}>
        <span className="mk">02</span> — discovery · home view
      </p>

      <nav className="tabbar">
        <button className="tab active" type="button" data-testid="tab-repos">
          <span className="gl" />
          Repos
        </button>
        <button className="tab" type="button" disabled title="분석 파이프라인 슬라이스에서 열립니다">
          <span className="gl" />
          Activity
        </button>
        <button
          className="tab"
          type="button"
          onClick={onOpenCredentials}
          data-testid="tab-keys"
        >
          <span className="gl" />
          Keys
        </button>
        <button className="tab" type="button" disabled title="설정 화면은 후속 슬라이스입니다">
          <span className="gl" />
          Settings
        </button>
      </nav>
    </main>
  );
}

function GearIcon() {
  return (
    <svg width="15" height="15" viewBox="0 0 15 15" fill="none">
      <circle cx="7.5" cy="7.5" r="2.4" stroke="currentColor" strokeWidth="1.1" />
      <path
        d="M7.5 1.4v2M7.5 11.6v2M1.4 7.5h2M11.6 7.5h2M3.2 3.2l1.4 1.4M10.4 10.4l1.4 1.4M11.8 3.2l-1.4 1.4M4.6 10.4l-1.4 1.4"
        stroke="currentColor"
        strokeWidth="1.1"
      />
    </svg>
  );
}

function PlusIcon() {
  return (
    <svg width="11" height="11" viewBox="0 0 11 11" fill="none" aria-hidden="true">
      <path
        d="M5.5 1.6v7.8M1.6 5.5h7.8"
        stroke="currentColor"
        strokeWidth="1.4"
        strokeLinecap="round"
      />
    </svg>
  );
}
