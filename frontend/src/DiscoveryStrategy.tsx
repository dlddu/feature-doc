// Discovery Strategy — the real screen behind
// docs/mockups/JRN-discover-features.html#STP-tune-strategy.
//
// Nothing here is client-side state: every mutation is a write the server answers
// with the new list, which is why a reload shows the same edits.

import { useEffect, useState } from 'react';
import {
  approveDiscoveryStrategy,
  getCandidates,
  getDiscoveryStrategy,
  putDiscoveryStrategy,
} from './api';
import type { DiscoveryStrategy as Strategy } from './api';

/** Matches `AnalysisProgress` — the other screen that waits on a stage to finish. */
const POLL_MS = 2_000;

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

type Props = {
  id: string;
  onBack: () => void;
  onOpenCandidates: () => void;
};

export function DiscoveryStrategy({ id, onBack, onOpenCandidates }: Props) {
  const [strategy, setStrategy] = useState<Strategy | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [draft, setDraft] = useState('');
  const [busy, setBusy] = useState(false);
  const [extracted, setExtracted] = useState(false);

  useEffect(() => {
    let active = true;
    getDiscoveryStrategy(id)
      .then((s) => active && setStrategy(s))
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id]);

  // The approved strategy re-queues stage 4; the candidates only exist once it has
  // run. Polled rather than read once because the wait spans this screen.
  const approved = strategy?.approved ?? false;
  useEffect(() => {
    if (!approved || extracted) return;
    let active = true;
    async function check() {
      try {
        const list = await getCandidates(id);
        if (active && list.extracted) setExtracted(true);
      } catch {
        // A failed read just means the next tick tries again.
      }
    }
    void check();
    const timer = setInterval(() => void check(), POLL_MS);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [id, approved, extracted]);

  // Takes the started request rather than a thunk: `() => Promise<Strategy>` would
  // read as product copy to the M3B copy extractor (it scans the text between `>`
  // and `<`).
  async function mutate(run: Promise<Strategy>) {
    setBusy(true);
    setError(null);
    try {
      setStrategy(await run);
    } catch (e: unknown) {
      setError(messageOf(e));
    } finally {
      setBusy(false);
    }
  }

  function drop(pattern: string) {
    if (strategy === null) return;
    const kept = strategy.entries.filter((e) => e.pattern !== pattern).map((e) => e.pattern);
    void mutate(putDiscoveryStrategy(id, kept));
  }

  function add() {
    if (strategy === null) return;
    const pattern = draft.trim();
    if (pattern === '') return;
    const next = [...strategy.entries.map((e) => e.pattern), pattern];
    setDraft('');
    void mutate(putDiscoveryStrategy(id, next));
  }

  if (strategy === null) {
    return (
      <main className="screen">
        <Appbar onBack={onBack} />
        {error !== null ? (
          <div className="row" style={{ marginTop: 22 }} data-testid="strategy-error">
            <span className="badge danger">
              <span className="dot" />
              Error
            </span>
            <span className="body sm">{error}</span>
          </div>
        ) : null}
      </main>
    );
  }

  const canAdd = draft.trim() !== '' && !strategy.approved && !busy;

  return (
    <main className="screen">
      <Appbar onBack={onBack} />

      <div style={{ marginTop: 18 }}>
        <h1 className="h-display">어디를 뒤질지 정하기</h1>
        <p className="h-display-sub">
          여기서 승인한 전략이 다음 단계의 후보 품질을 결정합니다. 자기 코드를 아는 사람만 보탤 수
          있는 진입점이 있다면 지금 넣어 주세요.
        </p>
      </div>

      <div className="notice warn on" style={{ marginTop: 16 }} data-testid="strategy-notice">
        승인 전까지 이 전략은 후보 추출에 쓰이지 않아요. 여기서 보탠 항목은 다음 분석에서도 그대로
        참조됩니다.
      </div>

      {error !== null && (
        <div className="row" style={{ marginTop: 12 }} data-testid="strategy-error">
          <span className="badge danger">
            <span className="dot" />
            Error
          </span>
          <span className="body sm">{error}</span>
        </div>
      )}

      <div className="stack" style={{ marginTop: 20 }}>
        <span className="caps">
          탐색 대상 <span data-testid="strategy-count">{strategy.entries.length}</span>건
        </span>
        <div>
          {strategy.entries.map((entry) => (
            <div
              className="strat"
              key={entry.pattern}
              data-testid="strategy-entry"
              data-source={entry.source}
            >
              <span className="sname">{entry.pattern}</span>
              {!strategy.approved && (
                <button
                  className="btn btn-ghost"
                  type="button"
                  disabled={busy}
                  onClick={() => drop(entry.pattern)}
                  data-testid="strategy-drop"
                >
                  지우기
                </button>
              )}
            </div>
          ))}
        </div>
      </div>

      {!strategy.approved && (
        <div className="stack" style={{ marginTop: 18 }}>
          <span className="caps">비표준 진입점 보태기</span>
          <div className="input-row">
            <div className="field" style={{ flex: 1, minWidth: 0 }}>
              <label htmlFor="in-entrypoint">Entry point</label>
              <input
                type="text"
                id="in-entrypoint"
                placeholder="cmd/admin-cli"
                autoComplete="off"
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                data-testid="strategy-input"
              />
            </div>
            <button
              className="btn btn-secondary"
              type="button"
              disabled={!canAdd}
              onClick={add}
              data-testid="strategy-add"
            >
              추가
            </button>
          </div>
          <p className="legend">
            <span className="mk">↳</span> 예: 사내 admin CLI, 배치 스크립트, 웹훅 수신부
          </p>
        </div>
      )}

      <div className="stack" style={{ marginTop: 24 }}>
        {/* One primary CTA, as the mockup has it. Before approval it approves; after,
            it is the way on to the candidates. Waiting is the disabled state, not a
            line of copy: between approval and the extraction finishing there is
            nothing to walk into yet. */}
        <button
          className="btn btn-primary block"
          type="button"
          disabled={
            busy || strategy.entries.length === 0 || (strategy.approved && !extracted)
          }
          onClick={() =>
            strategy.approved
              ? onOpenCandidates()
              : void mutate(approveDiscoveryStrategy(id))
          }
          data-testid={strategy.approved ? 'strategy-open-candidates' : 'strategy-approve'}
        >
          이 전략으로 후보 뽑기
        </button>
      </div>
    </main>
  );
}

/**
 * The right-hand `icon-btn ghost` is a spacer, not a control — without it
 * `.appbar`'s `space-between` pushes the title to the right edge.
 */
function Appbar({ onBack }: { onBack: () => void }) {
  return (
    <header className="appbar">
      <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
        ‹
      </button>
      <span className="appbar-title">탐색 전략</span>
      <span className="icon-btn ghost" aria-hidden="true" />
    </header>
  );
}
