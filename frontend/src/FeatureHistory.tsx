// 여정 `docs/user-journey/JRN-restore-history.md` 의 구현 — 세 단계를 한 화면에 접었다
// (`STP-open-history` 목록 · `STP-trace-origin` 출처 · `STP-restore-point` 미리보기와 복원).
//
// **목업이 없다.** 그 여정은 시각화 산출물이 없는 「수용된 위험」이고, 문서 권위 순서
// (가치 > PRD > 테스트 > 여정 > 목업 > 구현)에서 목업이 없으면 여정 문서가 이긴다.
// 그래서 이 파일에는 다른 화면들이 첫 줄에 다는 목업 매핑 주석이 없다 — 없는 목업을
// 가리키면 `tools/check-mockup-render.py` 가 유령 화면으로 잡는다. 그 사실은 숨기지
// 않고 `docs/doc-tracker/` 의 「수용된 위험」과 「활성 대조 대상」 절에 적혀 있다.

import { useEffect, useState } from 'react';
import { getHistory, getHistoryPoint, restoreHistoryPoint } from './api';
import type { FeatureHistory as History, HistoryEntry, HistoryPreview } from './api';

function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** 출처 셋은 AC3.4 의 어휘 그대로다 — 뭉뚱그리면 원인을 특정할 수 없어 복원이 도박이 된다. */
function sourceLabel(entry: HistoryEntry): string {
  if (entry.kind === 'restore') return '사용자 · 시점 복원';
  if (entry.source === 'user_llm') return '사용자 · 도움받아 작성';
  if (entry.source === 'user_direct') return '사용자 · 직접';
  return '자동 분석';
}

function titleOf(entry: HistoryEntry): string {
  if (entry.kind === 'auto') return '자동 분석이 이 문서를 썼어요';
  if (entry.kind === 'restore') return '이 시점으로 되돌렸어요';
  return entry.request ?? '';
}

function dateOf(unix: number): string {
  if (unix === 0) return '';
  return new Date(unix * 1000).toISOString().slice(0, 16).replace('T', ' ');
}

type Props = {
  id: string;
  featureKey: string;
  onBack: () => void;
};

export function FeatureHistory({ id, featureKey, onBack }: Props) {
  const [history, setHistory] = useState<History | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [picked, setPicked] = useState<string | null>(null);
  const [preview, setPreview] = useState<HistoryPreview | null>(null);
  const [busy, setBusy] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [generation, setGeneration] = useState(0);

  useEffect(() => {
    let active = true;
    getHistory(id, featureKey)
      .then((next) => active && setHistory(next))
      .catch((e: unknown) => active && setError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id, featureKey, generation]);

  // 고른 시점의 상태는 서버가 계산한다 — 화면이 짐작한 값으로 되돌리기를 보여 주면
  // 「미리 본 것」과 「실제로 되는 것」이 갈린다.
  useEffect(() => {
    if (picked === null) {
      setPreview(null);
      return;
    }
    let active = true;
    getHistoryPoint(id, featureKey, picked)
      .then((next) => active && setPreview(next))
      .catch((e: unknown) => active && setActionError(messageOf(e)));
    return () => {
      active = false;
    };
  }, [id, featureKey, picked, generation]);

  const restore = () => {
    if (picked === null) return;
    setBusy(true);
    setActionError(null);
    restoreHistoryPoint(id, featureKey, picked)
      .then(() => {
        setPicked(null);
        setGeneration((g) => g + 1);
      })
      .catch((e: unknown) => setActionError(messageOf(e)))
      .finally(() => setBusy(false));
  };

  if (history === null) {
    return (
      <main className="screen">
        <Appbar onBack={onBack} />
        <p className="body sm" style={{ marginTop: 22 }} data-testid="history-error">
          {error ?? '불러오는 중…'}
        </p>
      </main>
    );
  }

  return (
    <main className="screen">
      <Appbar onBack={onBack} />

      <div style={{ marginTop: 18 }}>
        <h1 className="h-display" data-testid="history-feature">
          {history.featureName ?? history.featureKey}
        </h1>
        <p className="h-display-sub">
          이 기능 문서가 어떻게 변해왔는지예요. 각 변경이 어디서 왔는지 — 자동 분석인지, 도움받아
          고친 것인지, 직접 고친 것인지 — 를 함께 적었습니다. 되돌리기 전에 무엇이 달라지는지 먼저
          보여 드려요.
        </p>
      </div>

      <div className="section-title" style={{ marginTop: 20 }}>
        <span className="caps">변경 이력</span>
        <span className="section-action" data-testid="history-count">
          {history.entries.length}
        </span>
      </div>

      <div className="collection" style={{ marginTop: 12 }} data-testid="history-list">
        {history.entries
          .slice()
          .reverse()
          .map((entry) => (
            <div
              className="scn"
              key={entry.id}
              data-testid="history-entry"
              data-entry={entry.id}
              data-kind={entry.kind}
              data-source={entry.source}
              data-standing={entry.standing}
              data-current={entry.current}
            >
              <span className="sn" data-testid="entry-source">
                {sourceLabel(entry)}
              </span>
              <span className="gwt" data-testid="entry-title">
                {titleOf(entry)}
              </span>
              <div className="src">
                <span className="esrc" data-testid="entry-at">
                  {dateOf(entry.at)}
                </span>
                {entry.current && (
                  <span className="tag success" data-testid="entry-current">
                    <span className="dot" />
                    지금 문서
                  </span>
                )}
                {!entry.standing && (
                  <span className="esrc" data-testid="entry-cut">
                    되돌려서 지금은 서 있지 않아요
                  </span>
                )}
                {entry.carriedFrom !== null && (
                  <span className="esrc" data-testid="entry-carried">
                    앞선 분석에서 이어받았어요
                  </span>
                )}
              </div>
              <button
                className="btn btn-ghost"
                type="button"
                onClick={() => setPicked(entry.id)}
                data-testid="pick-point"
              >
                이 시점 미리 보기
              </button>
            </div>
          ))}
      </div>

      {picked !== null && preview !== null && (
        <div className="card" style={{ marginTop: 16 }} data-testid="restore-preview">
          <span className="caps">되돌리면 이렇게 됩니다</span>
          <div className="code" style={{ marginTop: 10 }} data-testid="preview-diff">
            {preview.lines.map((line) => (
              <div className="ln" key={line.mark + line.text}>
                <span
                  className={line.mark === '+' ? 'add' : 'del'}
                  data-testid={line.mark === '+' ? 'preview-added' : 'preview-removed'}
                >
                  {line.mark}
                  {' '}
                  {line.text}
                </span>
              </div>
            ))}
          </div>
          {preview.isCurrent && (
            <p className="body sm" data-testid="preview-same">
              지금 문서가 이미 그 시점이에요. 되돌릴 것이 없습니다.
            </p>
          )}
          {actionError !== null && (
            <p className="body sm" data-testid="restore-error">
              {actionError}
            </p>
          )}
          <div className="stack" style={{ marginTop: 12 }}>
            <button
              className="btn btn-primary block"
              type="button"
              disabled={busy || preview.isCurrent}
              onClick={restore}
              data-testid="restore-point"
            >
              이 시점으로 복원
            </button>
            <button
              className="btn btn-ghost block"
              type="button"
              onClick={() => setPicked(null)}
              data-testid="keep-current"
            >
              확인만 하고 그대로 둘게요
            </button>
          </div>
        </div>
      )}

      <div className="section-title" style={{ marginTop: 22 }}>
        <span className="caps">지금 문서</span>
        <span className="section-action" data-testid="current-count">
          {history.scenarios.length}
        </span>
      </div>
      <div className="collection" style={{ marginTop: 12 }} data-testid="current-scenarios">
        {history.scenarios.map((scenario, index) => (
          <div className="scn" key={scenario.then + index} data-testid="current-scenario">
            <span className="sn">{'시나리오 '}{index + 1}</span>
            <span className="gwt">
              <span className="k">주어진 상황</span> <span>{scenario.given}</span>
              <br />
              <span className="k">이럴 때</span> <span>{scenario.when}</span>
              <br />
              <span className="k">이렇게 됩니다</span> <span>{scenario.then}</span>
            </span>
          </div>
        ))}
      </div>
    </main>
  );
}

function Appbar({ onBack }: { onBack: () => void }) {
  return (
    <header className="appbar">
      <button className="icon-btn" type="button" onClick={onBack} aria-label="back">
        ‹
      </button>
      <span className="appbar-title">변경 이력</span>
      <button className="icon-btn ghost" type="button" aria-hidden="true" tabIndex={-1} />
    </header>
  );
}
