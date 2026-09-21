// docs/mockups/JRN-discover-features.html#STP-add-missing 의 구현.
//
// 한 문장을 받아 초안 하나를 만드는 화면. 근거를 찾았는지, 초안이 무엇인지, 확정될
// 목록이 몇인지는 전부 서버가 준다 — 화면이 기억하는 것은 사람이 지금 치고 있는
// 문장과 방금 받은 초안뿐이다.

import { Fragment, useEffect, useState } from 'react';
import { decideAddition, draftAddition, getAdditions } from './api';
import type { DraftDependency, FeatureAddition, FeatureAdditions } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** Concatenated for the reason `FeatureCandidates` gives: a template literal reads as
 *  one product string to the copy gate, and this is a server value with a separator. */
function dependencyLine(d: DraftDependency): string {
  return d.category + ' · ' + d.name;
}

type Props = {
  id: string;
  onBack: () => void;
  onConfirmed: () => void;
};

export function AddFeature({ id, onBack, onConfirmed }: Props) {
  const [counts, setCounts] = useState<FeatureAdditions | null>(null);
  const [request, setRequest] = useState('');
  const [draft, setDraft] = useState<FeatureAddition | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let active = true;
    getAdditions(id)
      .then((next) => active && setCounts(next))
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id]);

  const empty = request.trim().length === 0;

  // 보내기 전에 화면이 막는 이유는 빈 문장으로 사람의 키를 쓰는 호출을 하지 않기 위해서다.
  async function findEvidence() {
    if (empty || busy) return;
    setBusy(true);
    setError(null);
    try {
      setDraft(await draftAddition(id, request.trim()));
    } catch (e: unknown) {
      setError(messageOf(e));
    } finally {
      setBusy(false);
    }
  }

  async function decide(decision: 'confirm' | 'cancel') {
    if (draft === null || busy) return;
    setBusy(true);
    setError(null);
    try {
      await decideAddition(id, draft.id, decision);
      setDraft(null);
      setRequest('');
      if (decision === 'confirm') setCounts(await getAdditions(id));
    } catch (e: unknown) {
      setError(messageOf(e));
    } finally {
      setBusy(false);
    }
  }

  const found = draft !== null && draft.evidenceFound;
  const missing = draft !== null && !draft.evidenceFound;

  return (
    <main className="screen">
      <header className="appbar">
        <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
          ‹
        </button>
        <span className="appbar-title">빠진 기능 추가</span>
        <span className="icon-btn ghost" aria-hidden="true" />
      </header>

      <div style={{ marginTop: 18 }}>
        <h1 className="h-display">자동으로는 못 잡은 것</h1>
        <p className="h-display-sub">
          “이건 왜 없지” 싶은 기능을 그냥 문장으로 적어 주세요. 코드에서 근거를 찾아 인수 기준 초안과
          의존성 후보를 제안합니다.
        </p>
      </div>

      <div className="stack" style={{ marginTop: 18 }}>
        <div className="field">
          <label htmlFor="in-newfeature">Feature</label>
          <input
            type="text"
            id="in-newfeature"
            placeholder="주간 사용량 리포트 메일 발송"
            autoComplete="off"
            value={request}
            onChange={(e) => setRequest(e.target.value)}
            data-testid="new-feature"
          />
        </div>
        <button
          className="btn btn-secondary block"
          type="button"
          disabled={empty || busy}
          onClick={() => void findEvidence()}
          data-testid="find-evidence"
        >
          코드에서 근거 찾기
        </button>
      </div>

      {error !== null && (
        <div className="notice err on" style={{ marginTop: 12 }} data-testid="add-error">
          {error}
        </div>
      )}

      {found && draft !== null && (
        <div className="card" style={{ marginTop: 16 }} data-testid="draft-result">
          <div className="row between">
            <span className="caps">인수 기준 초안</span>
            <span className="badge success" data-testid="evidence-badge">
              <span className="dot" />
              근거 있음
            </span>
          </div>
          {draft.scenarios.map((scenario, index) => (
            <Fragment key={scenario.evidence + index}>
              <div className="code" style={{ marginTop: 12 }} data-testid="draft-scenario">
                <div className="ln">
                  <span className="n">1</span>
                  <span>
                    <span className="kw">Given</span> <span>{scenario.given}</span>
                  </span>
                </div>
                <div className="ln">
                  <span className="n">2</span>
                  <span>
                    <span className="kw">When</span> <span>{scenario.when}</span>
                  </span>
                </div>
                <div className="ln">
                  <span className="n">3</span>
                  <span>
                    <span className="kw">Then</span> <span>{scenario.then}</span>
                  </span>
                </div>
              </div>
              <p className="legend" style={{ marginTop: 12 }}>
                <span className="mk">↳</span> 근거 <span data-testid="draft-evidence">{scenario.evidence}</span>
              </p>
            </Fragment>
          ))}
          {draft.dependencies.length !== 0 && (
            <div className="stack" style={{ marginTop: 12 }} data-testid="draft-dependencies">
              <span className="caps">의존성 후보</span>
              {draft.dependencies.map((dependency) => (
                <p
                  className="body sm"
                  key={dependency.category + dependency.name}
                  data-testid="draft-dependency"
                  data-category={dependency.category}
                >
                  <span>{dependencyLine(dependency)}</span>
                  {dependency.evidence !== null && (
                    <span className="legend">
                      {' '}
                      <span className="mk">↳</span> 근거{' '}
                      <span data-testid="dependency-evidence">{dependency.evidence}</span>
                    </span>
                  )}
                </p>
              ))}
            </div>
          )}
          <div className="btn-row" style={{ marginTop: 12 }}>
            <button
              className="btn btn-primary grow"
              type="button"
              disabled={busy}
              onClick={() => void decide('confirm')}
              data-testid="confirm-draft"
            >
              이 초안으로 추가
            </button>
            <button
              className="btn btn-ghost"
              type="button"
              disabled={busy}
              onClick={() => void decide('cancel')}
              data-testid="cancel-draft"
            >
              취소
            </button>
          </div>
        </div>
      )}

      {missing && (
        <div className="notice warn on" style={{ marginTop: 16 }} data-testid="no-evidence">
          코드에서 근거를 찾지 못했어요. 없는 근거를 지어내지 않습니다 — <strong>“근거 없음”으로 표시한 채</strong> 추가하거나, 취소하고 다른 이름으로 다시 찾아보세요.
          <div className="btn-row" style={{ marginTop: 10 }}>
            <button
              className="btn btn-secondary grow"
              type="button"
              disabled={busy}
              onClick={() => void decide('confirm')}
              data-testid="add-anyway"
            >
              근거 없음으로 추가
            </button>
            <button
              className="btn btn-ghost"
              type="button"
              disabled={busy}
              onClick={() => void decide('cancel')}
              data-testid="cancel-add"
            >
              취소
            </button>
          </div>
        </div>
      )}

      <div className="stack" style={{ marginTop: 20 }}>
        <span className="caps">확정될 목록</span>
        <div className="card row between">
          <span className="body sm" style={{ color: 'var(--text-secondary)' }}>
            승인된 후보 + 직접 추가
          </span>
          <span className="metric" data-testid="final-count">
            {counts === null ? '' : counts.finalCount}
          </span>
        </div>
      </div>

      <div className="stack" style={{ marginTop: 22 }}>
        <button
          className="btn btn-primary block"
          type="button"
          onClick={onConfirmed}
          data-testid="confirm-list"
        >
          확정 목록 만들기
        </button>
      </div>
    </main>
  );
}
