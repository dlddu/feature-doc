export function NoAccess() {
  return (
    <div className="notice warn" data-testid="no-access" style={{ marginTop: 12 }}>
      이 저장소는 아직 볼 수 없어요. 목록에도, 상세에도 나오지 않습니다 — 잘못 보내신 링크일 수도 있어요.
    </div>
  );
}
