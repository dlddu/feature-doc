// Analysis Diff (「달라진 것」) — the real screen behind
// docs/mockups/JRN-follow-code-change.html#STP-scan-diff (AC2.6).
//
// Its own screen rather than a section of Feature Acceptance, for the same reason
// Feature Dependencies is its own screen: the mockup draws it as its own screen card
// with its own appbar (「달라진 것」), and the 2026-09-18 「문서 권위 순서」 decision is
// that the mockup's composition wins where a higher document does not pin one. The
// journey's 터치포인트 column names the screens a reader passes through, not the
// screen composition.
//
// What it renders is a *comparison*, and the comparison is not the user's to choose:
// AC2.6 is about "이전 버전과의 차이", and the previous version of an analysis is the
// previous analysis of the same repository and branch. The server decides it
// (`GET /api/analyses/{id}/diff`) so the screen cannot show a difference against a
// run of something else.
//
// Only features that actually changed are listed. The journey pins that
// (`STP-scan-diff`: 「변경되지 않은 기능까지 갱신된 것처럼 보이면 diff를 신뢰하지 않게
// 된다 → 변경 없는 feature는 그대로 유지하고 표시도 하지 않는다」), and the server
// applies it — this screen never filters a feature out on its own.
//
// Three things the mockup draws that this slice does not, all registered in
// docs/doc-tracker.md "알려진 목업↔구현 편차" with 해소 시점 「슬라이스 6」:
//  · the conflict banner (「내가 손봤던 문장과 자동 결과가 부딪히는…」) and the
//    「확인 필요」 tag — a conflict is an event between an automatic result and a
//    *user edit*, and user edits are AC3.5. With nothing to conflict with, drawing
//    the banner would be a lie the screen tells on every run.
//  · 「부딪힌 곳 정리하기」 — the entry to `STP-resolve-conflict`, same AC3.5.
//  · the legend 「한 곳이라도 열어 봐야 다음으로 넘어갈 수 있어요」, which is that
//    CTA's gate and has nothing to gate without it.
// The always-true half of that pair — 「이번에는 부딪히는 편집이 없었어요」 — *is*
// drawn: it is true of every run in this slice, and it is what tells the reader the
// list in front of them is the whole story.

import { useEffect, useState } from 'react';
import { getAnalysisDiff } from './api';
import type { AnalysisDiff as Diff, DependencyLine, FeatureDiff, ScenarioLine } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** The mockup's Show filter. `all` is 「시나리오 + 의존성」. */
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
};

export function AnalysisDiff({ id, onBack, onOpenFeature }: Props) {
  const [diff, setDiff] = useState<Diff | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<string>(ALL);

  useEffect(() => {
    let active = true;
    getAnalysisDiff(id)
      .then((next) => active && setDiff(next))
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
            scenarioLines={row.scenarioLines}
            dependencyLines={row.dependencyLines}
            onOpen={() => onOpenFeature(row.feature.key)}
          />
        ))}
      </div>

      <div className="stack" style={{ marginTop: 22 }}>
        <button
          className="btn btn-ghost block"
          type="button"
          onClick={onBack}
          data-testid="diff-finish"
        >
          여기까지만 보고 마치기
        </button>
      </div>

    </main>
  );
}

function Card({
  feature,
  scenarioLines,
  dependencyLines,
  onOpen,
}: {
  feature: FeatureDiff;
  scenarioLines: ScenarioLine[];
  dependencyLines: DependencyLine[];
  onOpen: () => void;
}) {
  return (
    <div className="dfeat" data-testid="diff-feature" data-feat={feature.key}>
      <div className="dhead">
        <span className="dname" data-testid="diff-feature-name">
          {feature.name}
        </span>
        <span className="tag info">
          <span className="dot" />
          갱신됨
        </span>
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

/**
 * Copy the mockup has no counterpart for — the wait, the first run of a target, and
 * the run that changed nothing. The mockup draws a run that has changes to show;
 * the two states around it are ours. Registered in docs/doc-tracker.md
 * "알려진 목업↔구현 편차"; kept as constants so each deviation is one place.
 */
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
