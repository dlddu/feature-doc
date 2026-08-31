// S03 · Connect Repository — the real, stateful screen behind
// docs/mockups/JRN-connect-repo.html#STP-confirm-cost (S03, AC1.1 · AC4.6 pre-flight).
//
// The single primary action is deliberately two-phase: a target must pass
// pre-flight (`POST /api/analyses/preflight`) before it can be triggered, so the
// expected scale and cost are always on screen *before* the user enqueues anything
// (AC1.1: "사용자는 분석을 명시적으로 트리거할 수 있다" + 비용 사전 안내). A target
// outside the App's granted access never reaches the trigger: the screen shows the
// reason and the recovery path, and nothing is queued (test/01 시나리오 2).

import { useState } from 'react';
import { createAnalysis, getInstallUrl, preflightAnalysis } from './api';
import type { Preflight } from './api';
import { formatCost, formatSize } from './format';

type Phase = 'idle' | 'checking' | 'starting';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

type Props = {
  /** S03 → S02, after a successful trigger or a back tap. */
  onDone: () => void;
};

export function ConnectRepository({ onDone }: Props) {
  const [repoUrl, setRepoUrl] = useState('');
  const [branch, setBranch] = useState('');
  const [estimate, setEstimate] = useState<Preflight | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [phase, setPhase] = useState<Phase>('idle');

  /** Any edit invalidates the estimate — the trigger must never use a stale one. */
  function edit(setter: (v: string) => void) {
    return (value: string) => {
      setter(value);
      setEstimate(null);
      setError(null);
    };
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
      onDone();
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
    <main className="screen">
      <header className="appbar">
        <button
          className="icon-btn"
          type="button"
          aria-label="back"
          onClick={onDone}
          data-testid="back"
        >
          ‹
        </button>
        <span className="appbar-title">New Repository</span>
        <span className="icon-btn ghost" aria-hidden="true" />
      </header>

      <div style={{ marginTop: 22 }}>
        <h1 className="h-display">Connect a repository</h1>
        <p className="h-display-sub">
          분석을 시작하면 횡단 관심사 → 탐색 전략 → 기능 후보 순으로 진행돼요.
        </p>
      </div>

      <div className="stack-10" style={{ marginTop: 28 }}>
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
        <div className="input-row">
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
          <div className="input" data-testid="access">
            <span className="lbl">GitHub App</span>
            <span className={`val${estimate ? '' : ' placeholder'}`}>
              {estimate ? (estimate.hasAccess ? '✓ has access' : '✕ no access') : '미확인'}
            </span>
          </div>
        </div>
      </div>

      {estimate?.hasAccess && (
        <div className="card stack-14" style={{ marginTop: 18 }} data-testid="estimate">
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
      )}

      {estimate && !estimate.hasAccess && (
        <div className="card stack" style={{ marginTop: 18 }} data-testid="no-access">
          <div className="row">
            <span className="badge danger">
              <span className="dot" />
              No access
            </span>
            <span className="body sm grow">
              이 저장소에 접근할 수 없습니다. GitHub App 설치 범위에 {estimate.fullName} 을(를)
              추가해 주세요.
            </span>
          </div>
          <button
            className="btn btn-secondary block"
            type="button"
            onClick={openInstallScope}
            data-testid="manage-install"
          >
            App 설치 범위 관리
          </button>
        </div>
      )}

      {error && (
        <div className="row" style={{ marginTop: 18 }} data-testid="connect-error">
          <span className="badge danger">
            <span className="dot" />
            Error
          </span>
          <span className="body sm">{error}</span>
        </div>
      )}

      <div className="stack" style={{ marginTop: 18 }}>
        <button
          className="btn btn-primary block"
          type="button"
          onClick={ready ? start : check}
          disabled={busy || repoUrl.trim() === ''}
          data-testid={ready ? 'start-analysis' : 'check-access'}
        >
          {phase === 'checking'
            ? '확인 중…'
            : phase === 'starting'
              ? '시작하는 중…'
              : ready
                ? 'Start Analysis →'
                : '비용 확인하기'}
        </button>
        <button className="btn btn-ghost block" type="button" onClick={onDone} data-testid="cancel">
          취소
        </button>
      </div>

      <p className="legend" style={{ marginTop: 28 }}>
        <span className="mk">03</span> — discovery · connect repository
      </p>
    </main>
  );
}
