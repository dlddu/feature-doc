// Feature Dependencies — the real screen behind
// docs/mockups/JRN-review-feature.html#STP-trace-dependencies (AC2.4 · AC2.5).
//
// Its own screen rather than a section of Feature Acceptance, because the mockup
// draws it as its own screen card with its own appbar (「종단 의존성」) — and the
// 2026-09-18 「문서 권위 순서」 decision is that the mockup's composition wins when
// the journey does not pin one.
//
// What it renders is rows, not a document: AC2.5 stores dependencies as structured
// data so that "이 데이터 모델을 쓰는 feature 전부" is a query rather than a re-read of
// every analysis. The screen is one reader of that data; the reverse query and the
// export are two others, and they have no surface here because the mockup draws
// none — inventing screen copy the mockup does not have would be drift in the other
// direction.
//
// 좁은 화면 기본은 목록이다. The journey pins that (`STP-trace-dependencies`:
// 「좁은 화면에서는 카테고리 목록을 기본으로, 그래프는 확장 보기로」), and the graph is
// drawn from the same rows — never from anything this screen made up.
//
// Two things the mockup draws that this slice does not, both registered in
// docs/doc-tracker.md "알려진 목업↔구현 편차":
//  · `어색한 표현 고쳐 달라 하기` — the entry to `STP-request-edit`, which is AC3.1
//    (슬라이스 6). A CTA to a screen that does not exist is a dead end.
//  · the request and waiting states. The mockup draws a feature whose dependencies
//    are already extracted; asking for them is a step it never shows.

import { useEffect, useState } from 'react';
import { getDependencies, requestDependencies } from './api';
import type { Dependency, FeatureDependencies as Deps } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/**
 * The seven categories AC2.4 lists, labelled as the mockup's 분류 filter labels them.
 * The keys are the server's — `backend/src/dependencies.rs` `CATEGORIES` is the one
 * definition, and a label that has no key would silently draw an empty eighth kind.
 */
const CATEGORY_LABELS: Record<string, string> = {
  infrastructure: '인프라',
  data: '데이터',
  architecture: '아키텍처',
  framework: '프레임워크',
  middleware: '미들웨어',
  logic: '로직',
  interface: '인터페이스',
};

const ALL = 'all';

/** How often a queued trace is re-read. The run is one model call behind a claim. */
const POLL_MS = 2000;

type Props = {
  id: string;
  featureKey: string;
  /** Feature Dependencies → Feature Acceptance (back to the document it came from). */
  onBack: () => void;
};

export function FeatureDependencies({ id, featureKey, onBack }: Props) {
  const [deps, setDeps] = useState<Deps | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<string>(ALL);
  const [graph, setGraph] = useState(false);
  /** Bumped by the poll below; reading is the effect's only trigger. */
  const [tick, setTick] = useState(0);

  useEffect(() => {
    let active = true;
    getDependencies(id, featureKey)
      .then((next) => active && setDeps(next))
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id, featureKey, tick]);

  // A queued trace finishes in the worker, not here, so the screen re-reads rather
  // than pretending it knows when. Only while queued — a settled answer is settled.
  useEffect(() => {
    if (deps === null || deps.status !== 'queued') return;
    const timer = window.setTimeout(() => setTick((n) => n + 1), POLL_MS);
    return () => window.clearTimeout(timer);
  }, [deps]);

  const ask = () => {
    setError(null);
    requestDependencies(id, featureKey)
      .then(setDeps)
      .catch((e: unknown) => setError(messageOf(e)));
  };

  if (deps === null) {
    return (
      <main className="screen">
        <Appbar onBack={onBack} onLeave={onBack} />
        <p className="body sm" style={{ marginTop: 22 }} data-testid="dependencies-loading">
          {error ?? LOADING}
        </p>
      </main>
    );
  }

  const items = deps.categories.flatMap((group) => group.items);
  const shown = filter === ALL ? items : items.filter((item) => item.category === filter);

  return (
    <main className="screen">
      <Appbar onBack={onBack} onLeave={onBack} />

      <div style={{ marginTop: 18 }}>
        <h1 className="h-display">이 기능이 기대고 있는 것들</h1>
        <p className="h-display-sub">
          좁은 화면에서는 목록이 기본이에요. 그림으로 한눈에 보고 싶으면 아래에서 켜면 됩니다.
        </p>
      </div>

      {deps.status === null && (
        <div className="card" style={{ marginTop: 16 }} data-testid="dependencies-unasked">
          <p className="body sm">{NOT_TRACED}</p>
          <button className="btn btn-primary block" type="button" onClick={ask} data-testid="trace">
            {TRACE}
          </button>
        </div>
      )}

      {deps.status === 'queued' && (
        <div className="notice info on" style={{ marginTop: 16 }} data-testid="dependencies-queued">
          {QUEUED}
        </div>
      )}

      {deps.status === 'failed' && (
        <div className="notice warn on" style={{ marginTop: 16 }} data-testid="dependencies-failed">
          <span data-testid="dependencies-error">{deps.error}</span>
          <button className="btn btn-secondary" type="button" onClick={ask} data-testid="retrace">
            {RETRACE}
          </button>
        </div>
      )}

      <div className="field" style={{ marginTop: 18 }}>
        <label htmlFor="in-dep-filter">분류</label>
        <select
          id="in-dep-filter"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          data-testid="dependency-filter"
        >
          <option value={ALL}>전체 7종</option>
          {Object.keys(CATEGORY_LABELS).map((category) => (
            <option key={category} value={category}>
              {CATEGORY_LABELS[category]}
            </option>
          ))}
        </select>
      </div>

      <label className="choice" htmlFor="in-graph" style={{ marginTop: 12 }}>
        <input
          type="checkbox"
          id="in-graph"
          checked={graph}
          onChange={(e) => setGraph(e.target.checked)}
          data-testid="graph-toggle"
        />
        <span className="ct">
          그림으로 보기
          <span className="cs">좁은 화면에서는 목록이 더 잘 읽혀서 기본은 꺼둡니다</span>
        </span>
      </label>

      {graph && <Graph items={shown} />}

      <div className="card" style={{ marginTop: 12 }}>
        <div data-testid="dependency-list">
          {shown.map((item) => (
            <Row key={item.category + item.name} item={item} />
          ))}
        </div>
        <p className="legend" style={{ marginTop: 12 }}>
          <span className="mk">↳</span> 근거를 찾지 못한 항목은 <strong>근거 없음</strong>으로 그대로
          두고 임의로 채우지 않아요
        </p>
      </div>

      <p className="meta" style={{ marginTop: 12 }}>
        보이는 항목 <strong data-testid="dependency-count">{shown.length}</strong>개
      </p>
    </main>
  );
}

function Row({ item }: { item: Dependency }) {
  return (
    <div className="dep" data-testid="dependency" data-cat={item.category}>
      <span className="dname" data-testid="dependency-name">
        {item.name}
      </span>
      {item.evidence === null ? (
        <span className="tag warn" data-testid="dependency-no-evidence">
          <span className="dot" />
          근거 없음
        </span>
      ) : (
        <span className="esrc" data-testid="dependency-evidence">
          {item.evidence}
        </span>
      )}
    </div>
  );
}

/**
 * The expanded view. One node per dependency around the feature, drawn from the
 * same rows the list shows — the mockup's own graph is four example nodes, and
 * putting a fixed shape here would draw the same picture for every feature.
 */
function Graph({ items }: { items: Dependency[] }) {
  const height = 132;
  const cx = 52;
  const cy = height / 2;
  const step = items.length > 1 ? (height - 40) / (items.length - 1) : 0;
  return (
    <div className="card" style={{ marginTop: 12 }} data-testid="dependency-graph">
      <span className="caps">연결 그림</span>
      <svg
        viewBox="0 0 320 132"
        width="100%"
        height={height}
        role="img"
        aria-label="의존성 연결 그림"
        style={{ marginTop: 10 }}
      >
        {items.map((item, index) => {
          const y = items.length > 1 ? 20 + step * index : cy;
          return (
            <line
              key={item.category + item.name}
              x1={cx}
              y1={cy}
              x2={220}
              y2={y}
              stroke="var(--border-strong)"
              strokeWidth="1"
            />
          );
        })}
        <circle cx={cx} cy={cy} r="7" fill="var(--accent)" />
        {items.map((item, index) => {
          const y = items.length > 1 ? 20 + step * index : cy;
          return (
            <circle key={item.category + item.name} cx={220} cy={y} r="5" fill="var(--text-quaternary)" />
          );
        })}
      </svg>
    </div>
  );
}

/**
 * Copy the mockup has no counterpart for — the wait, and the states before anyone
 * asked for a trace. Registered in docs/doc-tracker.md "알려진 목업↔구현 편차"; kept
 * as constants so each deviation is one place.
 */
const LOADING = '불러오는 중…';
const NOT_TRACED = '아직 이 기능의 의존성을 분석하지 않았어요.';
const TRACE = '의존성 분석';
const QUEUED = '분석을 요청했어요. 끝나면 이 화면에 그려집니다.';
const RETRACE = '다시 분석하기';

/**
 * The mockup's three-slot appbar. Unlike `STP-read-scenarios` this step is *not* the
 * journey's first, so the left slot is a real back control (`‹`), and the right one
 * is the journey's exit (`✕ 나가기`). Both land on the same place this screen was
 * opened from — the acceptance document for this feature.
 */
function Appbar({ onBack, onLeave }: { onBack: () => void; onLeave: () => void }) {
  return (
    <header className="appbar">
      <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
        ‹
      </button>
      <span className="appbar-title">종단 의존성</span>
      <button className="icon-btn" type="button" onClick={onLeave} aria-label="나가기">
        ✕
      </button>
    </header>
  );
}
