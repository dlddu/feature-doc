// S08 · Feature · Acceptance — the real screen behind
// docs/mockups/JRN-review-feature.html#STP-read-scenarios (AC2.1 · AC2.2 · AC2.3).
//
// The mockup is the SSOT for what this screen says, so the copy below is the
// mockup's copy. What it *does* is PRD-2's first half: stage 5 read the confirmed
// feature's logic (AC2.1) and its tests (AC2.2) and wrote one document per feature
// in end-user language (AC2.3); this screen renders that document and nothing of its
// own. Every sentence on it came from the server, which is why a reload shows the
// same document — and why there is no local draft to lose.
//
// Two things the mockup draws that this slice does not implement, both registered in
// docs/doc-tracker.md "알려진 목업↔구현 편차" with 해소 시점 = 슬라이스 5b:
//  · the resume banner and the 검수 상태 (미검수 / 검수 완료, 잔여 카운트). Marking a
//    feature reviewed is an action the mockup only offers *after* the evidence step,
//    and that step is 5b — a 검수 완료 the user could set without opening any evidence
//    would be a signature, not a review.
//  · `근거가 진짜인지 확인하기` — the entry to `STP-verify-evidence`. Showing the code
//    behind a criterion needs the repository's file *contents*, and `repo_scan` reads
//    the tree only. That capability is 5b's, and a button that opened nothing would
//    be worse than no button.
//
// The contradiction block is here in full, though: separating what the code says
// from what the tests say is AC2.2 itself, not a nicety (test/02 시나리오 3).

import { useEffect, useState } from 'react';
import { getAcceptance } from './api';
import type { AcceptanceContradiction, AcceptanceScenario, FeatureAcceptance as Doc } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** `evidence` plus the symbol when the pass named one — AC2.1's 근거 위치.
 *
 *  Concatenated rather than interpolated for the reason `FeatureCandidates` gives:
 *  a template literal reads as one product string to the copy gate, and this is a
 *  server value with punctuation between its halves. */
function evidenceOf(scenario: AcceptanceScenario): string {
  return scenario.symbol === null
    ? scenario.evidence
    : scenario.evidence + ' · ' + scenario.symbol;
}

type Props = {
  id: string;
  /** S08 → S04 (back to the run this feature came out of). */
  onBack: () => void;
  /** S08 → S07 — the mockup's cross-journey exit when the *finding* was wrong. */
  onOpenCandidates: () => void;
};

export function FeatureAcceptance({ id, onBack, onOpenCandidates }: Props) {
  const [features, setFeatures] = useState<Doc[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  /** Whether the "this is not a feature" confirmation is showing. */
  const [notAFeature, setNotAFeature] = useState(false);

  useEffect(() => {
    let active = true;
    getAcceptance(id)
      .then((doc) => {
        if (!active) return;
        setFeatures(doc === null ? [] : doc.content.features);
      })
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id]);

  if (features === null) {
    return (
      <main className="screen">
        <Appbar onBack={onBack} />
        {error !== null ? (
          <p className="body sm" style={{ marginTop: 22 }} data-testid="acceptance-error">
            {error}
          </p>
        ) : (
          <p className="body sm" style={{ marginTop: 22 }} data-testid="acceptance-loading">
            {LOADING}
          </p>
        )}
      </main>
    );
  }

  if (features.length === 0) {
    return (
      <main className="screen">
        <Appbar onBack={onBack} />
        <p className="body sm" style={{ marginTop: 22 }} data-testid="acceptance-empty">
          {NOT_GENERATED}
        </p>
      </main>
    );
  }

  const current = features.find((f) => f.key === selected) ?? features[0];

  return (
    <main className="screen">
      <Appbar onBack={onBack} />

      <div className="field" style={{ marginTop: 16 }}>
        <label htmlFor="in-feature">검수할 기능</label>
        <select
          id="in-feature"
          value={current.key}
          onChange={(e) => setSelected(e.target.value)}
          data-testid="feature-select"
        >
          {features.map((feature) => (
            <option key={feature.key} value={feature.key}>
              {feature.name}
            </option>
          ))}
        </select>
      </div>

      <div style={{ marginTop: 18 }}>
        <h1 className="h-display" data-testid="feature-title">
          {current.name}
        </h1>
        <p className="h-display-sub">
          사용자에게 무슨 일이 벌어지는지를 그대로 적었어요. 개발 용어가 남아 있으면 알려 주세요 —
          이 문서는 코드를 읽지 않는 사람도 읽을 수 있어야 합니다.
        </p>
      </div>

      <div className="section-title" style={{ marginTop: 20 }}>
        <span className="caps">인수 시나리오</span>
        <span className="section-action" data-testid="scenario-count">
          {current.scenarios.length}
        </span>
      </div>

      <div style={{ marginTop: 12 }} data-testid="scenario-list">
        {current.scenarios.map((scenario, index) => (
          <Scenario key={scenario.evidence + index} scenario={scenario} index={index} />
        ))}
      </div>

      {current.contradictions.length !== 0 && (
        <div className="notice warn on" style={{ marginTop: 16 }} data-testid="contradictions">
          <strong>코드와 테스트가 다르게 말하는 부분이 있어요.</strong>
          <span>
            본 시나리오에 섞지 않고 따로 두었습니다 — 어느 쪽이 맞는지는 이 코드를 쓰신 분만
            판단할 수 있어요.
          </span>
          {current.contradictions.map((clash, index) => (
            <Contradiction key={clash.testEvidence + index} clash={clash} />
          ))}
        </div>
      )}

      <div className="stack" style={{ marginTop: 22 }}>
        <button
          className="btn btn-ghost block"
          type="button"
          onClick={() => setNotAFeature(true)}
          data-testid="not-a-feature"
        >
          이건 기능이 아닌 것 같아요
        </button>
        <button
          className="btn btn-ghost block"
          type="button"
          onClick={onBack}
          data-testid="leave-review"
        >
          나중에 이어서 볼게요
        </button>
      </div>

      {notAFeature && (
        <div className="notice warn on" style={{ marginTop: 16 }} data-testid="not-a-feature-confirm">
          표현이 아니라 <strong>발견 자체가 잘못된</strong> 경우예요. 이건 여기서 고칠 문제가 아니라
          후보를 다시 결정해야 하는 일이라, 후보 결정 화면으로 넘겨 드릴게요.
          <button
            className="btn btn-secondary"
            type="button"
            onClick={onOpenCandidates}
            data-testid="back-to-candidates"
          >
            후보 결정으로 넘기기
          </button>
        </div>
      )}
    </main>
  );
}

function Scenario({ scenario, index }: { scenario: AcceptanceScenario; index: number }) {
  return (
    <div className="scn" data-testid="scenario" data-source={scenario.source}>
      <span className="sn">{'시나리오 '}{index + 1}</span>
      <span className="gwt">
        <span className="k">주어진 상황</span> <span>{scenario.given}</span>
        <br />
        <span className="k">이럴 때</span> <span>{scenario.when}</span>
        <br />
        <span className="k">이렇게 됩니다</span> <span>{scenario.then}</span>
      </span>
      <div className="src">
        <span className="esrc" data-testid="scenario-evidence">
          {evidenceOf(scenario)}
        </span>
        <span className="tag success">
          <span className="dot" />
          근거 있음
        </span>
      </div>
    </div>
  );
}

function Contradiction({ clash }: { clash: AcceptanceContradiction }) {
  return (
    <div className="scn conflict" style={{ marginTop: 10 }} data-testid="contradiction">
      <span className="sn">확인 필요</span>
      <span className="gwt">
        <span>{clash.given}</span> <span>{clash.when}</span>
        <span> 코드는 </span>
        <strong>{clash.codeSays}</strong>. 테스트는 <strong>{clash.testSays}</strong>고 적혀 있어요.
      </span>
      <div className="src">
        <span className="esrc" data-testid="contradiction-code">
          {clash.codeEvidence}
        </span>
        <span className="esrc" data-testid="contradiction-test">
          {clash.testEvidence}
        </span>
      </div>
    </div>
  );
}

/**
 * Copy the static prototype has no counterpart for — a network wait and the state
 * before stage 5 has run. Registered in docs/doc-tracker.md "알려진 목업↔구현 편차";
 * kept as constants so each deviation is one place.
 */
const LOADING = '불러오는 중…';
const NOT_GENERATED = '인수 시나리오 생성 단계가 아직 끝나지 않았어요.';

function Appbar({ onBack }: { onBack: () => void }) {
  return (
    <div className="appbar">
      <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
        ‹
      </button>
      <div>
        <div className="appbar-title">기능 검수</div>
      </div>
    </div>
  );
}
