// docs/mockups/JRN-review-feature.html#STP-request-edit 의 구현.
//
// 한 줄을 받아 제안 하나를 만드는 화면. 무엇을 고칠지(대상)와 무엇을 피해야 하는지
// (거부 이력)는 둘 다 서버가 준다 — 화면이 기억하는 것은 사람이 지금 치고 있는 문장뿐이다.

import { useEffect, useState } from 'react';
import { getEditContext, proposeEdit } from './api';
import type { EditContext } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

type Props = {
  id: string;
  featureKey: string;
  scenarioIndex: number;
  onBack: () => void;
  onLeave: () => void;
  onProposed: (proposalId: string) => void;
};

export function RequestEdit({
  id,
  featureKey,
  scenarioIndex,
  onBack,
  onLeave,
  onProposed,
}: Props) {
  const [context, setContext] = useState<EditContext | null>(null);
  const [request, setRequest] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [sending, setSending] = useState(false);

  useEffect(() => {
    let active = true;
    getEditContext(id, featureKey, scenarioIndex)
      .then((next) => active && setContext(next))
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id, featureKey, scenarioIndex]);

  const empty = request.trim().length === 0;

  // 보내기 전에 화면이 막는 이유는 서버가 같은 요청을 거절하기 때문이 아니라, 빈
  // 요청으로 사람의 키를 쓰는 호출을 하지 않기 위해서다.
  const send = () => {
    if (empty || context === null || sending) return;
    setSending(true);
    setError(null);
    proposeEdit(id, featureKey, scenarioIndex, request.trim())
      .then((proposal) => onProposed(proposal.id))
      .catch((e: unknown) => {
        setError(messageOf(e));
        setSending(false);
      });
  };

  return (
    <main className="screen">
      <header className="appbar">
        <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
          ‹
        </button>
        <span className="appbar-title">편집 요청</span>
        <button className="icon-btn" type="button" onClick={onLeave} aria-label="나가기">
          ✕
        </button>
      </header>

      <div style={{ marginTop: 18 }}>
        <h1 className="h-display">뭘 고쳤으면 하는지 한 줄로</h1>
        <p className="h-display-sub">
          직접 쓰지 않아도 돼요. 읽던 자리에서 바로 부탁하면 고친 문장을 만들어 보여 드립니다.
        </p>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <span className="caps">고칠 대상</span>
        <p className="body sm" style={{ marginTop: 8 }} data-testid="edit-target">
          <span>{'시나리오 '}</span>
          <span data-testid="edit-target-index">{scenarioIndex + 1}</span>
          <span>{' · '}</span>
          <span data-testid="edit-target-when">{context === null ? '' : context.target.when}</span>
        </p>
      </div>

      <div className="card" style={{ marginTop: 12 }}>
        <div className="field">
          <label htmlFor="in-request">이렇게 고쳐 주세요</label>
          <textarea
            id="in-request"
            rows={3}
            value={request}
            onChange={(e) => setRequest(e.target.value)}
            placeholder="예: 왜 저장이 안 되는지 한 문장으로 더 분명하게"
            data-testid="edit-request"
          />
        </div>
        {empty && (
          <div className="notice warn on" style={{ marginTop: 10 }} data-testid="request-empty">
            무엇을 고칠지 한 줄은 있어야 요청을 보낼 수 있어요.
          </div>
        )}
        {error !== null && (
          <div className="notice err on" style={{ marginTop: 10 }} data-testid="request-error">
            {error}
          </div>
        )}
      </div>

      {context !== null && context.rejectedCount > 0 && (
        <div className="notice info on" style={{ marginTop: 12 }} data-testid="rejected-note">
          <span>지난번에 거부하신 제안이 </span>
          <strong data-testid="rejected-count">{context.rejectedCount}</strong>
          <span>건 있어요. 같은 방향은 다시 제안하지 않습니다 — </span>
          <span data-testid="rejected-reason">{context.rejectedReason}</span>
        </div>
      )}

      <div className="stack" style={{ marginTop: 20 }}>
        <button
          className="btn btn-primary block"
          type="button"
          onClick={send}
          disabled={empty || context === null || sending}
          data-testid="send-request"
        >
          이 요청 보내기
        </button>
      </div>
    </main>
  );
}
