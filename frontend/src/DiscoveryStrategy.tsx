// S06 · Discovery Strategy — the real screen behind
// docs/mockups/JRN-discover-features.html#STP-tune-strategy (S06, AC1.3).
//
// The mockup is the SSOT for what this screen says, so the copy below is the
// mockup's copy. What it *does* is AC1.3: the list stage 3 proposed is a draft the
// reviewer can delete from and add to, and only an approved list becomes the next
// stage's input. Nothing here is client-side state — every mutation is a write the
// server answers with the new list, which is why a reload shows the same edits.
//
// One thing the mockup draws that this screen deliberately does not: the five
// example patterns (`src/routes/**/*.ts` and friends). Those are sample data in a
// static prototype; here the list is whatever the model proposed for *this*
// repository, and hard-coding the samples would be a false list. The mockup marks
// them `data-sample` so the copy gate skips them (docs/mockups/README.md "예시값
// 표기 규약") — it used to be carried as a row in docs/doc-tracker.md "알려진
// 목업↔구현 편차" instead, which is no longer needed.

import { useEffect, useState } from 'react';
import { approveDiscoveryStrategy, getDiscoveryStrategy, putDiscoveryStrategy } from './api';
import type { DiscoveryStrategy as Strategy } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

type Props = {
  id: string;
  /** S06 → S04 (back to the run this strategy belongs to). */
  onBack: () => void;
  /**
   * S06 → S07 (AC1.4). The mockup's own wiring: `이 전략으로 후보 뽑기` carries
   * `data-goto="STP-sift-candidates"`, so approving and entering the candidate list
   * are one button. Before approval it approves; after, it is the way through.
   */
  onOpenCandidates: () => void;
};

export function DiscoveryStrategy({ id, onBack, onOpenCandidates }: Props) {
  const [strategy, setStrategy] = useState<Strategy | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [draft, setDraft] = useState('');
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let active = true;
    getDiscoveryStrategy(id)
      .then((s) => active && setStrategy(s))
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id]);

  // Takes the started request rather than a thunk: `() => Promise<Strategy>` would
  // read as product copy to the M3B copy extractor (it scans the text between `>`
  // and `<`), and there is nothing to gain from deferring the call by one tick.
  /** Every mutation goes through the server and renders its answer, never a guess. */
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
        ) : (
          <p className="body sm" style={{ marginTop: 22 }} data-testid="strategy-loading">
            {LOADING}
          </p>
        )}
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
        {strategy.approved && (
          <span className="tag" data-testid="strategy-approved">
            {APPROVED}
          </span>
        )}
        <button
          className="btn btn-primary block"
          type="button"
          disabled={busy || strategy.entries.length === 0}
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
 * Copy the static prototype has no counterpart for — a network wait and the
 * post-approval state. Both are registered in docs/doc-tracker.md "알려진
 * 목업↔구현 편차"; kept as constants so the deviation is one place, not scattered.
 */
const LOADING = '불러오는 중…';
const APPROVED = '승인된 전략이에요';

function Appbar({ onBack }: { onBack: () => void }) {
  return (
    <div className="appbar">
      <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
        ‹
      </button>
      <div>
        <div className="appbar-title">탐색 전략</div>
      </div>
    </div>
  );
}
