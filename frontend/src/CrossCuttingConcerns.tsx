// Cross-cutting Concerns — the real screen behind
// docs/mockups/JRN-discover-features.html#STP-review-landscape (Cross-cutting Concerns, AC1.2).
//
// Everything here comes from `GET /api/analyses/{id}/documents/cross-cutting`:
// the document stage 2 produced and stored, never anything derived client-side.
//
// Two deliberate differences from the mockup as drawn:
//   · The mockup groups items under **four** headings (Infrastructure,
//     Architecture, Framework, Middleware). AC1.2 names **five** axes — it also
//     requires 저장소 구조 (monorepo 여부 · 모듈 구분). The PRD is the SSOT, so the
//     fifth axis is rendered; the mockup is the thing that is behind.
//   · The mockup's per-item evidence is a single path. AC1.2 says "파일 경로/심볼
//     참조" without capping it at one, so an item renders every path it cites.
// The first is registered in docs/doc-tracker.md "알려진 목업↔구현 편차".
//
// The reproducibility line has no mockup counterpart at all: AC1.2's verification
// method requires that a re-analysis either reproduce deterministically *or* state
// the difference, and a screen that never mentions it cannot satisfy that clause.
//
// The way out to Discovery Strategy is the mockup's: this screen does not edit its own result, it
// points at the strategy screen where the correction actually lands. It is gated on
// stage 3 having succeeded — the same rule Analysis Progress uses for its per-stage entry points,
// so the link never leads to a 404. Stages finish in order but not instantly, so
// arriving here says nothing about whether the strategy exists yet.

import { useEffect, useState } from 'react';
import { getAnalysis, getCrossCutting } from './api';
import type { CrossCuttingDocument } from './api';

/** AC1.2's five axes, in PRD order, with the label each one renders under. */
const AXIS_LABELS: Record<string, string> = {
  infrastructure: 'Infrastructure',
  repository_structure: 'Repository structure',
  architecture: 'Architecture',
  framework: 'Framework · runtime',
  middleware: 'Middleware',
};

const AXIS_ORDER = Object.keys(AXIS_LABELS);

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** What the reproducibility badge says, per AC1.2's determinism clause. */
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
  /** Cross-cutting Concerns → Analysis Progress (back to the run this document came from). */
  onBack: () => void;
  /** Cross-cutting Concerns → Discovery Strategy, offered only once stage 3 has a strategy to review. */
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

  // Whether the way out is offered is a fact about the run, not about this
  // document — so it is a separate read, and its failure never blocks the page
  // this screen exists to show.
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
        <Appbar onBack={onBack} sub="" />
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
        <Appbar onBack={onBack} sub="" />
        <p className="body sm" style={{ marginTop: 22 }} data-testid="concerns-loading">
          불러오는 중…
        </p>
      </main>
    );
  }

  // Render in AC1.2's order regardless of the order the document arrived in, and
  // include an axis the document omitted entirely as an explicit empty section —
  // a silently missing heading would read as "this axis was not required".
  const byAxis = new Map(doc.content.categories.map((c) => [c.axis, c.items]));
  const sections = AXIS_ORDER.map((axis) => ({
    axis,
    label: AXIS_LABELS[axis],
    items: byAxis.get(axis) ?? [],
  }));

  return (
    <main className="screen">
      <Appbar onBack={onBack} sub={`run ${id.slice(0, 8)}`} />

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

function Appbar({ onBack, sub }: { onBack: () => void; sub: string }) {
  return (
    <div className="appbar">
      <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
        ‹
      </button>
      <div>
        <div className="appbar-title">횡단 관심사</div>
        <div className="appbar-sub">{sub}</div>
      </div>
    </div>
  );
}
