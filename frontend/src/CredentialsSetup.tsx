// S01 · Credentials Setup — the real, stateful screen behind
// docs/mockups/s01-credentials-setup.html (AC4.1 · AC4.2 · AC4.3).

import { useEffect, useState } from 'react';
import {
  deleteKey,
  getConnection,
  getInstallUrl,
  getMe,
  listKeys,
  LOGIN_URL,
  preflight,
  registerKey,
} from './api';
import type { Connection, LlmKey, ProviderId, User } from './api';

const PROVIDERS: { id: ProviderId; label: string; placeholder: string }[] = [
  { id: 'anthropic', label: 'Anthropic', placeholder: 'sk-ant-…' },
  { id: 'openai', label: 'OpenAI', placeholder: 'sk-…' },
  { id: 'google', label: 'Google', placeholder: 'AIza…' },
];

type KeyState = 'idle' | 'verifying' | 'error';
type ContinueState = 'idle' | 'checking' | 'done' | 'error';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export function CredentialsSetup() {
  // undefined = still loading the session
  const [me, setMe] = useState<User | null | undefined>(undefined);
  const [connection, setConnection] = useState<Connection | null>(null);
  const [keys, setKeys] = useState<LlmKey[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);

  const [provider, setProvider] = useState<ProviderId>('anthropic');
  const [keyInput, setKeyInput] = useState('');
  const [revealed, setRevealed] = useState(false);
  const [keyState, setKeyState] = useState<KeyState>('idle');
  const [keyError, setKeyError] = useState<string | null>(null);

  const [connecting, setConnecting] = useState(false);
  const [continueState, setContinueState] = useState<ContinueState>('idle');
  const [continueError, setContinueError] = useState<string | null>(null);

  useEffect(() => {
    void load();
  }, []);

  async function load() {
    try {
      const user = await getMe();
      setMe(user);
      if (user) {
        const [conn, ks] = await Promise.all([getConnection(), listKeys()]);
        setConnection(conn);
        setKeys(ks);
      }
    } catch (e) {
      setMe(null);
      setLoadError(messageOf(e));
    }
  }

  async function refreshKeys() {
    setKeys(await listKeys());
  }

  function signIn() {
    window.location.href = LOGIN_URL;
  }

  async function connectApp() {
    setConnecting(true);
    try {
      window.location.href = await getInstallUrl();
    } catch (e) {
      setConnecting(false);
      setLoadError(messageOf(e));
    }
  }

  function selectProvider(p: ProviderId) {
    setProvider(p);
    setKeyState('idle');
    setKeyError(null);
  }

  async function register() {
    setKeyState('verifying');
    setKeyError(null);
    try {
      await registerKey(provider, keyInput.trim());
      setKeyInput('');
      setRevealed(false);
      setKeyState('idle');
      await refreshKeys();
    } catch (e) {
      setKeyState('error');
      setKeyError(messageOf(e));
    }
  }

  async function removeKey(id: string) {
    try {
      await deleteKey(id);
      setKeyState('idle');
      setKeyError(null);
      await refreshKeys();
    } catch (e) {
      setKeyError(messageOf(e));
    }
  }

  async function onContinue() {
    setContinueState('checking');
    setContinueError(null);
    try {
      await preflight();
      setContinueState('done');
    } catch (e) {
      setContinueState('error');
      setContinueError(messageOf(e));
    }
  }

  const installed = connection?.installed ?? false;
  const permissions = connection?.permissions ?? [];
  const activeKey = keys.find((k) => k.provider === provider && k.status === 'active') ?? null;
  const hasAnyActiveKey = keys.some((k) => k.status === 'active');
  const ready = installed && hasAnyActiveKey;
  const placeholder = PROVIDERS.find((p) => p.id === provider)?.placeholder ?? '';

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

      {me === undefined && (
        <p className="body sm" style={{ marginTop: 28 }}>
          불러오는 중…
        </p>
      )}

      {me === null && (
        <div className="stack-10" style={{ marginTop: 28 }}>
          <button className="btn btn-primary block" type="button" onClick={signIn} data-testid="signin">
            Sign in with GitHub
          </button>
          <p className="body sm">
            읽기 전용 최소 권한만 요청하고, 접근할 저장소는 당신이 직접 고릅니다. LLM 호출은
            당신의 키로만 일어나요.
          </p>
          {loadError && (
            <div className="row">
              <span className="badge danger">
                <span className="dot" />
                Error
              </span>
              <span className="body sm">{loadError}</span>
            </div>
          )}
        </div>
      )}

      {me && (
        <>
          {/* ── GitHub App connection ── */}
          {installed ? (
            <div className="stack-10" style={{ marginTop: 28 }}>
              <div className="input" data-testid="connection">
                <span className="field-action badge success">
                  <span className="dot" />
                  Installed
                </span>
                <span className="lbl">GitHub App</span>
                <span className="val">
                  {connection?.account?.login ?? 'FeatureDoc'} ·{' '}
                  {connection?.repositoryCount ?? '—'}{' '}
                  {connection?.repositoryCount === 1 ? 'repository' : 'repositories'}
                </span>
              </div>
              <div className="tag-row">
                {permissions.map((p) => (
                  <span className="tag" key={p}>
                    {p}
                  </span>
                ))}
              </div>
            </div>
          ) : (
            <div className="stack-10" style={{ marginTop: 28 }}>
              <div className="tag-row" data-testid="requested-permissions">
                {permissions.map((p) => (
                  <span className="tag" key={p}>
                    {p}
                  </span>
                ))}
              </div>
              <p className="body sm">
                읽기 전용 최소 권한만 요청합니다. 접근할 저장소는 GitHub에서 직접 고릅니다.
              </p>
              <button
                className="btn btn-primary block"
                type="button"
                onClick={connectApp}
                disabled={connecting}
                data-testid="connect-app"
              >
                {connecting ? '여는 중…' : 'Connect GitHub App'}
              </button>
            </div>
          )}

          {/* ── LLM provider ── */}
          <div className="card stack" style={{ marginTop: 18 }}>
            <span className="caps">LLM Provider</span>
            <div className="segment" style={{ marginTop: 4 }}>
              {PROVIDERS.map((p) => (
                <button
                  key={p.id}
                  type="button"
                  className={`seg${p.id === provider ? ' active' : ''}`}
                  onClick={() => selectProvider(p.id)}
                  data-testid={`provider-${p.id}`}
                >
                  {p.label}
                </button>
              ))}
            </div>
          </div>

          {/* ── API key ── */}
          <div style={{ marginTop: 10 }}>
            {activeKey ? (
              <div className="stack-10">
                <div className="input" data-testid="active-key">
                  <span className="field-action badge success">
                    <span className="dot" />
                    Active
                  </span>
                  <span className="lbl">API Key · {activeKey.provider}</span>
                  <span className="val">{activeKey.masked}</span>
                </div>
                <div className="row between">
                  <span className="meta">id {activeKey.fingerprint}</span>
                  <button
                    className="btn btn-ghost"
                    type="button"
                    onClick={() => removeKey(activeKey.id)}
                    data-testid="remove-key"
                  >
                    Remove
                  </button>
                </div>
              </div>
            ) : (
              <div className="stack-10">
                <div className="input">
                  <button
                    className="field-action ico ico-24"
                    type="button"
                    onClick={() => setRevealed((r) => !r)}
                    aria-label={revealed ? 'hide key' : 'reveal key'}
                  >
                    <EyeIcon />
                  </button>
                  <span className="lbl">API Key</span>
                  <input
                    className="field-input"
                    type={revealed ? 'text' : 'password'}
                    value={keyInput}
                    placeholder={placeholder}
                    autoComplete="off"
                    spellCheck={false}
                    aria-label="API Key"
                    data-testid="key-input"
                    onChange={(e) => {
                      setKeyInput(e.target.value);
                      if (keyState === 'error') {
                        setKeyState('idle');
                        setKeyError(null);
                      }
                    }}
                  />
                </div>
                {keyState === 'error' && keyError && (
                  <div className="row" data-testid="key-error">
                    <span className="badge danger">
                      <span className="dot" />
                      Invalid
                    </span>
                    <span className="body sm">{keyError}</span>
                  </div>
                )}
                <button
                  className="btn btn-secondary block"
                  type="button"
                  onClick={register}
                  disabled={keyState === 'verifying' || keyInput.trim() === ''}
                  data-testid="register-key"
                >
                  {keyState === 'verifying' ? '검증 중…' : '키 등록 및 검증'}
                </button>
              </div>
            )}
          </div>

          {/* ── envelope-encryption reassurance ── */}
          <div className="card row top" style={{ marginTop: 14, gap: 12 }}>
            <span className="ico ico-28">
              <LockIcon />
            </span>
            <p className="body sm grow">
              키는 봉투 암호화(envelope encryption)로 저장되며, LLM 호출 직전에만 메모리에서
              복호화됩니다.
            </p>
          </div>

          {/* ── continue ── */}
          <div style={{ marginTop: 56 }}>
            <div className="btn-row">
              <button
                className="btn btn-secondary"
                style={{ flex: 'none', width: 50 }}
                disabled
                aria-label="back"
              >
                ‹
              </button>
              <button
                className={`btn grow ${ready ? 'btn-primary' : 'btn-secondary'}`}
                type="button"
                onClick={onContinue}
                disabled={!ready || continueState === 'checking'}
                data-testid="continue"
              >
                {continueState === 'checking'
                  ? '확인 중…'
                  : continueState === 'done'
                    ? '준비 완료 ✓'
                    : 'Continue'}
              </button>
            </div>
            {continueState === 'done' && (
              <p className="body sm" style={{ marginTop: 10 }} data-testid="ready">
                자격증명이 준비됐어요. 다음 단계로 이동할 수 있어요.
              </p>
            )}
            {continueState === 'error' && continueError && (
              <p className="body sm" style={{ marginTop: 10 }}>
                {continueError}
              </p>
            )}
            {!ready && continueState !== 'done' && (
              <p className="body sm" style={{ marginTop: 10 }}>
                {!installed && !hasAnyActiveKey
                  ? 'GitHub App 연결과 API Key 등록을 마치면 계속할 수 있어요.'
                  : !installed
                    ? 'GitHub App을 연결하면 계속할 수 있어요.'
                    : 'API Key를 등록하면 계속할 수 있어요.'}
              </p>
            )}
          </div>
        </>
      )}

      <p className="legend" style={{ marginTop: 28 }}>
        <span className="mk">01</span> — onboarding · single step
      </p>
    </main>
  );
}

function EyeIcon() {
  return (
    <svg width="13" height="13" viewBox="0 0 13 13" fill="none">
      <path
        d="M1 6.5C2.4 3.7 4.3 2.3 6.5 2.3S10.6 3.7 12 6.5C10.6 9.3 8.7 10.7 6.5 10.7S2.4 9.3 1 6.5Z"
        stroke="currentColor"
        strokeWidth="1.1"
      />
      <circle cx="6.5" cy="6.5" r="1.8" stroke="currentColor" strokeWidth="1.1" />
    </svg>
  );
}

function LockIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
      <rect x="2.5" y="6" width="9" height="6.5" rx="1.4" stroke="currentColor" strokeWidth="1.1" />
      <path d="M4.3 6V4.2A2.7 2.7 0 0 1 9.7 4.2V6" stroke="currentColor" strokeWidth="1.1" />
    </svg>
  );
}
