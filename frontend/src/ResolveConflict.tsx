// docs/mockups/JRN-follow-code-change.html#STP-resolve-conflict 의 구현.
//
// 결정하기 전에는 어느 쪽도 저장되지 않는다 — 고른 것은 화면이 들고 있고, 「결정
// 저장하고 후보 보기」가 서버에 보낸다. 합치기만 예외다: 제안은 서버가 만들어 주소를
// 가지므로(새로고침이 같은 제안을 다시 그린다) 고르는 순간 요청되고, 「이 문장으로
// 확정」이 곧 결정이다.

import { useEffect, useState } from 'react';
import { decideConflict, decideMerge, getConflict, listConflicts, proposeMerge } from './api';
import type { DocConflict } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

type Decision = '' | 'auto' | 'mine' | 'merge';

type Props = {
  id: string;
  conflictId: string;
  /** 부딪힌 문장 → 달라진 것. */
  onBack: () => void;
  /** 「결정 저장하고 후보 보기」 → 기능 후보. */
  onSaved: () => void;
};

/** `내가 고친 것 · 2026-08-24` — the day the reader approved that sentence. */
function dayOf(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toISOString().slice(0, 10);
}

export function ResolveConflict({ id, conflictId, onBack, onSaved }: Props) {
  const [conflict, setConflict] = useState<DocConflict | null>(null);
  const [unresolved, setUnresolved] = useState(0);
  const [decision, setDecision] = useState<Decision>('');
  const [keepMine, setKeepMine] = useState(false);
  const [mergeRejected, setMergeRejected] = useState(false);
  const [left, setLeft] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    Promise.all([getConflict(id, conflictId), listConflicts(id)])
      .then(([next, all]) => {
        if (!active) return;
        setConflict(next);
        setUnresolved(all.open);
        // 결정되지 않은 합친 제안이 서 있으면 그 자리로 돌아온다.
        if (next.mergeProposal?.status === 'proposed') setDecision('merge');
      })
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id, conflictId]);

  const decided = conflict !== null && conflict.status !== 'open';
  const proposal =
    conflict?.mergeProposal?.status === 'proposed' ? conflict.mergeProposal : null;

  const choose = (next: Decision) => {
    if (busy || decided) return;
    setDecision(next);
    setMergeRejected(false);
    setLeft(false);
    if (next !== 'merge' || proposal !== null) return;
    setBusy(true);
    setError(null);
    proposeMerge(id, conflictId, keepMine)
      .then((updated) => setConflict(updated))
      .catch((e: unknown) => {
        setError(messageOf(e));
        setDecision('');
      })
      .finally(() => setBusy(false));
  };

  const approveMerge = () => {
    if (busy) return;
    setBusy(true);
    setError(null);
    decideMerge(id, conflictId, 'approve')
      .then((updated) => {
        setConflict(updated);
        setUnresolved((n) => Math.max(0, n - 1));
      })
      .catch((e: unknown) => setError(messageOf(e)))
      .finally(() => setBusy(false));
  };

  const rejectMerge = () => {
    if (busy) return;
    setBusy(true);
    setError(null);
    decideMerge(id, conflictId, 'reject')
      .then((updated) => {
        setConflict(updated);
        setDecision('');
        setMergeRejected(true);
      })
      .catch((e: unknown) => setError(messageOf(e)))
      .finally(() => setBusy(false));
  };

  const save = () => {
    if (busy || conflict === null) return;
    if (decided) {
      onSaved();
      return;
    }
    if (decision !== 'auto' && decision !== 'mine') return;
    setBusy(true);
    setError(null);
    decideConflict(id, conflictId, decision)
      .then(onSaved)
      .catch((e: unknown) => {
        setError(messageOf(e));
        setBusy(false);
      });
  };

  const canSave = conflict !== null && !busy && (decided || decision === 'auto' || decision === 'mine');
  const mineText = conflict === null ? '' : conflict.mine.map((s) => s.then).join(' ');
  const lastMine = conflict === null ? '' : (conflict.mine[conflict.mine.length - 1]?.then ?? '');

  return (
    <main className="screen">
      <header className="appbar">
        <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
          ‹
        </button>
        <span className="appbar-title" data-testid="conflict-feature">
          {conflict === null ? '' : conflict.featureName}
        </span>
        <span className="icon-btn ghost" aria-hidden="true" />
      </header>

      {conflict === null && (
        <p className="body sm" style={{ marginTop: 22 }} data-testid="conflict-loading">
          {error ?? LOADING}
        </p>
      )}

      {conflict !== null && (
        <>
          <div className="card" style={{ marginTop: 16 }}>
            <span className="caps">부딪힌 문장</span>
            <div className="side mine" style={{ marginTop: 10 }}>
              <span className="sname">
                내가 고친 것 · <span data-testid="conflict-mine-day">{dayOf(conflict.mineDecidedAt)}</span>
              </span>
              <span className="stext" data-testid="conflict-mine">
                {mineText}
              </span>
            </div>
            <div className="side" style={{ marginTop: 10 }}>
              <span className="sname">이번 자동 결과</span>
              <span className="stext" data-testid="conflict-auto">
                {conflict.auto.then}
              </span>
            </div>
          </div>

          <div className="stack" style={{ marginTop: 18 }}>
            <span className="caps">어느 쪽을 살릴까요</span>
            <label className="choice" htmlFor="dec-auto">
              <input
                type="radio"
                id="dec-auto"
                name="decision"
                value="auto"
                checked={decision === 'auto'}
                disabled={decided || busy}
                onChange={() => choose('auto')}
                data-testid="decide-auto"
              />
              <span>
                <span className="ct">자동 결과로 바꾸기</span>
                <span className="cs">이번 코드 변경을 그대로 반영합니다. 내가 고친 문장은 이력에 남습니다.</span>
              </span>
            </label>
            <label className="choice" htmlFor="dec-mine">
              <input
                type="radio"
                id="dec-mine"
                name="decision"
                value="mine"
                checked={decision === 'mine'}
                disabled={decided || busy}
                onChange={() => choose('mine')}
                data-testid="decide-mine"
              />
              <span>
                <span className="ct">내 문장 그대로 두기</span>
                <span className="cs">자동 결과는 버립니다. 다음 재분석에서 다시 제안될 수 있어요.</span>
              </span>
            </label>
            <label className="choice" htmlFor="dec-merge">
              <input
                type="radio"
                id="dec-merge"
                name="decision"
                value="merge"
                checked={decision === 'merge'}
                disabled={decided || busy}
                onChange={() => choose('merge')}
                data-testid="decide-merge"
              />
              <span>
                <span className="ct">둘을 합쳐 보기</span>
                <span className="cs">내가 보탠 내용을 살린 채 새 규칙만 얹은 문장을 제안받습니다.</span>
              </span>
            </label>
          </div>

          <label className="choice" htmlFor="in-keep-mine" style={{ marginTop: 10 }}>
            <input
              type="checkbox"
              id="in-keep-mine"
              checked={keepMine}
              disabled={decided || busy}
              onChange={(e) => setKeepMine(e.target.checked)}
              data-testid="keep-mine"
            />
            <span>
              <span className="ct">내가 덧붙인 문단은 건드리지 않기</span>
              <span className="cs">합칠 때 그 문단을 그대로 두고 새 규칙만 덧붙입니다.</span>
            </span>
          </label>

          {keepMine && (
            <div className="notice ok on" style={{ marginTop: 10 }} data-testid="keep-preview">
              보존할 문단: <strong data-testid="keep-preview-text">{lastMine}</strong>
            </div>
          )}

          {proposal !== null && !decided && (
            <div className="card" style={{ marginTop: 14 }} data-testid="merge-panel">
              <span className="caps">합친 문장 제안</span>
              <div className="code" style={{ marginTop: 10 }}>
                {proposal.removed.map((line, index) => (
                  <div className="ln" key={`r${index}`}>
                    <span className="n">{index + 1}</span>
                    <span className="del" data-testid="merge-removed">
                      {'− '}
                      {line}
                    </span>
                  </div>
                ))}
                {proposal.added.map((line, index) => (
                  <div className="ln" key={`a${index}`}>
                    <span className="n">{proposal.removed.length + index + 1}</span>
                    <span className="add" data-testid="merge-added">
                      {'+ '}
                      {line}
                    </span>
                  </div>
                ))}
              </div>
              <div className="btn-row" style={{ marginTop: 12 }}>
                <button
                  className="btn btn-primary grow"
                  type="button"
                  onClick={approveMerge}
                  disabled={busy}
                  data-testid="approve-merge"
                >
                  이 문장으로 확정
                </button>
                <button
                  className="btn btn-ghost"
                  type="button"
                  onClick={rejectMerge}
                  disabled={busy}
                  data-testid="reject-merge"
                >
                  마음에 안 들어요
                </button>
              </div>
            </div>
          )}

          {mergeRejected && (
            <div className="notice warn on" style={{ marginTop: 12 }} data-testid="merge-rejected">
              합친 문장을 버렸어요. 위에서 다시 고르거나, 한 번 더 합쳐 볼 수 있습니다. 그때까지 이 충돌은 미해소로 남습니다.
            </div>
          )}

          {left && (
            <div className="notice warn on" style={{ marginTop: 12 }} data-testid="unresolved-note">
              <span>결정하지 않고 두었어요. 미해소 </span>
              <strong data-testid="unresolved-count">{unresolved}</strong>
              <span>건은 다음 재분석에서도 이 표시 그대로 남습니다. 어느 쪽도 자동으로 덮어쓰지 않아요.</span>
            </div>
          )}

          {error !== null && (
            <div className="notice err on" style={{ marginTop: 12 }} data-testid="conflict-error">
              {error}
            </div>
          )}

          <div className="stack" style={{ marginTop: 22 }}>
            <button
              className="btn btn-primary block"
              type="button"
              onClick={save}
              disabled={!canSave}
              data-testid="save-decision"
            >
              결정 저장하고 후보 보기
            </button>
            <button
              className="btn btn-ghost block"
              type="button"
              onClick={() => setLeft(true)}
              disabled={decided}
              data-testid="leave-conflict"
            >
              지금은 결정하지 않기
            </button>
          </div>

          <p className="legend" style={{ marginTop: 20 }}>
            <span className="mk">↳</span> 결정하기 전에는 어느 쪽도 저장되지 않아요
          </p>
        </>
      )}
    </main>
  );
}

const LOADING = '불러오는 중…';
