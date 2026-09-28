// docs/mockups/JRN-review-feature.html#STP-verify-evidence 의 구현.

import { useEffect, useState } from 'react';
import { getAcceptance, getEvidence } from './api';
import type { EvidenceExcerpt, FeatureAcceptance as Doc } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

const NONE = '';

type Props = {
  id: string;
  featureKey: string;
  onBack: () => void;
  onLeave: () => void;
  onOpenDependencies: () => void;
};

export function FeatureEvidence({ id, featureKey, onBack, onLeave, onOpenDependencies }: Props) {
  const [feature, setFeature] = useState<Doc | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [picked, setPicked] = useState(NONE);
  const [excerpt, setExcerpt] = useState<EvidenceExcerpt | null>(null);
  const [opened, setOpened] = useState<number[]>([]);
  const [busy, setBusy] = useState(false);
  const [openError, setOpenError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    getAcceptance(id)
      .then((doc) => {
        if (!active) return;
        const found = doc?.content.features.find((f) => f.key === featureKey) ?? null;
        setFeature(found);
        if (found === null) setError(NOT_GENERATED);
      })
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id, featureKey]);

  if (feature === null) {
    return (
      <main className="screen">
        <Appbar onBack={onBack} onLeave={onLeave} />
        <p className="body sm" style={{ marginTop: 22 }} data-testid="evidence-error">
          {error ?? LOADING}
        </p>
      </main>
    );
  }

  const scenarios = feature.scenarios;
  const index = picked === NONE ? -1 : Number(picked);

  const onOpen = () => {
    const scenario = scenarios[index];
    if (scenario === undefined) return;
    setBusy(true);
    setOpenError(null);
    getEvidence(id, featureKey, scenario.evidence)
      .then((next) => {
        setExcerpt(next);
        setOpened((seen) => (seen.includes(index) ? seen : [...seen, index]));
      })
      .catch((e: unknown) => {
        setExcerpt(null);
        setOpenError(messageOf(e));
      })
      .finally(() => setBusy(false));
  };

  return (
    <main className="screen">
      <Appbar onBack={onBack} onLeave={onLeave} />

      <div style={{ marginTop: 18 }}>
        <h1 className="h-display" data-testid="evidence-title">
          이 문장, 진짜 이 코드에서 나왔나요
        </h1>
        <p className="h-display-sub">
          근거는 파일 하나를 통째로 열지 않고 그 부분만 보여 드려요. 확인하고 나면 읽던 자리로
          그대로 돌아옵니다.
        </p>
      </div>

      <div className="field" style={{ marginTop: 18 }}>
        <label htmlFor="in-scn">근거를 열어 볼 시나리오</label>
        <select
          id="in-scn"
          value={picked}
          onChange={(e) => setPicked(e.target.value)}
          data-testid="evidence-select"
        >
          <option value={NONE}>고르지 않음</option>
          {scenarios.map((scenario, at) => (
            <option key={scenario.evidence + at} value={String(at)}>
              {'시나리오 '}
              {at + 1}
            </option>
          ))}
        </select>
      </div>
      <button
        className="btn btn-secondary block"
        type="button"
        style={{ marginTop: 10 }}
        disabled={index === -1 || busy}
        onClick={onOpen}
        data-testid="btn-open-evidence"
      >
        이 시나리오의 근거 열기
      </button>

      {opened.length === 0 && (
        <div className="notice on" style={{ marginTop: 14 }} data-testid="ev-empty">
          아직 아무 근거도 열어 보지 않았어요. 하나라도 확인해야 다음으로 넘어갈 수 있습니다 —
          읽지 않고 넘기면 검수가 아니라 서명이 되니까요.
        </div>
      )}

      {openError !== null && (
        <p className="body sm" style={{ marginTop: 14 }} data-testid="evidence-open-error">
          {openError}
        </p>
      )}

      {excerpt !== null && (
        <div className="card" style={{ marginTop: 14 }} data-testid="evidence-panel">
          <div className="row between">
            <span className="caps" data-testid="evidence-scenario">
              {'시나리오 '}
              {index + 1}
            </span>
            <span className="tag info">
              <span className="dot" />
              파일·심볼 단위
            </span>
          </div>
          <div className="snip">
            <div className="sh">
              <span data-testid="evidence-source">{sourceOf(excerpt)}</span>
              <span data-testid="evidence-lines">{rangeOf(excerpt)}</span>
            </div>
            <div className="code">
              {excerpt.lines.map((line, at) => (
                <div className="ln" key={at} data-testid="evidence-line">
                  {line}
                </div>
              ))}
            </div>
          </div>
          <div className="btn-row" style={{ marginTop: 12 }}>
            <button
              className="btn btn-ghost"
              type="button"
              onClick={onBack}
              data-testid="btn-ev-back"
            >
              읽던 자리로
            </button>
          </div>
        </div>
      )}

      <p className="meta" style={{ marginTop: 14 }}>
        확인한 근거 <strong data-testid="evidence-count">{opened.length}</strong>
        {' / '}
        {scenarios.length}
      </p>

      <div className="stack" style={{ marginTop: 20 }}>
        <button
          className="btn btn-primary block"
          type="button"
          disabled={opened.length === 0}
          onClick={onOpenDependencies}
          data-testid="btn-to-deps"
        >
          이 기능이 무엇에 기대는지 보기
        </button>
      </div>
    </main>
  );
}

function sourceOf(excerpt: EvidenceExcerpt): string {
  return excerpt.symbol === null ? excerpt.path : excerpt.path + ' · ' + excerpt.symbol;
}

function rangeOf(excerpt: EvidenceExcerpt): string {
  return 'L' + String(excerpt.startLine) + '–L' + String(excerpt.endLine);
}

const LOADING = '불러오는 중…';
const NOT_GENERATED = '인수 시나리오 생성 단계가 아직 끝나지 않았어요.';

function Appbar({ onBack, onLeave }: { onBack: () => void; onLeave: () => void }) {
  return (
    <header className="appbar">
      <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
        ‹
      </button>
      <span className="appbar-title">근거 확인</span>
      <button className="icon-btn" type="button" onClick={onLeave} aria-label="나가기">
        ✕
      </button>
    </header>
  );
}
