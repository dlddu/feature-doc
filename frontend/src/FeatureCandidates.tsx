// Feature Candidates — the real screen behind
// docs/mockups/JRN-discover-features.html#STP-sift-candidates.
//
// Nothing here is client-side state: every mutation is a write the server answers
// with the new list, which is why a reload shows the same decisions.

import { useEffect, useState } from 'react';
import {
  approveCandidate,
  getCandidates,
  mergeCandidates,
  rejectCandidate,
  renameCandidate,
} from './api';
import type { CandidateList, FeatureCandidate, PreviousRejection } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** Concatenated rather than a template literal: an interpolated literal is not
 *  product copy, but the M3B extractor reads it as one (it scans string literals
 *  whole), which would put a row in the copy ledger that is never true. */
function quotedRejection(prev: PreviousRejection): string {
  const when = new Date(prev.rejectedAt * 1000).toISOString().slice(0, 10);
  return '“' + prev.reason + '” (' + when + ')';
}

type Filter = 'all' | 'undecided' | 'approved' | 'rejected';

type Props = {
  id: string;
  onBack: () => void;
};

export function FeatureCandidates({ id, onBack }: Props) {
  const [list, setList] = useState<CandidateList | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<Filter>('all');
  const [busy, setBusy] = useState(false);
  const [rejecting, setRejecting] = useState<string | null>(null);
  const [reason, setReason] = useState('');
  const [renaming, setRenaming] = useState<string | null>(null);
  const [draftName, setDraftName] = useState('');
  const [picked, setPicked] = useState<string[]>([]);

  useEffect(() => {
    let active = true;
    getCandidates(id)
      .then((l) => active && setList(l))
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id]);

  async function mutate(run: Promise<CandidateList>) {
    setBusy(true);
    setError(null);
    try {
      setList(await run);
      return true;
    } catch (e: unknown) {
      setError(messageOf(e));
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function confirmRejection() {
    if (rejecting === null) return;
    if (await mutate(rejectCandidate(id, rejecting, reason))) {
      setRejecting(null);
      setReason('');
    }
  }

  async function confirmRename() {
    if (renaming === null) return;
    if (await mutate(renameCandidate(id, renaming, draftName))) {
      setRenaming(null);
      setDraftName('');
    }
  }

  async function foldInto(key: string) {
    if (await mutate(mergeCandidates(id, key, picked))) setPicked([]);
  }

  if (list === null) {
    return (
      <main className="screen">
        <Appbar onBack={onBack} />
        {error !== null ? (
          <div className="row" style={{ marginTop: 22 }} data-testid="candidates-error">
            <span className="badge danger">
              <span className="dot" />
              Error
            </span>
            <span className="body sm">{error}</span>
          </div>
        ) : (
          <p className="body sm" style={{ marginTop: 22 }} data-testid="candidates-loading">
            {LOADING}
          </p>
        )}
      </main>
    );
  }

  if (!list.extracted) {
    return (
      <main className="screen">
        <Appbar onBack={onBack} />
        <p className="body sm" style={{ marginTop: 22 }} data-testid="candidates-empty">
          {NOT_EXTRACTED}
        </p>
      </main>
    );
  }

  const shown = list.candidates.filter((c) => {
    if (c.mergedInto !== null) return false;
    return filter === 'all' || c.decision === filter;
  });

  return (
    <main className="screen">
      <Appbar onBack={onBack} />

      <div className="row between" style={{ marginTop: 16, gap: 10 }}>
        <div className="field" style={{ flex: 1, minWidth: 0 }}>
          <label htmlFor="in-filter">Filter</label>
          <select
            id="in-filter"
            value={filter}
            onChange={(e) => setFilter(e.target.value as Filter)}
            data-testid="candidate-filter"
          >
            <option value="all">전체</option>
            <option value="undecided">미결정</option>
            <option value="approved">승인</option>
            <option value="rejected">거부</option>
          </select>
        </div>
        <div style={{ textAlign: 'right' }}>
          <div className="metric" data-testid="undecided-count">
            {list.undecided}
          </div>
          <div className="caps" style={{ marginTop: 6 }}>
            미결정
          </div>
        </div>
      </div>

      {error !== null && (
        <div className="row" style={{ marginTop: 12 }} data-testid="candidates-error">
          <span className="badge danger">
            <span className="dot" />
            Error
          </span>
          <span className="body sm">{error}</span>
        </div>
      )}

      <div style={{ marginTop: 16 }} data-testid="candidate-list">
        {shown.map((candidate) => (
          <Card
            key={candidate.key}
            candidate={candidate}
            busy={busy}
            picked={picked.includes(candidate.key)}
            pickedCount={picked.length}
            renaming={renaming === candidate.key}
            draftName={draftName}
            onDraftName={setDraftName}
            onStartRename={() => {
              setRenaming(candidate.key);
              setDraftName(candidate.name);
            }}
            onCancelRename={() => setRenaming(null)}
            onConfirmRename={() => void confirmRename()}
            onApprove={() => void mutate(approveCandidate(id, candidate.key))}
            onReject={() => {
              setRejecting(candidate.key);
              setReason('');
            }}
            onPick={() =>
              setPicked((keys) =>
                keys.includes(candidate.key)
                  ? keys.filter((k) => k !== candidate.key)
                  : [...keys, candidate.key],
              )
            }
            onFoldInto={() => void foldInto(candidate.key)}
          />
        ))}
      </div>

      {rejecting !== null && (
        <div className="card" style={{ marginTop: 14 }} data-testid="reject-panel">
          <span className="caps">
            거부 사유 — <span>{nameOf(list, rejecting)}</span>
          </span>
          <div className="field" style={{ marginTop: 10 }}>
            <label htmlFor="in-reject-why">Why not a feature</label>
            <textarea
              id="in-reject-why"
              rows={2}
              placeholder="예: 내부 디버그용 엔드포인트라 사용자 기능이 아님"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              data-testid="reject-reason"
            />
          </div>
          {reason.trim() === '' && (
            <div className="notice warn on" style={{ marginTop: 10 }} data-testid="reject-guard">
              사유를 적어야 거부를 확정할 수 있어요. 다음 분석에서 같은 판단을 반복하지 않으려면 한
              줄이라도 남겨 주세요.
            </div>
          )}
          <div className="btn-row" style={{ marginTop: 12 }}>
            <button
              className="btn btn-primary grow"
              type="button"
              disabled={busy || reason.trim() === ''}
              onClick={() => void confirmRejection()}
              data-testid="reject-confirm"
            >
              거부 확정
            </button>
            <button
              className="btn btn-ghost"
              type="button"
              onClick={() => setRejecting(null)}
              data-testid="reject-cancel"
            >
              취소
            </button>
          </div>
        </div>
      )}

      <div className="stack" style={{ marginTop: 20 }}>
        <button
          className="btn btn-ghost block"
          type="button"
          onClick={onBack}
          data-testid="leave-partial"
        >
          여기까지 저장하고 나가기
        </button>
      </div>

      <p className="legend" style={{ marginTop: 20 }}>
        <span className="mk">↳</span> 미결정이 0건이 되면 확정 목록을 만들 수 있어요
      </p>
    </main>
  );
}

type CardProps = {
  candidate: FeatureCandidate;
  busy: boolean;
  picked: boolean;
  pickedCount: number;
  renaming: boolean;
  draftName: string;
  onDraftName: (value: string) => void;
  onStartRename: () => void;
  onCancelRename: () => void;
  onConfirmRename: () => void;
  onApprove: () => void;
  onReject: () => void;
  onPick: () => void;
  onFoldInto: () => void;
};

function Card(p: CardProps) {
  const { candidate } = p;
  const decided = candidate.decision !== 'undecided';
  return (
    <div
      className={`cand${decided ? ` ${candidate.decision}` : ''}`}
      data-testid="candidate"
      data-key={candidate.key}
      data-decision={candidate.decision}
    >
      <span className="cname">{candidate.name}</span>
      <span className="cwhere">{locationOf(candidate)}</span>
      <span className="cwhy">{candidate.rationale}</span>

      {candidate.previouslyRejected !== null && (
        <div className="prev" data-testid="previously-rejected">
          이전 분석에서 <strong>거부</strong>한 항목이에요. 자동으로 다시 채택하지 않았으니 이번에도 직접 결정해 주세요.<span className="prev-quote">{quotedRejection(candidate.previouslyRejected)}</span>
        </div>
      )}

      {decided && <span className="cstate">결정됨</span>}

      {p.renaming ? (
        <div className="field" style={{ marginTop: 10 }}>
          <label htmlFor={`in-rename-${candidate.key}`}>Feature name</label>
          <input
            type="text"
            id={`in-rename-${candidate.key}`}
            value={p.draftName}
            autoComplete="off"
            onChange={(e) => p.onDraftName(e.target.value)}
            data-testid="rename-input"
          />
          <div className="btn-row" style={{ marginTop: 10 }}>
            <button
              className="btn btn-secondary grow"
              type="button"
              disabled={p.busy || p.draftName.trim() === ''}
              onClick={p.onConfirmRename}
              data-testid="rename-confirm"
            >
              이름 저장
            </button>
            <button
              className="btn btn-ghost"
              type="button"
              onClick={p.onCancelRename}
              data-testid="rename-cancel"
            >
              취소
            </button>
          </div>
        </div>
      ) : (
        <div className="cbtns">
          <button
            className="btn btn-secondary"
            type="button"
            disabled={p.busy}
            onClick={p.onApprove}
            data-testid="candidate-approve"
          >
            승인
          </button>
          <button
            className="btn btn-ghost"
            type="button"
            disabled={p.busy}
            onClick={p.onReject}
            data-testid="candidate-reject"
          >
            거부
          </button>
        </div>
      )}

      {!p.renaming && (
        <div className="btn-row" style={{ marginTop: 8 }}>
          <button
            className="btn btn-ghost grow"
            type="button"
            disabled={p.busy}
            onClick={p.onStartRename}
            data-testid="candidate-rename"
          >
            이름 바꾸기
          </button>
          <button
            className="btn btn-ghost grow"
            type="button"
            disabled={p.busy}
            onClick={p.onPick}
            data-testid="candidate-pick"
          >
            {p.picked ? MERGE_PICKED : MERGE_PICK}
          </button>
          {p.pickedCount !== 0 && !p.picked && (
            <button
              className="btn btn-secondary grow"
              type="button"
              disabled={p.busy}
              onClick={p.onFoldInto}
              data-testid="candidate-fold-into"
            >
              {MERGE_INTO}
            </button>
          )}
        </div>
      )}
    </div>
  );
}

/** Concatenated for the same reason as [`quotedRejection`]. */
function locationOf(candidate: FeatureCandidate): string {
  return candidate.symbol === null ? candidate.location : candidate.location + ' · ' + candidate.symbol;
}

function nameOf(list: CandidateList, key: string): string {
  return list.candidates.find((c) => c.key === key)?.name ?? key;
}

const LOADING = '불러오는 중…';
const NOT_EXTRACTED = '후보 추출 단계가 아직 끝나지 않았어요.';
const MERGE_PICK = '합칠 후보로 고르기';
const MERGE_PICKED = '고름 해제';
const MERGE_INTO = '여기에 합치기';

/**
 * The right-hand `icon-btn ghost` is a spacer, not a control — it keeps the title
 * centred under `.appbar`'s `space-between`.
 */
function Appbar({ onBack }: { onBack: () => void }) {
  return (
    <header className="appbar">
      <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
        ‹
      </button>
      <span className="appbar-title">feature 후보</span>
      <span className="icon-btn ghost" aria-hidden="true" />
    </header>
  );
}
