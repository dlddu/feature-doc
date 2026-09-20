// Register LLM Key — the screen behind
// docs/mockups/JRN-connect-repo.html#STP-register-llm-key (AC4.2 · AC4.3).
//
// Split out of `CredentialsSetup.tsx` by 수렴 슬라이스 ⑦ (rct_20260919-0007): the
// mockup draws 권한 부여 and 키 등록 as two steps and the 2026-09-18 authority-order
// ruling gave the mockup the screen composition, so the merged screen became
// `GrantRepoAccess.tsx` + this one. The appbar, the headline, the 봉투 암호화 card
// and both buttons are the mockup's step, one for one.
//
// Two things this screen draws that the mockup's step does not, both 원장 rows:
//   * 제공자 변이 — the static mockup holds one variant (Anthropic) of copy its
//     prototype script swaps per provider (`KEY_RULES`). The screen derives the
//     variant from the selected provider, so `AIza` (Google's prefix) is the only
//     literal that has no mockup counterpart.
//   * 이미 등록된 키 — the mockup's step is the first-run scene only. A returning
//     user must see which key an analysis would call with, and 시나리오 4 requires
//     revoking it, so the API Key field has a registered state built from values
//     (masked id, provider) rather than new copy.
//
// `저장하고 계속` is one button doing what the mockup's `btn-savekey` does, in two
// moves: with a key typed it registers (and stays, because 시나리오 3 replaces a
// key without leaving), with the field empty and a key already active it
// pre-flights and hands off to Home. `나중에 하기` is the mockup's abandon branch.

import { useEffect, useState } from 'react';
import { deleteKey, getConnection, listKeys, preflight, registerKey } from './api';
import type { Connection, LlmKey, ProviderId } from './api';

// Mockup order (`JRN-connect-repo.html#STP-register-llm-key`, the `#in-provider`
// options): Anthropic → OpenAI → Google, with the key hint written for the first
// entry (`sk-ant-`). No upper document fixes the order — PRD AC4.2 and test/04
// 시나리오 13 are silent — so the mockup wins as the visual SSOT.
//
// The *backend's* default provider rule is a separate axis and is untouched: the
// worker still falls back to OpenAI and an analysis still picks an OpenAI key over
// the others (`llm::DEFAULT_PROVIDER`, `llmkey::ACTIVE_KEY_SQL`). What this array
// decides is only what the screen shows and which segment a user who has never
// chosen starts on — `load()` replaces it with the provider of an already active
// key, so the initial value is a first-run affordance, not a policy.
//
// `keyPrefix` is the mockup prototype's `KEY_RULES[…].prefix`: it writes both the
// input's example and the format hint, which is why neither is a per-provider
// sentence in this file.
const PROVIDERS: { id: ProviderId; label: string; keyPrefix: string }[] = [
  { id: 'anthropic', label: 'Anthropic', keyPrefix: 'sk-ant-' },
  { id: 'openai', label: 'OpenAI', keyPrefix: 'sk-' },
  { id: 'google', label: 'Google', keyPrefix: 'AIza' },
];

/**
 * The provider whose key an analysis would actually use, or `null` if the user has
 * none. Mirrors the backend's active-key rule (`llmkey::ACTIVE_KEY_SQL`): OpenAI
 * first, then most recently registered — so the screen names the same key the
 * pipeline would call with.
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

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

type Props = {
  /** Back to 권한 부여 — the mockup's appbar `‹` (`data-goto="STP-grant-repo-access"`). */
  onBack: () => void;
  /** Continue into Home, the mockup's `data-goto="STP-pick-target"`. */
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

  useEffect(() => {
    void load();
  }, []);

  async function load() {
    try {
      const [conn, ks] = await Promise.all([getConnection(), listKeys()]);
      setConnection(conn);
      setKeys(ks);
      // Someone who already registered a key sees *that* provider. The default
      // below is where a user who has not chosen starts, not a value that
      // overrides a choice already made — so this runs on mount only and never
      // fights a later selection.
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

  /** The mockup's `저장하고 계속`: save what was typed, or continue with what is stored. */
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
  // The mockup's `keyValid()`, minus the disabling: a key that fails the shape
  // check is still submitted, because the provider — not this screen — owns the
  // verdict (시나리오 13 rejects a well-shaped key the platform cannot call).
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
