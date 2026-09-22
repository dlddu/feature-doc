// Analysis Diff (「달라진 것」) — the real screen behind
// docs/mockups/JRN-follow-code-change.html#STP-scan-diff.
//
// The screen never drops a feature on its own: what is listed is what the server
// sent.

import { useEffect, useState } from 'react';
import { getAnalysisDiff, listConflicts } from './api';
import type {
  AnalysisDiff as Diff,
  DependencyLine,
  DocConflict,
  FeatureDiff,
  ScenarioLine,
} from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

const ALL = 'all';
const SCENARIO = 'scenario';
const DEP = 'dep';

/** `-` on the wire, `−` (U+2212) on screen — the mockup's minus is the glyph. */
function markOf(mark: string): string {
  return mark === '+' ? '+' : '−';
}

type Props = {
  id: string;
  /** 달라진 것 → Analysis Progress (the run this diff belongs to). */
  onBack: () => void;
  /** 「달라진 곳 보기」 → that feature's acceptance document. */
  onOpenFeature: (featureKey: string) => void;
  /** 「부딪힌 곳 정리하기」 → the first conflict still open. */
  onOpenConflict: (conflictId: string) => void;
};

/**
 * Whether the reader has opened at least one changed feature of this run — the
 * mockup lets nobody past this screen before reading one place. Opening navigates
 * away, so the mark outlives the screen but not the tab.
 */
function openedKey(id: string): string {
  return OPENED_PREFIX + id;
}

const OPENED_PREFIX = 'diff-opened-';

function hasOpened(id: string): boolean {
  try {
    return window.sessionStorage.getItem(openedKey(id)) === '1';
  } catch {
    return false;
  }
}

function markOpened(id: string): void {
  try {
    window.sessionStorage.setItem(openedKey(id), '1');
  } catch {
  }
}

export function AnalysisDiff({ id, onBack, onOpenFeature, onOpenConflict }: Props) {
  const [diff, setDiff] = useState<Diff | null>(null);
  const [conflicts, setConflicts] = useState<DocConflict[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<string>(ALL);
  const [opened, setOpened] = useState<boolean>(() => hasOpened(id));

  useEffect(() => {
    let active = true;
    Promise.all([getAnalysisDiff(id), listConflicts(id)])
      .then(([nextDiff, nextConflicts]) => {
        if (!active) return;
        setDiff(nextDiff);
        setConflicts(nextConflicts.conflicts.filter((c) => c.status === 'open'));
      })
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id]);

  if (diff === null) {
    return (
      <main className="screen">
        <Appbar onBack={onBack} />
        <p className="body sm" style={{ marginTop: 22 }} data-testid="diff-loading">
          {error ?? LOADING}
        </p>
      </main>
    );
  }

  const shown = diff.features
    .map((feature) => ({
      feature,
      scenarioLines: filter === DEP ? [] : feature.scenarioLines,
      dependencyLines: filter === SCENARIO ? [] : feature.dependencyLines,
    }))
    .filter((row) => row.scenarioLines.length > 0 || row.dependencyLines.length > 0);
  const lines = shown.reduce(
    (total, row) => total + row.scenarioLines.length + row.dependencyLines.length,
    0,
  );

  return (
    <main className="screen">
      <Appbar onBack={onBack} />

      <div className="row between" style={{ marginTop: 16, gap: 10 }}>
        <div className="field" style={{ flex: 1, minWidth: 0 }}>
          <label htmlFor="in-diff-scope">Show</label>
          <select
            id="in-diff-scope"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            data-testid="diff-filter"
          >
            <option value={ALL}>시나리오 + 의존성</option>
            <option value={SCENARIO}>시나리오만</option>
            <option value={DEP}>의존성만</option>
          </select>
        </div>
        <div style={{ textAlign: 'right' }}>
          <div className="metric" data-testid="diff-line-count">
            {lines}
          </div>
          <div className="caps" style={{ marginTop: 6 }}>
            변경 줄
          </div>
        </div>
      </div>

      {diff.comparedTo === null ? (
        // 「견줄 상대가 없다」와 「바뀐 게 없다」는 다른 말이다. 첫 분석에 후자를
        // 띄우면 이 저장소가 한 번 분석된 적이 있다고 말하는 셈이 된다.
        <div className="notice info on" style={{ marginTop: 14 }} data-testid="diff-first-run">
          {FIRST_RUN}
        </div>
      ) : conflicts.length > 0 ? (
        <div className="notice warn on" style={{ marginTop: 14 }} data-testid="diff-conflict-banner">
          <span>내가 손봤던 문장과 자동 결과가 부딪히는 기능이 </span>
          <strong data-testid="diff-conflict-count">{conflictFeatures(conflicts).length}</strong>
          <span>건 있어요. 덮어쓰지 않고 그대로 두었습니다.</span>
        </div>
      ) : (
        <div className="notice ok on" style={{ marginTop: 14 }} data-testid="diff-no-conflict">
          이번에는 부딪히는 편집이 없었어요. 달라진 곳만 확인하면 여기서 마쳐도 됩니다.
        </div>
      )}

      {diff.comparedTo !== null && diff.features.length === 0 && (
        <div className="notice info on" style={{ marginTop: 14 }} data-testid="diff-unchanged">
          {UNCHANGED}
        </div>
      )}

      <div style={{ marginTop: 16 }} data-testid="diff-list">
        {shown.map((row) => (
          <Card
            key={row.feature.key}
            feature={row.feature}
            conflicted={conflicts.some((c) => c.featureKey === row.feature.key)}
            scenarioLines={row.scenarioLines}
            dependencyLines={row.dependencyLines}
            onOpen={() => {
              markOpened(id);
              setOpened(true);
              onOpenFeature(row.feature.key);
            }}
          />
        ))}
      </div>

      <div className="stack" style={{ marginTop: 22 }}>
        <button
          className="btn btn-primary block"
          type="button"
          onClick={() => onOpenConflict(conflicts[0].id)}
          disabled={conflicts.length === 0 || !opened}
          data-testid="diff-to-conflict"
        >
          부딪힌 곳 정리하기
        </button>
        <button
          className="btn btn-ghost block"
          type="button"
          onClick={onBack}
          data-testid="diff-finish"
        >
          여기까지만 보고 마치기
        </button>
      </div>

      <p className="legend" style={{ marginTop: 20 }}>
        <span className="mk">↳</span> 한 곳이라도 열어 봐야 다음으로 넘어갈 수 있어요
      </p>
    </main>
  );
}

/** 배너가 세는 것은 충돌 행이 아니라 **기능**이다 — 한 기능에 두 자리가 부딪혀도 한 건. */
function conflictFeatures(conflicts: DocConflict[]): string[] {
  return Array.from(new Set(conflicts.map((c) => c.featureKey)));
}

function Card({
  feature,
  conflicted,
  scenarioLines,
  dependencyLines,
  onOpen,
}: {
  feature: FeatureDiff;
  conflicted: boolean;
  scenarioLines: ScenarioLine[];
  dependencyLines: DependencyLine[];
  onOpen: () => void;
}) {
  return (
    <div
      className={conflicted ? 'dfeat conflict' : 'dfeat'}
      data-testid="diff-feature"
      data-feat={feature.key}
      data-conflict={conflicted ? '1' : '0'}
    >
      <div className="dhead">
        <span className="dname" data-testid="diff-feature-name">
          {feature.name}
        </span>
        {conflicted ? (
          <span className="tag warn" data-testid="diff-feature-tag">
            <span className="dot" />
            확인 필요
          </span>
        ) : (
          <span className="tag info" data-testid="diff-feature-tag">
            <span className="dot" />
            갱신됨
          </span>
        )}
      </div>
      <span className="dwhere">
        <span data-testid="diff-feature-where">{feature.location}</span>
        <span data-testid="diff-feature-scenarios"> · {feature.scenarios}</span> scenarios
      </span>
      <div className="dbody">
        {scenarioLines.map((line, index) => (
          <div className="dline" data-kind="scenario" data-testid="diff-line" key={`s${index}`}>
            <span className="dmark">{markOf(line.mark)}</span>
            <span className="dtext" data-testid="diff-line-text">
              {line.text}
            </span>
          </div>
        ))}
        {dependencyLines.map((line, index) => (
          <div className="dline" data-kind="dep" data-testid="diff-line" key={`d${index}`}>
            <span className="dmark">{markOf(line.mark)}</span>
            <span className="dtext" data-testid="diff-line-text">
              {line.mark === '+' ? '의존성 추가 — ' : '의존성 제거 — '}
              <code data-testid="diff-dependency-name">{line.name}</code>
            </span>
          </div>
        ))}
      </div>
      <div className="btn-row" style={{ marginTop: 11 }}>
        <button className="btn btn-ghost" type="button" onClick={onOpen} data-testid="diff-open">
          달라진 곳 보기
        </button>
      </div>
    </div>
  );
}

/** 목업에 대응 카피가 없는 세 상태 — 대기 · 첫 분석 · 변경 없음. */
const LOADING = '불러오는 중…';
const FIRST_RUN = '이 저장소를 처음 분석했어요. 견줄 이전 결과가 아직 없습니다.';
const UNCHANGED = '이번 재분석에서 달라진 기능이 없어요.';

/**
 * The mockup's three-slot appbar: a back control, the title, and a right-hand
 * `icon-btn ghost` placeholder that keeps the title centred (M7 counts the slots).
 */
function Appbar({ onBack }: { onBack: () => void }) {
  return (
    <header className="appbar">
      <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
        ‹
      </button>
      <span className="appbar-title">달라진 것</span>
      <span className="icon-btn ghost" aria-hidden="true" />
    </header>
  );
}
