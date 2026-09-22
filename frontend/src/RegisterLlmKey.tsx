// Register LLM Key — the screen behind
// docs/mockups/JRN-connect-repo.html#STP-register-llm-key (AC4.2 · AC4.3).

import { useEffect, useState } from 'react';
import {
  deleteKey,
  getConnection,
  getLlmLanguage,
  listKeys,
  preflight,
  registerKey,
  setLlmLanguage,
} from './api';
import type { Connection, LlmKey, LlmLanguage, ProviderId } from './api';

// Screen-only: this order and its first entry decide what a user who has never
// chosen starts on. The backend's own provider rule (`llm::DEFAULT_PROVIDER`) is a
// separate axis and is untouched by anything here.
const PROVIDERS: { id: ProviderId; label: string; keyPrefix: string }[] = [
  { id: 'anthropic', label: 'Anthropic', keyPrefix: 'sk-ant-' },
  { id: 'openai', label: 'OpenAI', keyPrefix: 'sk-' },
  { id: 'google', label: 'Google', keyPrefix: 'AIza' },
];

/**
 * Duplicates the backend's active-key rule (`llmkey::ACTIVE_KEY_SQL`): OpenAI first,
 * then most recently registered. The two must move together, or the screen names a
 * different key from the one the pipeline calls with.
 */
function activeProviderOf(keys: LlmKey[]): ProviderId | null {
  const active = keys.filter((k) => k.status === 'active');
  if (active.length === 0) return null;
  const preferred =
    active.find((k) => k.provider === 'openai') ??
    active.reduce((a, b) => (b.createdAt > a.createdAt ? b : a));
  const known = PROVIDERS.find((p) => p.id === preferred.provider);
  return known ? known.id : null;
}

const LANGUAGES: { id: LlmLanguage; label: string }[] = [
  { id: 'ko', label: '한국어' },
  { id: 'en', label: 'English' },
];

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

type Props = {
  onBack: () => void;
  onReady: () => void;
};

export function RegisterLlmKey({ onBack, onReady }: Props) {
  const [connection, setConnection] = useState<Connection | null>(null);
  const [keys, setKeys] = useState<LlmKey[]>([]);

  const [provider, setProvider] = useState<ProviderId>(PROVIDERS[0].id);
  const [keyInput, setKeyInput] = useState('');
  const [revealed, setRevealed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [keyError, setKeyError] = useState<string | null>(null);
  // `null` until the stored value arrives, so no button reads as chosen before then.
  const [language, setLanguage] = useState<LlmLanguage | null>(null);

  useEffect(() => {
    void load();
  }, []);

  async function load() {
    try {
      const [conn, ks, lang] = await Promise.all([getConnection(), listKeys(), getLlmLanguage()]);
      setConnection(conn);
      setKeys(ks);
      setLanguage(lang);
      // Mount only: a later selection must not be overwritten by a reload of the
      // same value.
      const already = activeProviderOf(ks);
      if (already) setProvider(already);
    } catch (e) {
      setKeyError(messageOf(e));
    }
  }

  function selectProvider(p: ProviderId) {
    setProvider(p);
    setKeyInput('');
    setKeyError(null);
  }

  // Saved on tap: the choice is independent of the key form, so it must not wait
  // for — or be lost with — "저장하고 계속".
  async function selectLanguage(next: LlmLanguage) {
    if (next === language) return;
    const previous = language;
    setLanguage(next);
    try {
      setLanguage(await setLlmLanguage(next));
    } catch (e) {
      setLanguage(previous);
      setKeyError(messageOf(e));
    }
  }

  async function register() {
    setBusy(true);
    setKeyError(null);
    try {
      await registerKey(provider, keyInput.trim());
      setKeyInput('');
      setRevealed(false);
      setKeys(await listKeys());
    } catch (e) {
      setKeyError(messageOf(e));
    } finally {
      setBusy(false);
    }
  }

  async function removeKey(id: string) {
    setBusy(true);
    try {
      await deleteKey(id);
      setKeyError(null);
      setKeys(await listKeys());
    } catch (e) {
      setKeyError(messageOf(e));
    } finally {
      setBusy(false);
    }
  }

  async function onSave() {
    if (keyInput.trim() !== '') {
      await register();
      return;
    }
    setBusy(true);
    setKeyError(null);
    try {
      await preflight();
      onReady();
    } catch (e) {
      setKeyError(messageOf(e));
    } finally {
      setBusy(false);
    }
  }

  const selected = PROVIDERS.find((p) => p.id === provider) ?? PROVIDERS[0];
  const activeKey = keys.find((k) => k.provider === provider && k.status === 'active') ?? null;
  const hasAnyActiveKey = keys.some((k) => k.status === 'active');
  const typed = keyInput.trim();
  // Shown, but not enforced: a badly shaped key is still submitted, because the
  // provider — not this screen — owns the verdict.
  const formatBad = typed !== '' && !typed.startsWith(selected.keyPrefix);
  const installed = connection?.installed ?? false;
  const permissions = connection?.permissions ?? [];

  return (
    <main className="screen">
      <header className="appbar">
        <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
          ‹
        </button>
        <span className="appbar-title">Bring your own key</span>
        <button className="icon-btn ghost" type="button" aria-hidden="true" tabIndex={-1} />
      </header>

      {installed && !hasAnyActiveKey && (
        <div className="notice" style={{ marginTop: 16 }}>
          설치한 App 범위는 그대로 보존돼 있어요. 키 등록만 마치면 이어서 진행됩니다.
        </div>
      )}

      {installed && (
        <div className="stack-10" style={{ marginTop: 12 }}>
          <div className="input" data-testid="connection">
            <span className="val">
              {connection?.account?.login ?? 'FeatureDoc'} · {connection?.repositoryCount ?? '—'}{' '}
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
      )}

      <div style={{ marginTop: 20 }}>
        <h1 className="h-display">분석 비용을 낼 키</h1>
        <p className="h-display-sub">
          모든 LLM 호출은 당신의 키로 수행돼요. FeatureDoc은 키를 보관할 뿐 소유하지 않습니다.
        </p>
      </div>

      <div className="card stack" style={{ marginTop: 24 }}>
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

      <div style={{ marginTop: 10 }}>
        {activeKey ? (
          <div className="input" data-testid="active-key">
            <span className="lbl">API Key</span>
            <span className="val">
              {activeKey.provider} · {activeKey.masked}
            </span>
            <button
              className="field-action ico ico-24"
              type="button"
              onClick={() => removeKey(activeKey.id)}
              disabled={busy}
              aria-label="revoke key"
              data-testid="remove-key"
            >
              <TrashIcon />
            </button>
          </div>
        ) : (
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
              placeholder={selected.keyPrefix + '…'}
              autoComplete="off"
              spellCheck={false}
              aria-label="API Key"
              data-testid="key-input"
              onChange={(e) => {
                setKeyInput(e.target.value);
                setKeyError(null);
              }}
            />
          </div>
        )}
      </div>

      {(formatBad || keyError) && (
        <div className="notice err" style={{ marginTop: 12 }} data-testid="key-error">
          {formatBad && (
            <span>
              <span>키 형식이 올바르지 않아요. </span>
              <span>
                {selected.label} 키는 <code>{selected.keyPrefix}</code>로 시작합니다.
              </span>
            </span>
          )}
          {keyError && <span>{keyError}</span>}
        </div>
      )}

      <div className="card stack" style={{ marginTop: 16 }}>
        <span className="caps">출력 언어</span>
        <div className="segment" style={{ marginTop: 4 }}>
          {LANGUAGES.map((l) => (
            <button
              key={l.id}
              type="button"
              className={`seg${l.id === language ? ' active' : ''}`}
              onClick={() => selectLanguage(l.id)}
              disabled={language === null}
              aria-pressed={l.id === language}
              data-testid={`lang-${l.id}`}
            >
              {l.label}
            </button>
          ))}
        </div>
        <p className="body sm">분석 결과의 문장을 이 언어로 받아요. 다음에 시작하는 분석부터 적용됩니다.</p>
      </div>

      <div className="card row top" style={{ marginTop: 16, gap: 12 }}>
        <span className="ico ico-28">
          <LockIcon />
        </span>
        <p className="body sm grow">
          봉투 암호화로 저장되고 호출 직전에만 복호화됩니다. 폐기하면 진행 중인 분석은 현재
          호출까지만 마치고 멈춰요.
        </p>
      </div>

      <div className="stack" style={{ marginTop: 26 }}>
        <button
          className="btn btn-primary block"
          type="button"
          onClick={onSave}
          disabled={busy || (typed === '' && !hasAnyActiveKey)}
          data-testid="register-key"
        >
          저장하고 계속
        </button>
        <button
          className="btn-link"
          style={{ alignSelf: 'center' }}
          type="button"
          onClick={onReady}
          data-testid="skip-key"
        >
          나중에 하기
        </button>
      </div>
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

function TrashIcon() {
  return (
    <svg width="13" height="13" viewBox="0 0 13 13" fill="none">
      <path d="M2.6 3.6h7.8" stroke="currentColor" strokeWidth="1.1" />
      <path
        d="M4.2 3.6V2.6h4.6v1M3.6 3.6l.5 6.4h4.8l.5-6.4"
        stroke="currentColor"
        strokeWidth="1.1"
      />
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
