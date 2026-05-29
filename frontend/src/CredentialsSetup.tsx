import { useCallback, useEffect, useState } from 'react';
import {
  ApiError,
  getConnection,
  getInstallUrl,
  getMe,
  listKeys,
  loginHref,
  registerKey,
  revokeKey,
  type Connection,
  type LlmKey,
  type Me,
  type Provider,
} from './api';

const PROVIDERS: { id: Provider; label: string; hint: string }[] = [
  { id: 'anthropic', label: 'Anthropic', hint: 'sk-ant-…' },
  { id: 'openai', label: 'OpenAI', hint: 'sk-…' },
  { id: 'google', label: 'Google', hint: 'AIza…' },
];

type Boot =
  | { kind: 'loading' }
  | { kind: 'error'; detail: string }
  | { kind: 'signed-out' }
  | { kind: 'ready'; me: Me };

type SubmitState =
  | { kind: 'idle' }
  | { kind: 'validating' }
  | { kind: 'error'; detail: string };

const RevealIcon = () => (
  <svg width="13" height="13" viewBox="0 0 13 13" fill="none">
    <path
      d="M1 6.5C2.4 3.7 4.3 2.3 6.5 2.3S10.6 3.7 12 6.5C10.6 9.3 8.7 10.7 6.5 10.7S2.4 9.3 1 6.5Z"
      stroke="currentColor"
      strokeWidth="1.1"
    />
    <circle cx="6.5" cy="6.5" r="1.8" stroke="currentColor" strokeWidth="1.1" />
  </svg>
);

const LockIcon = () => (
  <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
    <rect x="2.5" y="6" width="9" height="6.5" rx="1.4" stroke="currentColor" strokeWidth="1.1" />
    <path d="M4.3 6V4.2A2.7 2.7 0 0 1 9.7 4.2V6" stroke="currentColor" strokeWidth="1.1" />
  </svg>
);

export function CredentialsSetup() {
  const [boot, setBoot] = useState<Boot>({ kind: 'loading' });
  const [connection, setConnection] = useState<Connection | null>(null);
  const [keys, setKeys] = useState<LlmKey[]>([]);
  const [provider, setProvider] = useState<Provider>('anthropic');
  const [keyInput, setKeyInput] = useState('');
  const [reveal, setReveal] = useState(false);
  const [submit, setSubmit] = useState<SubmitState>({ kind: 'idle' });

  const loadSignedInData = useCallback(async () => {
    const [conn, ks] = await Promise.all([getConnection(), listKeys()]);
    setConnection(conn);
    setKeys(ks);
  }, []);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const me = await getMe();
        if (cancelled) return;
        if (!me) {
          setBoot({ kind: 'signed-out' });
          return;
        }
        await loadSignedInData();
        if (!cancelled) setBoot({ kind: 'ready', me });
      } catch (err) {
        if (cancelled) return;
        setBoot({
          kind: 'error',
          detail: err instanceof Error ? err.message : String(err),
        });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [loadSignedInData]);

  const activeKey = keys.find((k) => k.provider === provider && k.status === 'active');

  const onConnect = useCallback(async () => {
    try {
      const { url } = await getInstallUrl();
      window.location.assign(url);
    } catch (err) {
      setBoot({
        kind: 'error',
        detail: err instanceof Error ? err.message : String(err),
      });
    }
  }, []);

  const onRegister = useCallback(async () => {
    setSubmit({ kind: 'validating' });
    try {
      await registerKey(provider, keyInput.trim());
      setKeyInput('');
      setReveal(false);
      setSubmit({ kind: 'idle' });
      await loadSignedInData();
    } catch (err) {
      const detail =
        err instanceof ApiError ? err.message : err instanceof Error ? err.message : String(err);
      setSubmit({ kind: 'error', detail });
    }
  }, [provider, keyInput, loadSignedInData]);

  const onRevoke = useCallback(
    async (id: string) => {
      await revokeKey(id);
      await loadSignedInData();
    },
    [loadSignedInData],
  );

  // ── render ────────────────────────────────────────────────
  if (boot.kind === 'loading') {
    return (
      <main className="screen">
        <p className="body-small">Loading…</p>
      </main>
    );
  }

  if (boot.kind === 'error') {
    return (
      <main className="screen">
        <p className="body-small">Could not reach backend: {boot.detail}</p>
      </main>
    );
  }

  return (
    <main className="screen">
      <div className="toprow">
        <span className="brand">
          <span className="mk">●</span> FeatureDoc
        </span>
        <span className="stepcount">Step 1 / 1</span>
      </div>

      <div style={{ marginTop: 38 }}>
        <h1 className="h-display">Bring your own keys</h1>
        <p className="h-display-sub">
          모든 LLM 호출과 GitHub 접근은 당신의 자격증명으로 수행돼요. FeatureDoc은 키를 대신
          보관할 뿐 소유하지 않습니다.
        </p>
      </div>

      {boot.kind === 'signed-out' ? (
        <div className="stack-10" style={{ marginTop: 28 }}>
          <a className="btn btn-primary block" href={loginHref} data-testid="signin-btn">
            Sign in with GitHub
          </a>
          <p className="caps" style={{ textAlign: 'center' }}>
            로그인 후 저장소 연결과 키 등록을 진행합니다
          </p>
        </div>
      ) : (
        <SignedIn
          connection={connection}
          activeKey={activeKey}
          provider={provider}
          setProvider={setProvider}
          keyInput={keyInput}
          setKeyInput={setKeyInput}
          reveal={reveal}
          setReveal={setReveal}
          submit={submit}
          onConnect={onConnect}
          onRegister={onRegister}
          onRevoke={onRevoke}
        />
      )}

      <p className="legend" style={{ marginTop: 28 }}>
        <span className="mk">01</span> — onboarding · single step
      </p>
    </main>
  );
}

function SignedIn(props: {
  connection: Connection | null;
  activeKey: LlmKey | undefined;
  provider: Provider;
  setProvider: (p: Provider) => void;
  keyInput: string;
  setKeyInput: (v: string) => void;
  reveal: boolean;
  setReveal: (v: boolean) => void;
  submit: SubmitState;
  onConnect: () => void;
  onRegister: () => void;
  onRevoke: (id: string) => void;
}) {
  const {
    connection,
    activeKey,
    provider,
    setProvider,
    keyInput,
    setKeyInput,
    reveal,
    setReveal,
    submit,
    onConnect,
    onRegister,
    onRevoke,
  } = props;

  const installed = connection?.installed === true;
  const ready = installed && !!activeKey;

  return (
    <>
      {/* ── GitHub App connection ───────────────────────────── */}
      <div className="stack-10" style={{ marginTop: 28 }} data-testid="github-section">
        {installed && connection?.installed ? (
          <div className="input">
            <span className="field-action badge success" data-testid="github-status">
              <span className="dot" />
              Installed
            </span>
            <span className="lbl">GitHub App</span>
            <span className="val">
              {connection.account} · {connection.repo_count} repositories
            </span>
          </div>
        ) : (
          <div className="input">
            <span className="lbl">GitHub App</span>
            <span className="val placeholder" data-testid="github-status">
              Not connected
            </span>
          </div>
        )}

        <div className="tag-row">
          {(connection?.permissions ?? []).map((p) => (
            <span className="tag" key={p}>
              {p}
            </span>
          ))}
        </div>

        {!installed && (
          <button className="btn btn-secondary block" onClick={onConnect} data-testid="connect-btn">
            Connect GitHub App
          </button>
        )}
      </div>

      {/* ── LLM provider ────────────────────────────────────── */}
      <div className="card stack" style={{ marginTop: 18 }}>
        <span className="caps">LLM Provider</span>
        <div className="segment" style={{ marginTop: 4 }}>
          {PROVIDERS.map((p) => (
            <button
              key={p.id}
              className={`seg${provider === p.id ? ' active' : ''}`}
              onClick={() => setProvider(p.id)}
              data-testid={`provider-${p.id}`}
            >
              {p.label}
            </button>
          ))}
        </div>
      </div>

      {/* ── API key entry / display ─────────────────────────── */}
      {activeKey ? (
        <div className="input" style={{ marginTop: 10 }}>
          <span className="field-action badge success" data-testid="key-status">
            <span className="dot" />
            Active
          </span>
          <span className="lbl">API Key</span>
          <span className="val" data-testid="key-masked">
            {activeKey.masked}
          </span>
          <button
            className="btn btn-ghost"
            style={{ height: 28, padding: '0 4px', marginTop: 8 }}
            onClick={() => onRevoke(activeKey.id)}
            data-testid="revoke-btn"
          >
            Revoke key
          </button>
        </div>
      ) : (
        <div className="stack" style={{ marginTop: 10 }}>
          <div className="input">
            <span
              className="field-action ico ico-24 btn-like"
              role="button"
              aria-label="reveal"
              onClick={() => setReveal(!reveal)}
              data-testid="reveal-toggle"
            >
              <RevealIcon />
            </span>
            <span className="lbl">API Key</span>
            <input
              className="entry"
              type={reveal ? 'text' : 'password'}
              value={keyInput}
              onChange={(e) => setKeyInput(e.target.value)}
              placeholder={PROVIDERS.find((p) => p.id === provider)?.hint}
              autoComplete="off"
              spellCheck={false}
              data-testid="key-input"
            />
          </div>
          {submit.kind === 'error' && (
            <p className="field-error" data-testid="key-error">
              {submit.detail}
            </p>
          )}
          <button
            className="btn btn-secondary block"
            onClick={onRegister}
            disabled={submit.kind === 'validating' || keyInput.trim().length === 0}
            data-testid="key-submit"
          >
            {submit.kind === 'validating' ? 'Validating…' : 'Register key'}
          </button>
        </div>
      )}

      {/* ── envelope-encryption assurance ───────────────────── */}
      <div className="card row top" style={{ marginTop: 14, gap: 12 }}>
        <span className="ico ico-28">
          <LockIcon />
        </span>
        <p className="body sm grow" style={{ color: 'var(--text-secondary)' }}>
          키는 봉투 암호화(envelope encryption)로 저장되며, LLM 호출 직전에만 메모리에서
          복호화됩니다.
        </p>
      </div>

      {/* ── nav ─────────────────────────────────────────────── */}
      <div className="btn-row" style={{ marginTop: 56 }}>
        <button className="btn btn-secondary" style={{ flex: 'none', width: 50 }} aria-label="back">
          ‹
        </button>
        <button className="btn btn-primary grow" disabled={!ready} data-testid="continue-btn">
          Continue
        </button>
      </div>
    </>
  );
}
