// docs/mockups/JRN-review-feature.html#STP-decide-diff 의 구현.
//
// 승인하기 전에는 문서가 바뀌지 않는다 — 이 화면이 그리는 것은 제안이고, 적용은
// 서버가 승인을 받은 뒤에 한다. 거부에 사유를 받는 칸은 목업에 없다: 테스트 문서가
// 「사유와 함께 거부」를 요구해(권위 순서상 목업보다 위) 한 칸을 더 그렸고, 그 차이는
// 원장에 등재돼 있다.

import { useEffect, useState } from 'react';
import { decideEdit, getEditProposal } from './api';
import type { EditProposal } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

type Props = {
  id: string;
  proposalId: string;
  onBack: () => void;
  onLeave: () => void;
  onApproved: () => void;
  onRejected: () => void;
};

export function DecideDiff({
  id,
  proposalId,
  onBack,
  onLeave,
  onApproved,
  onRejected,
}: Props) {
  const [proposal, setProposal] = useState<EditProposal | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [rejecting, setRejecting] = useState(false);
  const [reason, setReason] = useState('');
  const [deciding, setDeciding] = useState(false);

  useEffect(() => {
    let active = true;
    getEditProposal(id, proposalId)
      .then((next) => active && setProposal(next))
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id, proposalId]);

  const approve = () => {
    if (deciding) return;
    setDeciding(true);
    setError(null);
    decideEdit(id, proposalId, 'approve')
      .then(onApproved)
      .catch((e: unknown) => {
        setError(messageOf(e));
        setDeciding(false);
      });
  };

  // 첫 탭은 사유 칸을 연다 — 사유 없는 거부는 다음 제안이 무엇을 피할지 말해 주지
  // 못하므로 서버도 받지 않는다.
  const reject = () => {
    if (deciding) return;
    if (!rejecting || reason.trim().length === 0) {
      setRejecting(true);
      return;
    }
    setDeciding(true);
    setError(null);
    decideEdit(id, proposalId, 'reject', reason.trim())
      .then(onRejected)
      .catch((e: unknown) => {
        setError(messageOf(e));
        setDeciding(false);
      });
  };

  return (
    <main className="screen">
      <header className="appbar">
        <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
          ‹
        </button>
        <span className="appbar-title">제안 확인</span>
        <button className="icon-btn" type="button" onClick={onLeave} aria-label="나가기">
          ✕
        </button>
      </header>

      <div style={{ marginTop: 18 }}>
        <h1 className="h-display">이렇게 바꿔도 될까요</h1>
        <p className="h-display-sub">
          승인하기 전에는 문서가 바뀌지 않아요. 무엇을 누가 왜 바꿨는지는 항상 이력에 남습니다.
        </p>
      </div>

      <div className="code-header" style={{ marginTop: 16 }}>
        <span data-testid="diff-target">
          <span>{'시나리오 '}</span>
          <span>{proposal === null ? '' : proposal.scenarioIndex + 1}</span>
        </span>
        <span className="badge success">
          <span className="dot" />
          <span data-testid="diff-changed-lines">{proposal === null ? 0 : proposal.changedLines}</span>
          줄 바뀜
        </span>
      </div>
      <div className="code" data-testid="diff">
        {proposal !== null &&
          proposal.removed.map((line) => (
            <div className="ln" key={line}>
              <span className="del" data-testid="diff-removed">
                {'- '}
                {line}
              </span>
            </div>
          ))}
        {proposal !== null &&
          proposal.added.map((line) => (
            <div className="ln" key={line}>
              <span className="add" data-testid="diff-added">
                {'+ '}
                {line}
              </span>
            </div>
          ))}
      </div>

      <div className="card" style={{ marginTop: 12 }}>
        <span className="caps">이력에 남을 내용</span>
        <div style={{ marginTop: 10 }}>
          <div className="ev">
            <span className="ename">바꾼 주체</span>
            <span className="esrc" data-testid="edit-source">
              사용자 · 도움받아 작성
            </span>
          </div>
          <div className="ev">
            <span className="ename">요청한 문장</span>
            <span className="esrc" data-testid="edit-request-text">
              {proposal === null ? '' : proposal.request}
            </span>
          </div>
        </div>
      </div>

      {rejecting && (
        <div className="card" style={{ marginTop: 12 }}>
          <div className="field">
            <label htmlFor="in-reason">왜 이게 아닌가요</label>
            <textarea
              id="in-reason"
              rows={2}
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              data-testid="reject-reason"
            />
          </div>
        </div>
      )}

      {error !== null && (
        <div className="notice err on" style={{ marginTop: 12 }} data-testid="decide-error">
          {error}
        </div>
      )}

      <div className="stack" style={{ marginTop: 20 }}>
        <button
          className="btn btn-primary block"
          type="button"
          onClick={approve}
          disabled={proposal === null || deciding}
          data-testid="approve-diff"
        >
          이대로 승인
        </button>
        <button
          className="btn btn-secondary block"
          type="button"
          onClick={reject}
          disabled={proposal === null || deciding}
          data-testid="reject-diff"
        >
          내가 말한 건 이게 아니에요
        </button>
      </div>
    </main>
  );
}
