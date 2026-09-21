// Analysis in Progress — the real, stateful screen behind
// docs/mockups/JRN-discover-features.html#STP-leave-and-return.
//
// The component holds no progress of its own: a reload is just another read of the
// same server state.

import { useCallback, useEffect, useRef, useState } from 'react';
import { getAnalysis, retryStage } from './api';
import type { AnalysisDetail, Stage } from './api';
import { formatCost, formatDuration } from './format';

const POLL_MS = 2_000;

/** Statuses that can still change on their own — the ones worth polling for. */
const ACTIVE = new Set(['queued', 'running']);

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

function toneOf(status: Stage['status']): string {
  if (status === 'succeeded') return 'done';
  if (status === 'running') return 'active';
  if (status === 'failed') return 'failed';
  return 'todo';
}

// Display titles keyed by the wire `key`. The server's `title` is persisted per analysis at
// enqueue, so rows seeded before a copy change would keep the old text — the screen owns the copy.
const STAGE_TITLES: Record<string, string> = {
  fetch: '저장소 내려받기',
  cross_cutting: '횡단 관심사 추출',
  discovery_strategy: '탐색 전략 생성',
  feature_candidates: 'feature 후보 추출',
  acceptance_dependencies: '인수 시나리오 생성',
};

function titleOf(stage: Stage): string {
  return STAGE_TITLES[stage.key] ?? stage.title;
}

function subOf(stage: Stage): string {
  // The server's reason is not drawn here: the mockup answers a failed stage with
  // one standing sentence (below the list), and 시나리오 6 only asks that the stage
  // can be re-run.
  if (stage.status === 'failed') return '실패했어요';
  if (stage.detail) return stage.detail;
  if (stage.status === 'running') return '진행 중';
  if (stage.status === 'succeeded') return '완료';
  return '대기 중';
}

function elapsedOf(stage: Stage, nowSeconds: number): string {
  if (stage.startedAt === null) return '';
  const end = stage.finishedAt ?? nowSeconds;
  return formatDuration(end - stage.startedAt);
}

type Props = {
  id: string;
  /** Analysis Progress → Home (back, close, or "Run in background" — the job keeps running). */
  onBack: () => void;
  onOpenCrossCutting: () => void;
  onOpenDiff: () => void;
};

export function AnalysisProgress({
  id,
  onBack,
  onOpenCrossCutting,
  onOpenDiff,
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
  // At most one stage is failed at a time — the pipeline stops there.
  const failed = stages.find((stage) => stage.status === 'failed');

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
                <div className="label">{titleOf(stage)}</div>
                <div className="sub" data-testid="stage-sub">
                  {subOf(stage)}
                </div>
              </div>
              <span className="time">{elapsedOf(stage, now)}</span>
            </div>
            {/* A stage that produced a document gets a way into it. Gated on the
                stage having succeeded, so the link never leads to a 404.
                Stage 3 is deliberately not one of them: the mockup walks this
                journey 진행 → 횡단 관심사 → 탐색 전략, so the way in is the
                cross-cutting screen's own CTA. */}
            {stage.key === 'cross_cutting' && stage.status === 'succeeded' && (
              <button
                className="btn btn-secondary block"
                type="button"
                style={{ marginTop: 12 }}
                onClick={onOpenCrossCutting}
                data-testid="open-cross-cutting"
              >
                추출된 횡단 관심사 보기
              </button>
            )}
            {/* 5단계가 쓰기 전에는 이 실행에 견줄 표현 자체가 없다. */}
            {stage.key === 'acceptance_dependencies' && stage.status === 'succeeded' && (
              <button
                className="btn btn-secondary block"
                type="button"
                style={{ marginTop: 12 }}
                onClick={onOpenDiff}
                data-testid="open-diff"
              >
                무엇이 달라졌는지 보기
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
                이 단계만 다시 시도
              </button>
            )}
          </div>
        ))}
      </div>

      {failed !== undefined && (
        <div className="notice err" style={{ marginTop: 16 }} data-testid="stage-failed">
          <strong>{titleOf(failed)}</strong>
          {' 단계가 실패했어요. 앞 단계 결과는 그대로 있으니 이 단계만 다시 돌리면 됩니다.'}
        </div>
      )}

      {analysis.status === 'awaiting_pipeline' && (
        <p className="body sm" style={{ marginTop: 14 }} data-testid="awaiting-pipeline">
          여기까지는 끝났어요. 남은 단계는 검토가 필요해서 기다리고 있습니다.
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
          앱 닫고 나가기
        </button>
      </div>

      <p className="legend" style={{ marginTop: 24 }}>
        <span className="mk">↳</span> 실패한 단계는 그 단계만 재시도합니다 · 누적 비용은 항상 표시
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
