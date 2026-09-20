// docs/mockups/JRN-discover-features.html#STP-review-landscape
//
// Everything rendered here is the stored document, never anything derived
// client-side.
//
// Two places where this screen has no mockup counterpart to compare against: an
// item renders *every* path it cites where the mockup draws one, and the
// reproducibility line is drawn nowhere in the mockup at all.
//
// Arriving here says nothing about whether the strategy exists yet — stages finish
// in order but not instantly — so the way out is gated rather than always offered.

import { useEffect, useState } from 'react';
import { getAnalysis, getCrossCutting } from './api';
import type { CrossCuttingDocument } from './api';

/** Insertion order is PRD order — `AXIS_ORDER` below takes it from these keys. */
const AXIS_LABELS: Record<string, string> = {
  infrastructure: '인프라',
  repository_structure: '저장소 구조',
  architecture: '아키텍처',
  framework: '프레임워크 · 런타임',
  middleware: '미들웨어',
};

const AXIS_ORDER = Object.keys(AXIS_LABELS);

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

function reproText(doc: CrossCuttingDocument): string {
  switch (doc.reproducibility.verdict) {
    case 'unchanged':
      return '직전 분석과 동일한 결과예요';
    case 'changed':
      return '직전 분석과 결과가 달라졌어요';
    default:
      return '이 저장소의 첫 분석이에요';
  }
}

type Props = {
  id: string;
  onBack: () => void;
  onOpenDiscoveryStrategy: () => void;
};

export function CrossCuttingConcerns({ id, onBack, onOpenDiscoveryStrategy }: Props) {
  const [doc, setDoc] = useState<CrossCuttingDocument | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [strategyReady, setStrategyReady] = useState(false);
  const [hintOpen, setHintOpen] = useState(false);

  useEffect(() => {
    let active = true;
    getCrossCutting(id)
      .then((d) => active && setDoc(d))
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id]);

  // A separate read, so its failure never blocks the page this screen exists to show.
  useEffect(() => {
    let active = true;
    getAnalysis(id)
      .then((a) => {
        const stage = a.stages.find((s) => s.key === 'discovery_strategy');
        if (active) setStrategyReady(stage?.status === 'succeeded');
      })
      .catch(() => undefined);
    return () => {
      active = false;
    };
  }, [id]);

  if (error !== null) {
    return (
      <main className="screen">
        <Appbar onBack={onBack} />
        <div className="row" style={{ marginTop: 22 }} data-testid="concerns-error">
          <span className="badge danger">
            <span className="dot" />
            Error
          </span>
          <span className="body sm">{error}</span>
        </div>
      </main>
    );
  }

  if (doc === null) {
    return (
      <main className="screen">
        <Appbar onBack={onBack} />
        {/* The mockup draws no waiting copy, so the wait is an empty place rather
            than a sentence this screen invented. */}
        <div style={{ marginTop: 22 }} data-testid="concerns-loading" />
      </main>
    );
  }

  // An axis the document omitted entirely still gets its heading: a silently
  // missing one would read as "this axis was not required".
  const byAxis = new Map(doc.content.categories.map((c) => [c.axis, c.items]));
  const sections = AXIS_ORDER.map((axis) => ({
    axis,
    label: AXIS_LABELS[axis],
    items: byAxis.get(axis) ?? [],
  }));

  return (
    <main className="screen">
      <Appbar onBack={onBack} />

      <h1 className="h-display" style={{ marginTop: 24 }}>
        내 코드가 서 있는 바닥
      </h1>
      <p className="h-display-sub" data-testid="concerns-lede">
        항목마다 근거가 된 코드 위치를 함께 적었어요. 같은 커밋을 다시 분석하면 같은 결과가
        나오고, 달라지면 무엇이 달라졌는지 항목별로 알려 드립니다.
      </p>

      <div className="row" style={{ marginTop: 14 }} data-testid="reproducibility">
        <span className="tag" data-verdict={doc.reproducibility.verdict}>
          {reproText(doc)}
        </span>
        <span className="meta">{doc.model}</span>
      </div>

      <div className="stack-14" style={{ marginTop: 22 }}>
        {sections.map((section) => (
          <div className="card" key={section.axis} data-testid="axis" data-axis={section.axis}>
            <div className="section-title">
              <span>{section.label}</span>
              <span className="count">{section.items.length}</span>
            </div>
            {section.items.length === 0 ? (
              <p className="body sm" data-testid="axis-empty">
                이 저장소에서는 근거를 찾지 못했어요
              </p>
            ) : (
              <div className="stack-10" style={{ marginTop: 10 }}>
                {section.items.map((item, i) => (
                  <div className="row between" key={`${section.axis}-${i}`} data-testid="concern">
                    <span className="label" data-testid="concern-name">
                      {item.name}
                    </span>
                    {item.evidence.length === 0 ? (
                      <span className="tag warn" data-testid="concern-no-evidence">
                        <span className="dot" />
                        근거 없음
                      </span>
                    ) : (
                      <span className="meta" data-testid="concern-evidence">
                        {item.evidence.join(' · ')}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            )}
            <p className="legend" style={{ marginTop: 12 }}>
              <span className="mk">↳</span> 근거를 찾지 못한 항목은 지어내지 않고 그대로 표시합니다
            </p>
          </div>
        ))}
      </div>

      {strategyReady && (
        <>
          {hintOpen && (
            <div className="notice info on" style={{ marginTop: 16 }} data-testid="edit-hint">
              이 화면에서는 결과를 직접 고치지 않아요. 무엇이 빠졌는지 기억해 두었다가 <strong>다음 화면의 탐색 전략</strong>에서
              보태면, 그 보정이 후보 추출에 반영됩니다.
              <button
                className="btn btn-secondary"
                type="button"
                onClick={onOpenDiscoveryStrategy}
                data-testid="hint-to-strategy"
              >
                탐색 전략으로 가기
              </button>
            </div>
          )}

          <div className="stack" style={{ marginTop: 22 }}>
            <button
              className="btn btn-primary block"
              type="button"
              onClick={onOpenDiscoveryStrategy}
              data-testid="to-discovery-strategy"
            >
              탐색 전략 검토하기
            </button>
            <button
              className="btn btn-ghost block"
              type="button"
              onClick={() => setHintOpen(true)}
              data-testid="want-edit"
            >
              이 결과를 고치고 싶어요
            </button>
          </div>
        </>
      )}
    </main>
  );
}

function Appbar({ onBack }: { onBack: () => void }) {
  return (
    <header className="appbar">
      <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
        ‹
      </button>
      <span className="appbar-title">횡단 관심사</span>
      <span className="icon-btn ghost" aria-hidden="true" />
    </header>
  );
}
