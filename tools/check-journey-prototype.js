#!/usr/bin/env node
/*
 * 여정 프로토타입 충실도 하네스 — 모델 규칙 5 (c)(d)(e) 의 집행기.
 *
 * 왜 정적 체커로 안 되는가: 파일을 읽어 속성만 세면 배선이 끊긴 버튼과 살아 있는
 * 버튼이 구분되지 않는다. `<button>` 이 있다는 사실과 그 버튼이 다음 단계로
 * 데려간다는 사실은 다른 명제다. 그래서 실제 DOM(jsdom)에서 굴려 확인한다.
 *
 * 기대값의 원천은 **여정 문서**(docs/user-journey/JRN-*.md)다. 페이지가 선언한
 * 값으로 이동하는지를 그 페이지에서 읽어 단언하면 자기참조라 뮤테이션에 통과한다
 * — 단계 순서도 분기 대상도 전부 문서에서 파싱한다.
 *
 * 검사
 *   P1 (f) 딥링크      — #STP-* 로 각 단계에 바로 진입하고, 활성 단계는 항상 1개
 *   P2 (c) 화면 내 전진 — 모든 단계가 제품 화면 안의 행동으로 다음 단계에 도달한다.
 *                        래퍼/보조 레이어의 버튼으로 넘어가는 단계가 하나라도 있으면 실패
 *   P3 (d) 실제 입력   — 텍스트·선택이 진짜 폼 요소이고, 타이핑·선택이 관측 가능한
 *                        상태 변화를 일으킨다. "입력처럼 보이는 div" 패턴은 금지
 *   P4 (e) 상태 변형   — 여정 문서 §4 분기표의 각 행이 선언된 단계로 실제 이동하고,
 *                        그 상황의 상태(오류·경고·재시도)가 화면에 실제로 나타난다
 *   P5 (g) 분기와 끝   — END-* 갈래의 끝에 도달한다
 *   P6 (b) 보조 레이어 — 문서 메타를 담은 <details> 가 기본으로 닫혀 있다
 *   P7 (h) 정적 동작   — 외부 자원은 폰트뿐. 스크립트·스타일 외부 참조 0
 *
 * 여정별로 다른 부분은 **등록부 4종**에 둔다 — 한 여정의 화면 id 를 검사 본문에
 * 직접 쓰면 두 번째 여정 페이지가 생기는 순간 하네스가 그 페이지에서도 같은 id 를
 * 찾다가 죽는다(실제로 그랬다. 아래 INPUT_PROBE 주석 참조).
 *   ARM[jid][stepId]  — 그 단계의 CTA 가 살아나게 하는 최소 입력      (P2)
 *   INPUT_PROBE[jid]  — 타이핑·선택이 상태를 바꾸는지 보는 프로브     (P3)
 *   PRODUCT_PATHS[jid]— §4 각 분기 상황의 제품 화면 경로              (P4)
 *   SECOND_END[jid]   — 정상 경로 말고 또 하나의 갈래의 끝            (P5)
 * 네 등록부 중 하나라도 비면 P0 가 그 페이지를 실패시킨다 — 새 여정이 이관될 때
 * 하네스가 조용히 공허해지는 것을 막는다.
 */
'use strict';

const fs = require('fs');
const path = require('path');
const { JSDOM } = require('jsdom');

const ROOT = path.resolve(__dirname, '..');
const UJ = path.join(ROOT, 'docs', 'user-journey');
const MK = path.join(ROOT, 'docs', 'mockups');

let checks = 0;
const failures = [];

function ok(cond, rule, msg) {
  checks += 1;
  if (!cond) failures.push(`${rule}: ${msg}`);
  return !!cond;
}

/* ── 여정 문서 파싱 (기대값의 원천) ─────────────────────────────── */
function parseJourney(file) {
  const doc = fs.readFileSync(file, 'utf8');
  const stepRe = /^### `(STP-[a-z0-9-]+)` (.+)$/gm;
  const steps = [];
  let m;
  while ((m = stepRe.exec(doc)) !== null) steps.push({ id: m[1], title: m[2].trim() });

  const branches = [];
  const i4 = doc.indexOf('\n## 4.');
  const i5 = doc.indexOf('\n## 5.');
  if (i4 !== -1 && i5 !== -1) {
    for (const line of doc.slice(i4, i5).split('\n')) {
      const t = line.trim();
      if (!t.startsWith('|')) continue;
      const cells = t.replace(/^\||\|$/g, '').split('|').map((c) => c.trim());
      if (cells.length !== 3) continue;
      if (cells.every((c) => /^[-: ]*$/.test(c))) continue;
      const g = cells[2].match(/`(STP-[a-z0-9-]+)`/);
      if (g) branches.push({ situation: cells[0], to: g.group === undefined ? g[1] : g[1] });
    }
  }
  return { steps, branches };
}

/* ── 페이지 로드 ────────────────────────────────────────────────── */
function load(file, hash) {
  const html = fs.readFileSync(file, 'utf8');
  const dom = new JSDOM(html, {
    runScripts: 'dangerously',
    url: 'https://dlddu.github.io/feature-doc/mockups/' + path.basename(file) + (hash || ''),
    pretendToBeVisual: false,
  });
  return dom;
}

function click(win, el) {
  el.dispatchEvent(new win.MouseEvent('click', { bubbles: true, cancelable: true }));
}

function type(win, el, value) {
  el.value = value;
  el.dispatchEvent(new win.Event('input', { bubbles: true }));
}

function change(win, el, value) {
  el.value = value;
  el.dispatchEvent(new win.Event('change', { bubbles: true }));
}

function check(win, el, on) {
  el.checked = on;
  el.dispatchEvent(new win.Event('change', { bubbles: true }));
}

function active(doc) {
  return doc.body.getAttribute('data-active-section');
}

function visible(el) {
  return !!el && el.classList.contains('on') && el.style.display !== 'none';
}

/* ── 페이지별 제품 경로 등록 ─────────────────────────────────────
   (c) 는 "각 단계 화면 안의 주요 행동을 눌러 다음 단계에 도달" 이라, 그 단계에서
   무엇을 입력해야 CTA 가 살아나는지는 화면마다 다르다. 그 최소 입력을 여기에
   등록한다. 등록이 없는 여정 페이지는 통과시키지 않는다 — 새 여정이 이관될 때
   하네스가 조용히 공허해지는 것을 막는다.                                       */
const ARM = {
  'JRN-connect-repo': {
    // 단계별로 "그 화면 안에서 실제 사용자가 하는 최소 입력"
    'STP-sign-in': () => {},
    'STP-grant-repo-access': (win, doc) => {
      check(win, doc.querySelector('.repo-check[value="dlddu/payments-api"]'), true);
    },
    'STP-register-llm-key': (win, doc) => {
      change(win, doc.getElementById('in-provider'), 'anthropic');
      type(win, doc.getElementById('in-key'), 'sk-ant-api03-demo-key');
    },
    'STP-pick-target': (win, doc) => {
      type(win, doc.getElementById('in-repo'), 'github.com/dlddu/payments-api');
      change(win, doc.getElementById('in-branch'), 'main');
    },
    'STP-confirm-cost': () => {},
  },
  'JRN-discover-features': {
    // 단계별로 "그 화면 안에서 실제 사용자가 하는 최소 입력"
    'STP-leave-and-return': () => {},
    'STP-review-landscape': () => {},
    'STP-tune-strategy': (win, doc) => {
      type(win, doc.getElementById('in-entrypoint'), 'cmd/admin-cli');
      click(win, doc.getElementById('btn-add-entrypoint'));
    },
    'STP-sift-candidates': (win, doc) => {
      // 후보 4건을 전부 결정해야 다음으로 넘어간다(미결정 0건이 완료 기준).
      for (const b of [...doc.querySelectorAll('.cand [data-decide="approve"]')]) click(win, b);
    },
    'STP-add-missing': (win, doc) => {
      type(win, doc.getElementById('in-newfeature'), '주간 사용량 리포트 메일 발송');
      click(win, doc.getElementById('btn-draft'));
    },
  },
};

/* ── 페이지별 입력 프로브 등록 (P3) ──────────────────────────────
   "타이핑·선택이 관측 가능한 상태 변화를 일으키는가"는 그 여정의 어떤 필드가
   무엇을 검증하는지에 달려 있어 여정마다 다르다. 여정 무관한 부분(폼 요소가
   0개가 아닌가 · 가짜 필드 패턴이 없는가)만 공통으로 두고, 나머지는 여기에
   등록한다. 등록이 없으면 P0 가 실패시킨다.

   ⚠️ 이 등록부가 없던 시절, P3 는 `JRN-connect-repo` 전용 id(`#in-key` 등)를
   여정 조건 없이 만졌다 — 두 번째 여정 페이지가 생기는 순간 null 에 `.value` 를
   대입해 하네스가 uncaught TypeError 로 통째 죽었다(단언 보고조차 못 한다).   */
const INPUT_PROBE = {
  'JRN-connect-repo': (win, doc, at, ok) => {
    at('#STP-register-llm-key');
    const key = doc.getElementById('in-key');
    ok(key && key.tagName === 'INPUT', 'P3', `API Key 가 실제 <input> 이 아니다`);
    if (!key) return;
    type(win, key, 'not-a-key');
    ok(key.value === 'not-a-key', 'P3', `입력한 값이 필드에 반영되지 않는다`);
    ok(doc.getElementById('btn-savekey').disabled === true, 'P3',
       `잘못된 키인데 저장 버튼이 살아 있다 (입력 검증이 죽어 있다)`);
    ok(visible(doc.getElementById('key-error')), 'P3',
       `잘못된 키에 대한 검증 표시가 나타나지 않는다`);
    type(win, key, 'sk-ant-api03-demo-key');
    ok(doc.getElementById('btn-savekey').disabled === false, 'P3',
       `올바른 키를 입력해도 저장 버튼이 살아나지 않는다`);

    // 선택(select)도 동작하는가
    const prov = doc.getElementById('in-provider');
    ok(prov && prov.tagName === 'SELECT', 'P3', `Provider 가 실제 <select> 가 아니다`);
    // google 키는 `AIza` 로 시작하므로 위의 anthropic 키와 접두사가 겹치지 않는다.
    // (openai 의 `sk-` 는 `sk-ant-` 의 접두사라 이 자리에서 쓰면 검사가 공허해진다.)
    change(win, prov, 'google');
    ok(doc.getElementById('btn-savekey').disabled === true, 'P3',
       `provider 를 바꿨는데 키 검증이 따라 바뀌지 않는다 (선택이 죽어 있다)`);
  },
  'JRN-discover-features': (win, doc, at, ok) => {
    // ① 거부 사유(textarea) — 비어 있으면 거부를 확정할 수 없다(F7 재발 방지).
    at('#STP-sift-candidates');
    const first = doc.querySelector('.cand');
    click(win, first.querySelector('[data-decide="reject"]'));
    const why = doc.getElementById('in-reject-why');
    ok(why && why.tagName === 'TEXTAREA', 'P3', `거부 사유가 실제 <textarea> 가 아니다`);
    if (!why) return;
    ok(doc.getElementById('btn-reject-confirm').disabled === true, 'P3',
       `사유가 비었는데 거부 확정 버튼이 살아 있다 (입력 검증이 죽어 있다)`);
    ok(visible(doc.getElementById('reject-error')), 'P3',
       `빈 사유에 대한 검증 표시가 나타나지 않는다`);
    type(win, why, '내부 디버그용 엔드포인트라 사용자 기능이 아님');
    ok(why.value === '내부 디버그용 엔드포인트라 사용자 기능이 아님', 'P3',
       `입력한 값이 필드에 반영되지 않는다`);
    ok(doc.getElementById('btn-reject-confirm').disabled === false, 'P3',
       `사유를 적어도 거부 확정 버튼이 살아나지 않는다`);

    // ② 필터(select) — 선택이 실제로 목록을 줄인다.
    const shown = () => [...doc.querySelectorAll('.cand')].filter((c) => c.style.display !== 'none').length;
    const filt = doc.getElementById('in-filter');
    ok(filt && filt.tagName === 'SELECT', 'P3', `후보 필터가 실제 <select> 가 아니다`);
    const before = shown();
    change(win, filt, 'undecided');
    ok(shown() < before, 'P3', `필터를 바꿨는데 목록이 그대로다 (선택이 죽어 있다)`);

    // ③ 수동 추가(input) — 빈 이름으로는 초안을 만들 수 없다.
    at('#STP-add-missing');
    const nf = doc.getElementById('in-newfeature');
    ok(nf && nf.tagName === 'INPUT', 'P3', `기능명이 실제 <input> 이 아니다`);
    ok(doc.getElementById('btn-draft').disabled === true, 'P3',
       `기능명이 비었는데 초안 생성 버튼이 살아 있다`);
    type(win, nf, '주간 사용량 리포트 메일 발송');
    ok(doc.getElementById('btn-draft').disabled === false, 'P3',
       `기능명을 입력해도 초안 생성 버튼이 살아나지 않는다`);
  },
};

/* ── 페이지별 두 번째 갈래의 끝 등록 (P5) ────────────────────────
   정상 경로의 끝은 P2 가 마지막 단계의 전진으로 확인한다. 그 밖에 최소 하나의
   **다른** 갈래가 END-* 로 끝나는지를 여기서 본다 — 갈래가 하나뿐인 프로토타입은
   규칙 5(g) 의 "각 갈래의 끝이 표현된다"를 충족하지 못한다.                     */
const SECOND_END = {
  'JRN-connect-repo': { name: '저장 후 종료', at: '#STP-confirm-cost', run: (win, doc, ok) => {
    const later = doc.getElementById('btn-later');
    if (!ok(later, 'P5', `대안 갈래(저장 후 종료)의 행동이 없다`)) return;
    click(win, later);
  } },
  'JRN-discover-features': { name: '누적 비용 초과로 중단', at: '#STP-sift-candidates', run: (win, doc, ok) => {
    const stop = doc.getElementById('btn-stop-cost');
    if (!ok(stop, 'P5', `대안 갈래(누적 비용 초과 중단)의 행동이 없다`)) return;
    click(win, stop);
  } },
};

/* 분기 상황이 화면에 실제로 나타나는지 — 제품 화면 안의 경로로 확인한다.
   각 항목은 여정 문서 §4 의 행 순서에 대응하며, 기대 도착 단계는 문서에서 온다. */
const PRODUCT_PATHS = {
  'JRN-connect-repo': [
    { name: '재방문 사용자', run: (win, doc) => {
        click(win, doc.getElementById('btn-returning'));
        return { landed: active(doc), evidence: doc.getElementById('home-count').textContent === '2' };
      } },
    { name: 'App 범위 밖 저장소', run: (win, doc) => {
        click(win, doc.getElementById('btn-returning'));
        type(win, doc.getElementById('in-repo'), 'github.com/acme/secret-svc');
        const shown = visible(doc.getElementById('repo-outside'));
        const blocked = doc.getElementById('btn-pick').disabled === true;
        click(win, doc.getElementById('btn-add-scope'));
        return { landed: active(doc), evidence: shown && blocked };
      } },
    { name: 'URL 오타', run: (win, doc) => {
        click(win, doc.getElementById('btn-returning'));
        type(win, doc.getElementById('in-repo'), 'github.com/oops');
        return { landed: active(doc),
                 evidence: visible(doc.getElementById('repo-bad')) &&
                           doc.getElementById('btn-pick').disabled === true };
      } },
    { name: 'LLM Key 미등록', run: (win, doc) => {
        click(win, doc.getElementById('btn-signin'));
        check(win, doc.querySelector('.repo-check[value="dlddu/payments-api"]'), true);
        click(win, doc.getElementById('btn-install'));
        click(win, doc.getElementById('btn-key-later'));
        type(win, doc.getElementById('in-repo'), 'github.com/dlddu/payments-api');
        const shown = visible(doc.getElementById('repo-nokey'));
        click(win, doc.getElementById('btn-go-key'));
        return { landed: active(doc),
                 evidence: shown && visible(doc.getElementById('key-resume')) };
      } },
    { name: '큐 등록 실패', run: (win, doc) => {
        const btn = [...doc.querySelectorAll('[data-scenario="queuefail"]')][0];
        click(win, btn);                       // 사용자가 일으킬 수 없는 상태를 장전
        click(win, doc.getElementById('btn-start'));
        const failed = visible(doc.getElementById('queue-failed'));
        const stayed = active(doc);
        click(win, doc.getElementById('btn-retry'));
        const cleared = !visible(doc.getElementById('queue-failed'));
        click(win, doc.getElementById('btn-start'));
        const recovered = active(doc) === 'END-analysis-queued';
        return { landed: stayed, evidence: failed && cleared && recovered };
      } },
    { name: '키 등록 중 이탈', run: (win, doc) => {
        click(win, doc.getElementById('btn-signin'));
        check(win, doc.querySelector('.repo-check[value="dlddu/payments-api"]'), true);
        click(win, doc.getElementById('btn-install'));
        click(win, doc.getElementById('btn-key-later'));
        click(win, doc.getElementById('btn-go-key'));
        return { landed: active(doc),
                 evidence: visible(doc.getElementById('key-resume')) };
      } },
    { name: '로그아웃', run: (win, doc) => {
        click(win, doc.getElementById('btn-returning'));
        click(win, doc.getElementById('btn-logout'));
        return { landed: active(doc),
                 evidence: doc.getElementById('home-count').textContent === '0' };
      } },
  ],
};

/* ── 본 검사 ────────────────────────────────────────────────────── */
const pages = fs.readdirSync(MK).filter((f) => /^JRN-[a-z0-9-]+\.html$/.test(f)).sort();
if (pages.length === 0) failures.push('P0: docs/mockups/JRN-*.html 여정 페이지를 하나도 찾지 못했다');

for (const file of pages) {
  const jid = file.replace(/\.html$/, '');
  const docPath = path.join(UJ, jid + '.md');
  if (!fs.existsSync(docPath)) {
    failures.push(`P0: \`${file}\` 에 대응하는 여정 문서 ${jid}.md 가 없다`);
    continue;
  }
  const jr = parseJourney(docPath);
  const pagePath = path.join(MK, file);

  const missing = [['ARM', ARM], ['PRODUCT_PATHS', PRODUCT_PATHS],
                   ['INPUT_PROBE', INPUT_PROBE], ['SECOND_END', SECOND_END]]
    .filter(([, reg]) => !reg[jid]).map(([n]) => n);
  if (missing.length) {
    failures.push(`P0: \`${file}\` 의 제품 경로가 tools/check-journey-prototype.js 에 등록되지 않았다 ` +
                  `(누락: ${missing.join(', ')}) ` +
                  '— 등록 없이는 규칙 5(c)(d)(e) 를 굴려 볼 수 없다 (하네스가 공허해진다)');
    continue;
  }

  /* P6 (b) 보조 레이어가 기본으로 닫혀 있다 */
  {
    const { window } = load(pagePath);
    const doc = window.document;
    const details = doc.querySelector('details[data-meta="doc"]');
    ok(details, 'P6', `${file}: 문서 메타 보조 레이어 <details data-meta="doc"> 가 없다`);
    if (details) ok(details.open === false, 'P6', `${file}: 보조 레이어가 열린 채로 시작한다 (규칙 5b)`);
    ok(doc.querySelectorAll('.stp.on').length === 1, 'P6',
       `${file}: 진입 직후 활성 화면이 1개가 아니다 (${doc.querySelectorAll('.stp.on').length}개)`);
    window.close();
  }

  /* P7 (h) 외부 자원은 폰트뿐 */
  {
    const html = fs.readFileSync(pagePath, 'utf8');
    const ext = [...html.matchAll(/(?:src|href)="(https?:\/\/[^"]+)"/g)].map((m) => m[1]);
    const bad = ext.filter((u) => !/^https:\/\/fonts\.(googleapis|gstatic)\.com\//.test(u));
    ok(bad.length === 0, 'P7', `${file}: 폰트 외 외부 자원을 참조한다: ${bad.join(', ')}`);
    ok(!/<script\s[^>]*src=/i.test(html), 'P7', `${file}: 외부 스크립트를 불러온다 (정적 동작 위반)`);
  }

  /* P1 (f) 딥링크 — 문서의 각 단계로 바로 진입 */
  for (const st of jr.steps) {
    const { window } = load(pagePath, '#' + st.id);
    const doc = window.document;
    ok(active(doc) === st.id, 'P1',
       `${file}: #${st.id} 딥링크로 진입했는데 활성 화면이 ${active(doc)} 다`);
    ok(doc.querySelectorAll('.stp.on').length === 1, 'P1',
       `${file}: #${st.id} 진입 시 활성 화면이 1개가 아니다`);
    window.close();
  }

  /* P1 (f) 세션 안에서 해시를 바꿔도 따라와야 한다. 진입 시 해시만 읽고 마는
     페이지는 이미 열려 있는 탭에서 링크를 눌렀을 때 화면이 그대로 멈춘다 —
     뮤테이션(hashchange 핸들러 제거)이 잡아낸 사각이다. */
  {
    const { window } = load(pagePath);
    const doc = window.document;
    for (const st of [...jr.steps].reverse()) {
      window.location.hash = '#' + st.id;
      window.dispatchEvent(new window.HashChangeEvent('hashchange'));
      ok(active(doc) === st.id, 'P1',
         `${file}: 세션 중 해시를 #${st.id} 로 바꿨는데 화면이 ${active(doc)} 에 머문다`);
    }
    window.close();
  }

  /* P3 (d) 실제 입력 요소 — 가짜 필드 금지 + 타이핑이 상태를 바꾼다 */
  {
    const { window } = load(pagePath);
    const doc = window.document;
    const controls = doc.querySelectorAll('input, select, textarea');
    ok(controls.length > 0, 'P3', `${file}: 페이지 전체에 폼 요소가 0개다 (규칙 5d)`);
    ok(doc.querySelectorAll('.input .val').length === 0, 'P3',
       `${file}: "입력처럼 보이는 비대화형 요소"(.input > .val) 패턴이 남아 있다 (규칙 5d)`);

    // 타이핑·선택이 실제로 관측 가능한 상태 변화를 만드는가 — 어떤 필드가 무엇을
    // 검증하는지는 여정마다 다르므로 등록된 프로브가 굴린다.
    const at = (hash) => {
      window.location.hash = hash;
      window.dispatchEvent(new window.HashChangeEvent('hashchange'));
    };
    INPUT_PROBE[jid](window, doc, at, (cond, rule, msg) => ok(cond, rule, `${file}: ${msg}`));
    window.close();
  }

  /* P2 (c) 모든 단계가 화면 안의 행동으로 전진한다 */
  {
    const { window } = load(pagePath);
    const doc = window.document;
    for (let i = 0; i < jr.steps.length; i += 1) {
      const cur = jr.steps[i].id;
      const next = i + 1 < jr.steps.length ? jr.steps[i + 1].id : null;
      ok(active(doc) === cur, 'P2',
         `${file}: ${cur} 에 있어야 하는데 ${active(doc)} 다 (앞 단계의 전진이 어긋났다)`);

      const section = doc.getElementById(cur);
      const cta = section && section.querySelector('.frame [data-cta="advance"]');
      if (!ok(cta, 'P2', `${file}: ${cur} 의 제품 화면 안에 전진 행동(data-cta="advance")이 없다`)) break;
      ok(!cta.closest('details'), 'P2',
         `${file}: ${cur} 의 전진 행동이 보조 레이어 안에 있다 — 제품 화면 안이어야 한다 (규칙 5c)`);

      ARM[jid][cur](window, doc);
      ok(cta.disabled !== true, 'P2',
         `${file}: ${cur} 의 화면 내 입력을 마쳤는데 전진 행동이 비활성이다`);
      click(window, cta);

      if (next) {
        ok(active(doc) === next, 'P2',
           `${file}: ${cur} 의 화면 내 행동으로 ${next} 에 도달하지 못했다 (도착: ${active(doc)})`);
      } else {
        /* P5 (g) 마지막 단계의 전진은 갈래의 끝으로 */
        ok(/^END-/.test(active(doc) || ''), 'P5',
           `${file}: 마지막 단계의 행동이 갈래의 끝(END-*)으로 이어지지 않는다 (도착: ${active(doc)})`);
      }
    }
    window.close();
  }

  /* P4 (e) 여정 문서 §4 의 각 분기가 선언된 단계로 실제 이동한다 */
  {
    ok(jr.branches.length > 0, 'P4', `${jid}.md §4 분기표를 읽지 못했다`);
    const { window } = load(pagePath);
    const doc = window.document;
    const btns = [...doc.querySelectorAll('.branches [data-goto]')];
    ok(btns.length === jr.branches.length, 'P4',
       `${file}: 분기 항목 ${btns.length}개가 여정 문서 §4 의 ${jr.branches.length}행과 다르다`);
    window.close();

    for (let i = 0; i < Math.min(btns.length, jr.branches.length); i += 1) {
      const want = jr.branches[i].to;                    // ← 기대값은 여정 문서에서
      const w = load(pagePath).window;
      const d = w.document;
      click(w, [...d.querySelectorAll('.branches [data-goto]')][i]);
      ok(active(d) === want, 'P4',
         `${file}: 분기 ${i + 1} "${jr.branches[i].situation}" 이 ${want} 로 가지 않고 ${active(d)} 로 갔다`);
      w.close();
    }

    /* 그리고 그 상황이 제품 화면 안에서 실제로 재현되는가 */
    const paths = PRODUCT_PATHS[jid];
    ok(paths.length === jr.branches.length, 'P4',
       `${file}: 등록된 제품 경로 ${paths.length}개가 여정 문서 §4 의 ${jr.branches.length}행과 다르다`);
    for (let i = 0; i < Math.min(paths.length, jr.branches.length); i += 1) {
      const want = jr.branches[i].to;                    // ← 여기서도 문서가 기대값
      const w = load(pagePath).window;
      const r = paths[i].run(w, w.document);
      ok(r.landed === want, 'P4',
         `${file}: 제품 경로 "${paths[i].name}" 이 ${want} 로 가지 않고 ${r.landed} 로 갔다`);
      ok(r.evidence === true, 'P4',
         `${file}: 제품 경로 "${paths[i].name}" 에서 그 상황의 화면 상태가 나타나지 않는다 (규칙 5e)`);
      w.close();
    }
  }

  /* P5 (g) 두 번째 갈래의 끝 — 어떤 행동이 대안 갈래인지는 여정마다 다르다 */
  {
    const spec = SECOND_END[jid];
    const { window } = load(pagePath, spec.at);
    const doc = window.document;
    spec.run(window, doc, (cond, rule, msg) => ok(cond, rule, `${file}: ${msg}`));
    ok(/^END-/.test(active(doc) || ''), 'P5',
       `${file}: 대안 갈래 "${spec.name}" 이 END-* 로 끝나지 않는다 (도착: ${active(doc)})`);
    window.close();
  }
}

/* ── 보고 ───────────────────────────────────────────────────────── */
console.log(`여정 프로토타입 ${pages.length}개 · 단언 ${checks}건 실행`);
if (checks === 0) {
  console.error('\n실패: 단언을 하나도 실행하지 않았다 — 게이트가 공허하다');
  process.exit(1);
}
if (failures.length) {
  console.error(`\n실패 ${failures.length}건:`);
  for (const f of failures) console.error('  ✗ ' + f);
  process.exit(1);
}
console.log('\n통과 — 규칙 5 (b)(c)(d)(e)(f)(g)(h) 이상 없음');
