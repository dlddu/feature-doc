// docs/mockups/JRN-review-feature.html#STP-read-scenarios 의 구현.
//
// Every sentence on this screen came from the server — which is why a reload shows
// the same document, and why there is no local draft to lose.

import { useEffect, useState } from 'react';
import { deleteFeature, getAcceptance, listDeletions, restoreFeature } from './api';
import type {
  AcceptanceContradiction,
  AcceptanceScenario,
  FeatureAcceptance as Doc,
  FeatureDeletion,
} from './api';
import { useWideViewport } from './viewport';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** `evidence` plus the symbol when the pass named one.
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
  onBack: () => void;
  onOpenCandidates: () => void;
  onOpenHistory: (featureKey: string) => void;
};

export function FeatureAcceptance({ id, onBack, onOpenCandidates, onOpenHistory }: Props) {
  const [features, setFeatures] = useState<Doc[] | null>(null);
  // 문서와 함께 읽어 두 목록이 같은 시점의 서버 상태를 그린다.
  const [archive, setArchive] = useState<FeatureDeletion[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [notAFeature, setNotAFeature] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [reason, setReason] = useState('');
  const [busy, setBusy] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const wide = useWideViewport();
  const [generation, setGeneration] = useState(0);

  useEffect(() => {
    let active = true;
    Promise.all([getAcceptance(id), listDeletions(id)])
      .then(([doc, deletions]) => {
        if (!active) return;
        setFeatures(doc === null ? [] : doc.content.features);
        setArchive(deletions.deletions);
      })
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id, generation]);

  const reload = () => {
    setDeleting(false);
    setReason('');
    setSelected(null);
    setGeneration((g) => g + 1);
  };

  const onConfirmDelete = (key: string) => {
    setBusy(true);
    setActionError(null);
    deleteFeature(id, key, reason)
      .then(reload)
      .catch((e: unknown) => setActionError(messageOf(e)))
      .finally(() => setBusy(false));
  };

  const onRestore = (deletion: string) => {
    setBusy(true);
    setActionError(null);
    restoreFeature(id, deletion)
      .then(reload)
      .catch((e: unknown) => setActionError(messageOf(e)))
      .finally(() => setBusy(false));
  };

  if (features === null) {
    return (
      <main className="screen">
        <Appbar onLeave={onBack} />
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

  if (features.length === 0 && archive.length === 0) {
    return (
      <main className="screen">
        <Appbar onLeave={onBack} />
        <p className="body sm" style={{ marginTop: 22 }} data-testid="acceptance-empty">
          {NOT_GENERATED}
        </p>
      </main>
    );
  }

  if (features.length === 0) {
    return (
      <main className="screen">
        <Appbar onLeave={onBack} />
        <Archive archive={archive} busy={busy} error={actionError} onRestore={onRestore} />
      </main>
    );
  }

  const current = features.find((f) => f.key === selected) ?? features[0];

  return (
    <main className="screen">
      <Appbar onLeave={onBack} />

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

      <details className="disclosure" open={wide} data-testid="scenarios-disclosure">
        <summary className="section-title" style={{ marginTop: 20 }}>
          <span className="caps">인수 시나리오</span>
          <span className="section-action" data-testid="scenario-count">
            {current.scenarios.length}
          </span>
        </summary>

        <div className="collection" style={{ marginTop: 12 }} data-testid="scenario-list">
          {current.scenarios.map((scenario, index) => (
            <Scenario key={scenario.evidence + index} scenario={scenario} index={index} />
          ))}
        </div>
      </details>

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
        <button
          className="btn btn-ghost block"
          type="button"
          onClick={() => setDeleting(true)}
          data-testid="delete-feature"
        >
          이 기능 삭제하기
        </button>
        <button
          className="btn btn-ghost block"
          type="button"
          onClick={() => onOpenHistory(current.key)}
          data-testid="open-history"
        >
          이 기능이 어떻게 변해왔는지
        </button>
      </div>

      {deleting && (
        <div className="notice warn on" style={{ marginTop: 16 }} data-testid="delete-confirm">
          <span>삭제하면 보관소로 옮겨져요. 보관 기간 안에는 되돌릴 수 있습니다.</span>
          <div className="field" style={{ marginTop: 10 }}>
            <label htmlFor="in-delete-reason">삭제 사유</label>
            <input
              type="text"
              id="in-delete-reason"
              value={reason}
              autoComplete="off"
              placeholder="선택 — 다음 분석에서 같은 자리가 다시 잡히면 함께 보여 드려요"
              onChange={(e) => setReason(e.target.value)}
              data-testid="delete-reason"
            />
          </div>
          <button
            className="btn btn-secondary"
            type="button"
            disabled={busy}
            onClick={() => onConfirmDelete(current.key)}
            data-testid="confirm-delete"
          >
            보관소로 옮기기
          </button>
          <button
            className="btn btn-ghost"
            type="button"
            disabled={busy}
            onClick={() => {
              setDeleting(false);
              setReason('');
            }}
            data-testid="cancel-delete"
          >
            그만두기
          </button>
          {actionError !== null && (
            <p className="body sm" data-testid="delete-error">
              {actionError}
            </p>
          )}
        </div>
      )}

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

      {archive.length !== 0 && (
        <Archive archive={archive} busy={busy} error={actionError} onRestore={onRestore} />
      )}
    </main>
  );
}

function Archive({
  archive,
  busy,
  error,
  onRestore,
}: {
  archive: FeatureDeletion[];
  busy: boolean;
  error: string | null;
  onRestore: (deletion: string) => void;
}) {
  const wide = useWideViewport();
  return (
    <>
      <details className="disclosure" open={wide} data-testid="archive-disclosure">
        <summary className="section-title" style={{ marginTop: 22 }}>
          <span className="caps">보관소</span>
          <span className="section-action" data-testid="archive-count">
            {archive.length}
          </span>
        </summary>
        <div className="collection" style={{ marginTop: 12 }} data-testid="archive-list">
          {archive.map((deletion) => (
            <div
              className="scn"
              key={deletion.id}
              data-testid="archived-feature"
              data-key={deletion.key}
              data-restorable={deletion.restorable}
            >
              <span className="sn" data-testid="archived-name">
                {deletion.name}
              </span>
              {deletion.reason !== null && (
                <span className="gwt" data-testid="archived-reason">
                  {deletion.reason}
                </span>
              )}
              <div className="src">
                <span className="esrc" data-testid="restore-until">
                  {dateOf(deletion.restoreUntil)}까지 되돌릴 수 있어요
                </span>
              </div>
              <button
                className="btn btn-ghost"
                type="button"
                disabled={busy || !deletion.restorable}
                onClick={() => onRestore(deletion.id)}
                data-testid="restore-feature"
              >
                되돌리기
              </button>
            </div>
          ))}
        </div>
      </details>
      {error !== null && (
        <p className="body sm" data-testid="archive-error">
          {error}
        </p>
      )}
    </>
  );
}

function dateOf(unix: number): string {
  return new Date(unix * 1000).toISOString().slice(0, 10);
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

const LOADING = '불러오는 중…';
const NOT_GENERATED = '인수 시나리오 생성 단계가 아직 끝나지 않았어요.';

/**
 * The left slot is empty on purpose — the ghost button is the spacer that keeps the
 * title centred under `.appbar`'s `space-between`, not a control.
 */
function Appbar({ onLeave }: { onLeave: () => void }) {
  return (
    <header className="appbar">
      <button className="icon-btn ghost" type="button" aria-hidden="true" tabIndex={-1} />
      <span className="appbar-title">기능 검수</span>
      <button className="icon-btn" type="button" onClick={onLeave} aria-label="나가기">
        ✕
      </button>
    </header>
  );
}
