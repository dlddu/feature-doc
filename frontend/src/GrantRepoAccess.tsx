// Grant Repository Access — the screen behind
// docs/mockups/JRN-connect-repo.html#STP-grant-repo-access (AC4.1).
//
// The step the mockup draws under that anchor is **GitHub's own** install-consent
// page (`data-owner="github.com"`): scope radio, repository checkboxes,
// `Install & Authorize`. FeatureDoc does not render it — it hands the browser over
// and waits for the round trip. That structural deviation is the 원장's 제3자 소유
// row, and it is why this screen draws only the part that is ours: what we are
// about to ask for, and the door to GitHub.
//
// Split out of `CredentialsSetup.tsx` by 수렴 슬라이스 ⑦ (rct_20260919-0007). The
// 2026-09-18 authority-order ruling put the screen composition on the mockup's
// side — 권한 부여 and 키 등록 are two steps, so they are two screens. Once the
// installation exists there is nothing left to ask here, so the screen hands off
// to `RegisterLlmKey.tsx` instead of drawing a second state.

import { useEffect, useState } from 'react';
import { getConnection, getInstallUrl, getMe } from './api';
import type { Connection, User } from './api';
import { SignIn } from './SignIn';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

type Props = {
  /** The installation exists — `RegisterLlmKey` owns everything from here. */
  onInstalled: () => void;
};

export function GrantRepoAccess({ onInstalled }: Props) {
  // undefined = still loading the session
  const [me, setMe] = useState<User | null | undefined>(undefined);
  const [connection, setConnection] = useState<Connection | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);

  useEffect(() => {
    void load();
  }, []);

  async function load() {
    try {
      const user = await getMe();
      setMe(user);
      if (user) setConnection(await getConnection());
    } catch (e) {
      setMe(null);
      setLoadError(messageOf(e));
    }
  }

  const installed = connection?.installed ?? false;

  // A user who already installed the App has nothing to do on this step. The
  // mockup goes straight from GitHub's consent page to 키 등록, so the hand-off
  // runs the moment the connection says so rather than behind a button this
  // screen's mockup does not draw.
  useEffect(() => {
    if (installed) onInstalled();
  }, [installed, onInstalled]);

  async function connectApp() {
    setConnecting(true);
    try {
      window.location.href = await getInstallUrl();
    } catch (e) {
      setConnecting(false);
      setLoadError(messageOf(e));
    }
  }

  // Unauthenticated is a different screen, not a state of this one.
  if (me === null) {
    return <SignIn error={loadError} />;
  }

  const permissions = connection?.permissions ?? [];

  return (
    <main className="screen">
      <div className="toprow">
        <span className="brand">
          <span className="mk">●</span> FeatureDoc
        </span>
      </div>

      <p className="ext-note">
        <span className="mk">↳</span>{' '}
        <span>
          이 화면은 <b>GitHub이 소유하고 렌더링합니다.</b> FeatureDoc은 여기를 그리지 않고
          브라우저로 넘겨주며, 설치를 마치면 돌아옵니다.
        </span>
      </p>

      <div style={{ marginTop: 20 }}>
        <h1 className="h-display">들여다볼 범위를 고르세요</h1>
        <p className="h-display-sub">
          FeatureDoc은 <strong style={{ color: 'var(--text-primary)' }}>읽기 전용 최소 권한</strong>
          만 요청하고, 여기서 고른 저장소에만 접근합니다. 설치는 GitHub에서 언제든 해제할 수
          있어요.
        </p>
      </div>

      {me === undefined && (
        <p className="body sm" style={{ marginTop: 28 }}>
          불러오는 중…
        </p>
      )}

      {me && (
        <div className="stack" style={{ marginTop: 20 }}>
          <span className="caps">Repository access</span>
          <div className="tag-row" data-testid="requested-permissions">
            {permissions.map((p) => (
              <span className="tag" key={p}>
                {p}
              </span>
            ))}
          </div>
          <button
            className="btn btn-primary block"
            type="button"
            style={{ marginTop: 14 }}
            onClick={connectApp}
            disabled={connecting}
            data-testid="connect-app"
          >
            Connect GitHub App
          </button>
          {loadError && (
            <p className="body sm" style={{ marginTop: 10 }}>
              {loadError}
            </p>
          )}
        </div>
      )}
    </main>
  );
}
