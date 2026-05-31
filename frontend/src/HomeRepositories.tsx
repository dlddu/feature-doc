// S02 · Home · Repositories — the real screen behind
// docs/mockups/s02-home-repositories.html (AC1.1 · AC1.5).
//
// Analysis-derived values (features/spend/progress/status) are not produced yet,
// so every connected repo renders the reserved "Not analyzed" state and the
// aggregate metrics show "—". The "+ New" action is inert here (connecting a repo
// is S03, out of scope).

import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { listRepositories } from './api';
import type { Repository } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export function HomeRepositories() {
  // undefined = still loading
  const [repos, setRepos] = useState<Repository[] | undefined>(undefined);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    void load();
  }, []);

  async function load() {
    try {
      setRepos(await listRepositories());
    } catch (e) {
      setRepos([]);
      setLoadError(messageOf(e));
    }
  }

  const count = repos?.length ?? 0;

  return (
    <>
      <main className="screen has-tabbar" data-testid="home">
        <div className="row between" style={{ paddingTop: 14 }}>
          <div>
            <span className="brand">
              <span className="mk">●</span> FeatureDoc
            </span>
            <h1 className="h-display" style={{ marginTop: 6 }}>
              Repositories
            </h1>
          </div>
          <button className="icon-btn" type="button" aria-label="settings" disabled>
            <SettingsIcon />
          </button>
        </div>

        <div className="metric-grid" style={{ marginTop: 18 }} data-testid="metrics">
          <div className="cell">
            <div className="k">Repos</div>
            <div className="v" data-testid="metric-repos">
              {count}
            </div>
          </div>
          <div className="cell">
            <div className="k">Features</div>
            <div className="v">—</div>
          </div>
          <div className="cell">
            <div className="k">Spend</div>
            <div className="v">—</div>
          </div>
        </div>

        <div className="section-title" style={{ marginTop: 28, alignItems: 'center' }}>
          <span>Repositories</span>
          <span className="row" style={{ gap: 12 }}>
            <span className="count">{count}</span>
            <button className="section-action" type="button" disabled data-testid="new-repo">
              <PlusIcon /> New
            </button>
          </span>
        </div>

        {repos === undefined ? (
          <p className="body sm" style={{ marginTop: 12 }}>
            불러오는 중…
          </p>
        ) : count === 0 ? (
          <div className="card" style={{ marginTop: 12 }} data-testid="repos-empty">
            <p className="body">연결된 저장소가 없어요</p>
            <p className="body sm" style={{ marginTop: 6 }}>
              GitHub App으로 접근을 허용한 저장소를 연결하면 여기에 표시돼요.
            </p>
          </div>
        ) : (
          <div className="stack-10" style={{ marginTop: 12 }} data-testid="repo-list">
            {repos.map((r) => (
              <RepoCard key={r.id} repo={r} />
            ))}
          </div>
        )}

        {loadError && (
          <div className="row" style={{ marginTop: 12 }}>
            <span className="badge danger">
              <span className="dot" />
              Error
            </span>
            <span className="body sm">{loadError}</span>
          </div>
        )}

        <p className="legend" style={{ marginTop: 24 }}>
          <span className="mk">02</span> — discovery · home view
        </p>
      </main>

      <nav className="tabbar" aria-label="primary">
        <span className="tab active" aria-current="page">
          <span className="gl" />
          Repos
        </span>
        <span className="tab disabled" aria-disabled="true">
          <span className="gl" />
          Activity
        </span>
        <Link className="tab" to="/setup" data-testid="tab-keys">
          <span className="gl" />
          Keys
        </Link>
        <span className="tab disabled" aria-disabled="true">
          <span className="gl" />
          Settings
        </span>
      </nav>
    </>
  );
}

function RepoCard({ repo }: { repo: Repository }) {
  // The analysis pipeline has not produced derived values yet, so every repo shows
  // the reserved "Not analyzed" badge and "—" placeholders (plan: null 예약).
  const label = repo.status === 'not_analyzed' ? 'Not analyzed' : repo.status;
  return (
    <div className="card" data-testid="repo-card">
      <div className="row between top">
        <div className="grow">
          <div className="body" style={{ fontWeight: 600 }}>
            {repo.name}
          </div>
          <div className="meta" style={{ marginTop: 3 }}>
            ⎇ {repo.branch}
          </div>
        </div>
        <span className="badge">
          <span className="dot" />
          {label}
        </span>
      </div>
      <div className="meta" style={{ marginTop: 12 }}>
        — features <span className="dot-sep">·</span> — conflicts <span className="dot-sep">·</span> —
      </div>
    </div>
  );
}

function SettingsIcon() {
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
      <path d="M5.5 1.6v7.8M1.6 5.5h7.8" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
    </svg>
  );
}
