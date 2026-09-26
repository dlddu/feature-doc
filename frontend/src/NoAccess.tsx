// 볼 수 없는 저장소 — 목업 JRN-understand-feature.html#no-access 의 상태 블록.
//
// 화면이 아니라 상태 블록이므로 상단에 목업 *매핑* 주석을 붙이지 않는다 — M1 이 화면을
// 발견하는 표식은 `docs/mockups/<파일>.html` 경로 리터럴이고, 그것을 적으면 `STP-open-shared`
// 가 활성 단계가 되어 그 단계의 카피 전량이 M3A 의 분모로 들어온다.
// 대신 `data-testid` 가 목업 `id` 와 짝지어져 M9·M10 의 대조 단위가 된다 — 태그·클래스 집합·
// 인라인 선언·문면이 목업과 같아야 통과하므로, 색이나 간격이 흘러도 게이트가 잡는다.
//
// 여정 JRN-understand-feature §4 분기표: 권한 없는 저장소 링크는 목록·상세 모두 미노출이고
// 여기서 여정이 끝난다. 목록·상세의 미노출은 서버가 이미 세웠고(소유자 스코프 질의의 404),
// 비어 있던 것은 그 사실을 사용자에게 말하는 이 자리 하나였다.
export function NoAccess() {
  return (
    <div className="notice warn" data-testid="no-access" style={{ marginTop: 12 }}>
      이 저장소는 아직 볼 수 없어요. 목록에도, 상세에도 나오지 않습니다 — 잘못 보내신 링크일 수도 있어요.
    </div>
  );
}
