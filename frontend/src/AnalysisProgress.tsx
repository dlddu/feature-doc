// S04 · Analysis in Progress — the real, stateful screen behind
// docs/mockups/JRN-discover-features.html#STP-leave-and-return (AC1.5). The standalone
// s04-*.html mockup was absorbed when JRN-follow-code-change migrated (2026-09-01); the
// journey page's step anchor is the screen's public path now.
//
// Every figure on this screen comes from `GET /api/analyses/{id}`: the stage rows
// the worker reports into (AC4.5 seeded them at enqueue), and nothing else. The
// component holds no progress of its own, which is what makes test/01 시나리오 5
// ("앱을 종료했다 다시 열어도 같은 진행률") true by construction — a reload is just
// another read of the same server state.
//
// Two mockup elements are deliberately not reproduced as drawn:
//   · "LLM Spend $0.32 of est. $0.80 / Calls 47" — the *measured* spend needs
//     per-call accounting, which is AC4.6 (slice 7). Showing an invented number
//     would be worse than showing the estimate and saying the actual is pending.
//   · "run #14" — analyses have no run counter; the sub-line carries the branch and
//     the job's short id instead, which is what actually identifies this run.
// Both are registered in docs/doc-tracker.md "알려진 목업↔구현 편차".

import { useCallback, useEffect, useRef, useState } from 'react';
import { getAnalysis, retryStage } from './api';
import type { AnalysisDetail, Stage } from './api';
import { formatCost, formatDuration } from './format';

/** How often an unfinished analysis is re-read (async progress, AC1.5). */
const POLL_MS = 2_000;

/** Statuses that can still change on their own — the ones worth polling for. */
const ACTIVE = new Set(['queued', 'running']);

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** The mockup's three step tones, plus the failure the mockup never shows. */
function toneOf(status: Stage['status']): string {
  if (status === 'succeeded') return 'done';
  if (status === 'running') return 'active';
  if (status === 'failed') return 'failed';
  return 'todo';
}

/** The one-liner under a step label: what it measured, or why it stopped. */
function subOf(stage: Stage): string {
  if (stage.status === 'failed') return stage.error ?? '실패했어요';
  if (stage.detail) return stage.detail;
  if (stage.status === 'running') return '진행 중…';
  if (stage.status === 'succeeded') return '완료';
  return '대기 중';
}

/** Wall time a step took, or has been taking. Blank until it starts. */
function elapsedOf(stage: Stage, nowSeconds: number): string {
  if (stage.startedAt === null) return '';
  const end = stage.finishedAt ?? nowSeconds;
  return formatDuration(end - stage.startedAt);
}

type Props = {
  id: string;
  /** S04 → S02 (back, close, or "Run in background" — the job keeps running). */
  onBack: () => void;
  /** S04 → S05, offered once the cross-cutting stage has produced its document. */
  onOpenCrossCutting: () => void;
  /** S04 → S06, offered once stage 3 has proposed a strategy to review (AC1.3). */
  onOpenDiscoveryStrategy: () => void;
};

export function AnalysisProgress({
  id,
  onBack,
  onOpenCrossCutting,
  onOpenDiscoveryStrategy,
}: Props) {
  const [analysis, setAnalysis] = useState<AnalysisDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [retrying, setRetrying] = useState<string | null>(null);
  // Re-rendered on the poll tick so a running step's elapsed time keeps moving.
  const [now, setNow] = useState(() => Math.floor(Date.now() / 1000));
  // Read inside the interval without making it a dependency (which would restart
  // the timer on every tick).
  const status = useRef<string | null>(null);
  status.current = analysis?.status ?? null;

  const load = useCallback(async () => {
    try {
      setAnalysis(await getAnalysis(id));
      setError(null);
    } catch (e) {
      setError(messageOf(e));
    }
  }, [id]);

  useEffect(() => {
    void load();
    const timer = setInterval(() => {
      setNow(Math.floor(Date.now() / 1000));
      if (status.current === null || ACTIVE.has(status.current)) void load();
    }, POLL_MS);
    return () => clearInterval(timer);
  }, [load]);

  async function retry(stageKey: string) {
    setRetrying(stageKey);
    try {
      setAnalysis(await retryStage(id, stageKey));
      setError(null);
    } catch (e) {
      setError(messageOf(e));
    } finally {
      setRetrying(null);
    }
  }

  if (error && analysis === null) {
    return (
      <main className="screen">
        <Appbar title="Analysis" sub="" onBack={onBack} />
        <div className="row" style={{ marginTop: 22 }} data-testid="progress-error">
          <span className="badge danger">
            <span className="dot" />
            Error
          </span>
          <span className="body sm">{error}</span>
        </div>
      </main>
    );
  }

  if (analysis === null) {
    return (
      <main className="screen">
        <Appbar title="Analysis" sub="" onBack={onBack} />
        <p className="body sm" style={{ marginTop: 22 }} data-testid="progress-loading">
          불러오는 중…
        </p>
      </main>
    );
  }

  const { stages, stagesDone, stagesTotal } = analysis;
  const percent = stagesTotal === 0 ? 0 : Math.round((stagesDone / stagesTotal) * 100);

  return (
    <main className="screen">
      <Appbar
        title={analysis.repoName}
        sub={`${analysis.branch} · run ${analysis.id.slice(0, 8)}`}
        onBack={onBack}
      />

      <div style={{ display: 'flex', justifyContent: 'center', marginTop: 24 }}>
        <ProgressRing percent={percent} />
      </div>

      <div className="section-title" style={{ marginTop: 34 }}>
        <span>Pipeline</span>
        <span className="count" data-testid="pipeline-count">
          {stagesDone} of {stagesTotal}
        </span>
      </div>

      <div className="stack" style={{ marginTop: 12 }}>
        {stages.map((stage) => (
          // `.card` wraps `.step` (the mockup puts both on one element) so a failed
          // step can carry its retry action underneath the row.
          <div className="card" key={stage.key} data-testid="stage" data-stage={stage.key}>
            <div className={`step ${toneOf(stage.status)}`}>
              <span className="ic">{stage.status === 'succeeded' && <CheckIcon />}</span>
              <div className="body-col">
                <div className="label">{stage.title}</div>
                <div className="sub" data-testid="stage-sub">
                  {subOf(stage)}
                </div>
              </div>
              <span className="time">{elapsedOf(stage, now)}</span>
            </div>
            {/* A stage that produced a document gets a way into it. Gated on the
                stage having succeeded, so the link never leads to a 404. */}
            {stage.key === 'discovery_strategy' && stage.status === 'succeeded' && (
              <button
                className="btn btn-secondary block"
                type="button"
                style={{ marginTop: 12 }}
                onClick={onOpenDiscoveryStrategy}
                data-testid="open-discovery-strategy"
              >
                탐색 전략 검토하기
              </button>
            )}
            {stage.key === 'cross_cutting' && stage.status === 'succeeded' && (
              <button
                className="btn btn-secondary block"
                type="button"
                style={{ marginTop: 12 }}
                onClick={onOpenCrossCutting}
                data-testid="open-cross-cutting"
              >
                횡단 관심사 문서 보기
              </button>
            )}
            {stage.status === 'failed' && (
              <button
                className="btn btn-secondary block"
                type="button"
                style={{ marginTop: 12 }}
                disabled={retrying !== null}
                onClick={() => void retry(stage.key)}
                data-testid="retry"
              >
                {retrying === stage.key ? '다시 시도하는 중…' : '이 단계만 다시 시도'}
              </button>
            )}
          </div>
        ))}
      </div>

      {analysis.status === 'awaiting_pipeline' && (
        <p className="body sm" style={{ marginTop: 14 }} data-testid="awaiting-pipeline">
          구현된 단계는 모두 끝났어요. 남은 단계는 분석 파이프라인 슬라이스에서 실행됩니다.
        </p>
      )}

      {error && (
        <div className="row" style={{ marginTop: 14 }} data-testid="progress-error">
          <span className="badge danger">
            <span className="dot" />
            Error
          </span>
          <span className="body sm">{error}</span>
        </div>
      )}

      {/* Estimates, labelled as estimates: the measured spend is AC4.6 (slice 7). */}
      <div className="card row between" style={{ marginTop: 14 }} data-testid="spend">
        <div>
          <div className="caps">Est. LLM Spend</div>
          <div className="row" style={{ gap: 8, marginTop: 8 }}>
            <span className="metric" style={{ fontSize: 15 }}>
              {formatCost(analysis.estCostCents)}
            </span>
            <span className="meta">실측 누적은 아직 계측 전</span>
          </div>
        </div>
        <div style={{ textAlign: 'right' }}>
          <div className="caps">Est. Calls</div>
          <div className="metric" style={{ fontSize: 15, marginTop: 8 }}>
            {analysis.estLlmCalls}
          </div>
        </div>
      </div>

      <div className="btn-row" style={{ marginTop: 18 }}>
        <button
          className="btn btn-secondary grow"
          type="button"
          onClick={onBack}
          data-testid="run-in-background"
        >
          Run in background
        </button>
      </div>

      <p className="legend" style={{ marginTop: 24 }}>
        <span className="mk">02</span> — discovery · async progress
      </p>
    </main>
  );
}

function Appbar({ title, sub, onBack }: { title: string; sub: string; onBack: () => void }) {
  return (
    <header className="appbar">
      <button
        className="icon-btn"
        type="button"
        aria-label="back"
        onClick={onBack}
        data-testid="back"
      >
        ‹
      </button>
      <div className="grow">
        <div className="appbar-title" data-testid="progress-title">
          {title}
        </div>
        {sub && (
          <div className="appbar-sub" data-testid="progress-sub">
            {sub}
          </div>
        )}
      </div>
      <span className="icon-btn ghost" aria-hidden="true" />
    </header>
  );
}

/** The mockup's headline figure: an arc of the circle plus the percentage. */
function ProgressRing({ percent }: { percent: number }) {
  const radius = 62;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference * (1 - Math.min(100, Math.max(0, percent)) / 100);
  return (
    <div className="ring">
      <svg width="148" height="148" viewBox="0 0 148 148">
        <circle cx="74" cy="74" r={radius} fill="none" stroke="var(--border-default)" strokeWidth="2" />
        <circle
          cx="74"
          cy="74"
          r={radius}
          fill="none"
          stroke="var(--text-primary)"
          strokeWidth="2"
          strokeLinecap="round"
          strokeDasharray={circumference.toFixed(1)}
          strokeDashoffset={offset.toFixed(1)}
          transform="rotate(-90 74 74)"
        />
      </svg>
      <div className="readout">
        <span className="pct" data-testid="progress-percent">
          {percent}
        </span>
        <span className="caps" style={{ marginTop: 3, fontSize: 8, letterSpacing: '0.1em' }}>
          Percent Complete
        </span>
      </div>
    </div>
  );
}

function CheckIcon() {
  return (
    <svg width="11" height="11" viewBox="0 0 11 11" fill="none" aria-hidden="true">
      <path
        d="M2 5.6L4.4 8L9 3"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
