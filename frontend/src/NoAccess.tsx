// 볼 수 없는 저장소 — 목업 JRN-understand-feature.html#no-access 의 상태 블록과, 거기서
// 공유 요청을 보낸 뒤 서는 종료 화면 #END-no-access.
//
// 화면이 아니라 상태 블록이므로 상단에 목업 *매핑* 주석을 붙이지 않는다 — M1 이 화면을
// 발견하는 표식은 `docs/mockups/<파일>.html` 경로 리터럴이고, 그것을 적으면 `STP-open-shared`
// 가 활성 단계가 되어 그 단계의 카피 전량이 M3A 의 분모로 들어온다.
// 대신 `data-testid` 가 목업 `id` 와 짝지어져 M9·M10 의 대조 단위가 된다 — 태그·클래스 집합·
// 인라인 선언·문면이 목업과 같아야 통과하므로, 색이나 간격이 흘러도 게이트가 잡는다.
//
// 여정 JRN-understand-feature §4 분기표: 권한 없는 저장소 링크는 목록·상세 모두 미노출이고
// 여기서 여정이 끝난다. 목록·상세의 미노출은 서버가 이미 세웠고(소유자 스코프 질의의 404),
// 비어 있던 것은 그 사실을 사용자에게 말하는 이 자리 하나였다.
//
// 그 자리에서 앞으로 가는 유일한 경로가 공유 요청이다(AC4.10). 요청은 대상을 조회하지 않는
// 경로로 나가고(`backend/src/access_request.rs`), 그래서 이 화면은 대상이 실재하든 아니든
// 같은 것을 그린다 — 링크 하나로 「이 id 는 존재한다」를 알아낼 수 없다는 AC4.7 의 성질이
// 화면 쪽에서도 유지되는 자리다.

import { useState } from 'react';
import { requestAccess } from './api';

type Props = {
  analysisId: string;
  /** 요청이 접수된 뒤 종료 화면(`AccessRequested`)으로 넘어가는 신호. */
  onRequested: () => void;
};

export function NoAccess({ analysisId, onRequested }: Props) {
  const [sending, setSending] = useState(false);

  // 실패했을 때 띄울 통지를 두지 않는다. 목업은 이 경로의 실패 상태를 그리지 않고
  // AC4.10·시나리오 15 도 요구하지 않으므로, 여기에 통지를 지으면 자매 모델의 편차 원장에
  // 「목업 미갱신」 한 줄을 새로 여는 일이 된다. 대신 실패하면 버튼이 그대로 남아 다시 누를
  // 수 있고, 접수되지 않은 요청을 접수됐다고 말하지 않는다 — 넘어가는 것은 성공했을 때뿐이다.
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

/**
 * 요청을 보낸 뒤의 종료 화면(목업 `#END-no-access`).
 *
 * 앱바를 두지 않는 것이 목업과 같다 — 여정이 여기서 끝나므로 되돌아갈 상위 자리가 없고,
 * 이어지는 경로는 다른 링크를 여는 것 하나다(시나리오 15 의 기대 결과).
 */
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
