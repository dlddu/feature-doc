// Sign In — the standalone screen behind
// docs/mockups/JRN-connect-repo.html#STP-sign-in (AC4.8).
//
// `data-testid="signin"` stays on the primary button: it is the one UI entry
// point sc04-11 drives, and every other spec signs in through `?as=<handle>`.

import { LOGIN_URL } from './api';

export function SignIn({ error }: { error?: string | null }) {
  function signIn() {
    window.location.href = LOGIN_URL;
  }

  return (
    <main className="screen">
      <div className="toprow">
        <span className="brand">
          <span className="mk">●</span> FeatureDoc
        </span>
      </div>

      <div style={{ marginTop: 96 }}>
        <h1 className="h-display">코드가 말해주는 기능 목록</h1>
        <p className="h-display-sub">
          저장소를 연결하면 최종 사용자 관점의 기능 목록을 만들어 드려요. 먼저 이
          인스턴스에서의 당신을 확인할게요.
        </p>
      </div>

      <div className="card row top" style={{ marginTop: 30, gap: 12 }}>
        <span className="ico ico-28">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <circle cx="7" cy="4.6" r="2.4" stroke="currentColor" strokeWidth="1.1" />
            <path
              d="M2.4 12c0-2.3 2.1-3.7 4.6-3.7s4.6 1.4 4.6 3.7"
              stroke="currentColor"
              strokeWidth="1.1"
            />
          </svg>
        </span>
        <p className="body sm grow" style={{ color: 'var(--text-secondary)' }}>
          이 단계에서 받는 권한은 <strong style={{ color: 'var(--text-primary)' }}>신원 확인뿐</strong>
          이에요. 저장소 접근은 다음 화면에서 따로 요청하고, 당신이 고른 저장소에만
          적용됩니다.
        </p>
      </div>

      {error !== undefined && error !== null && (
        <p className="body sm" style={{ marginTop: 14 }} data-testid="signin-error">
          {error}
        </p>
      )}

      <div className="stack" style={{ marginTop: 34 }}>
        <button
          className="btn btn-primary block"
          type="button"
          onClick={signIn}
          data-testid="signin"
        >
          GitHub으로 계속하기
        </button>
      </div>

      <p className="legend" style={{ marginTop: 26 }}>
        <span className="mk">↳</span> 읽기 전용 최소 권한 · 언제든 회수 가능
      </p>
    </main>
  );
}
