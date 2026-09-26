import { useState } from 'react';
import { requestAccess } from './api';

type Props = {
  analysisId: string;
  onRequested: () => void;
};

export function NoAccess({ analysisId, onRequested }: Props) {
  const [sending, setSending] = useState(false);

  async function send() {
    if (sending) return;
    setSending(true);
    try {
      await requestAccess(analysisId);
      onRequested();
    } catch {
      setSending(false);
    }
  }

  return (
    <div className="notice warn" data-testid="no-access" style={{ marginTop: 12 }}>
      이 저장소는 아직 볼 수 없어요. 목록에도, 상세에도 나오지 않습니다 — 잘못 보내신 링크일 수도 있어요.
      <button
        className="btn btn-secondary"
        data-testid="btn-request-access"
        onClick={() => void send()}
      >
        소유자에게 공유 요청 보내기
      </button>
    </div>
  );
}

export function AccessRequested({ onOpenAnother }: { onOpenAnother: () => void }) {
  return (
    <>
      <div className="card end-card" data-testid="access-requested" style={{ marginTop: 170 }}>
        <div className="em">◌</div>
        <h1 className="h-display" style={{ marginTop: 14 }}>
          아직 볼 수 없는 저장소예요
        </h1>
        <p className="h-display-sub" style={{ marginTop: 8 }}>
          소유자에게 공유 요청을 보냈어요. 허락이 오면 알림이 갑니다 — 그때까지 목록에도 상세에도 나오지 않아요.
        </p>
        <span className="badge warn" style={{ marginTop: 16 }}>
          <span className="dot" />
          공유 요청 보냄
        </span>
      </div>
      <div className="stack" style={{ marginTop: 26 }}>
        <button
          className="btn btn-secondary block"
          data-testid="btn-open-another"
          onClick={onOpenAnother}
        >
          다른 링크 열어보기
        </button>
      </div>
    </>
  );
}
