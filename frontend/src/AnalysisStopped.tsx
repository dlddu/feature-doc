import { formatCost } from './format';

type Props = {
  spentCents: number;
  onBackToDecisions: () => void;
};

export function AnalysisStopped({ spentCents, onBackToDecisions }: Props) {
  return (
    <>
      <div className="card end-card" data-testid="cost-abandoned" style={{ marginTop: 170 }}>
        <div className="em">◼</div>
        <h1 className="h-display" style={{ marginTop: 14 }}>
          여기서 멈췄어요
        </h1>
        <p className="h-display-sub" style={{ marginTop: 8 }}>
          더 이상 비용이 발생하지 않습니다. 지금까지의 결정은 저장돼 있어서, 원할 때 이어서 진행할 수 있어요.
        </p>
        <span className="badge warn" style={{ marginTop: 16 }} data-testid="stopped-cost">
          <span className="dot" />
          중단 · 누적 <span>{formatCost(spentCents)}</span>
        </span>
      </div>
      <div className="stack" style={{ marginTop: 26 }}>
        <button
          className="btn btn-secondary block"
          type="button"
          data-testid="back-to-decisions"
          onClick={onBackToDecisions}
        >
          결정 화면으로 돌아가기
        </button>
      </div>
    </>
  );
}
