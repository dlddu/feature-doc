#!/usr/bin/env node
/**
 * Generate 10 SVG wireframes (one per screen).
 * Convention:
 *   - 393 × 844 (iOS mobile)
 *   - white bg, dark hairline strokes
 *   - shapes only: rect / line / text / circle
 *   - text uses system sans + mono fallback (small placeholder)
 *   - fills: #fff (canvas), #f4f4f4 (plate), #e8e8e8 (block), #d4d4d4 (label fill)
 *   - strokes: #1a1a1a hairline (1px)
 *   - all annotations greyscale; no color hints
 */

const fs = require('fs');
const path = require('path');

const W = 393, H = 844;
const PAD = 22;

/* ── primitives ──────────────────────────────────────────── */
const STROKE = '#1a1a1a';
const STROKE_LIGHT = '#bdbdbd';
const STROKE_GHOST = '#e0e0e0';
const FILL_PLATE = '#f4f4f4';
const FILL_BLOCK = '#ebebeb';
const FILL_LABEL = '#d4d4d4';
const TEXT_DARK = '#1a1a1a';
const TEXT_MUTE = '#757575';
const TEXT_GHOST = '#a8a8a8';

const SANS = "ui-sans-serif, system-ui, -apple-system, 'Segoe UI', sans-serif";
const MONO = "ui-monospace, 'JetBrains Mono', 'SF Mono', Menlo, monospace";

/* helpers */
const rect = (x, y, w, h, opts = {}) => {
  const { fill = 'none', stroke = STROKE, sw = 1, rx = 0, dash } = opts;
  const da = dash ? ` stroke-dasharray="${dash}"` : '';
  return `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${rx}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"${da}/>`;
};
const line = (x1, y1, x2, y2, opts = {}) => {
  const { stroke = STROKE_LIGHT, sw = 1, dash } = opts;
  const da = dash ? ` stroke-dasharray="${dash}"` : '';
  return `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${stroke}" stroke-width="${sw}"${da}/>`;
};
const circle = (cx, cy, r, opts = {}) => {
  const { fill = 'none', stroke = STROKE, sw = 1, dash } = opts;
  const da = dash ? ` stroke-dasharray="${dash}"` : '';
  return `<circle cx="${cx}" cy="${cy}" r="${r}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"${da}/>`;
};
const text = (x, y, str, opts = {}) => {
  const {
    size = 11,
    family = SANS,
    fill = TEXT_DARK,
    weight = 400,
    anchor = 'start',
    upper = false,
    tracking = 0,
  } = opts;
  let t = upper ? str.toUpperCase() : str;
  // escape XML entities — only ampersand needs escaping for plain text
  t = t.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  const ls = tracking ? ` letter-spacing="${tracking}"` : '';
  return `<text x="${x}" y="${y}" font-size="${size}" font-family="${family}" fill="${fill}" font-weight="${weight}" text-anchor="${anchor}"${ls}>${t}</text>`;
};

/* compositions */
// ghost line (placeholder for body text)
const placeholderLine = (x, y, w, opts = {}) => {
  const { h = 6, fill = FILL_BLOCK } = opts;
  return rect(x, y, w, h, { fill, stroke: 'none', rx: 2 });
};

// labelled placeholder: filled block representing a real text element
const blockText = (x, y, w, h, label, opts = {}) => {
  const { caps = false, family = SANS, size = 11 } = opts;
  return [
    rect(x, y, w, h, { fill: FILL_PLATE, stroke: STROKE_GHOST, rx: 4 }),
    text(x + 8, y + h / 2 + size / 3, label, { size, fill: TEXT_MUTE, family, upper: caps, tracking: caps ? 1 : 0 }),
  ].join('');
};

// status bar + home indicator (shared)
const chrome = () => [
  // status bar
  text(28, 32, '9:41', { size: 13, weight: 600 }),
  // signal triangle
  rect(330, 26, 14, 8, { fill: TEXT_DARK, stroke: 'none', rx: 1 }),
  // wifi
  rect(348, 26, 12, 8, { fill: TEXT_DARK, stroke: 'none', rx: 1 }),
  // battery
  rect(364, 26, 18, 8, { fill: 'none', stroke: TEXT_DARK, rx: 2 }),
  rect(366, 28, 14, 4, { fill: TEXT_DARK, stroke: 'none' }),
  // home indicator
  rect(393 / 2 - 67, 824, 134, 5, { fill: '#444', stroke: 'none', rx: 2.5 }),
].join('');

// device frame outline
const frame = () => rect(0, 0, W, H, { fill: '#fff', stroke: STROKE, rx: 38, sw: 1.5 });

// app bar (back + center + right)
const appBar = (centerLabel, opts = {}) => {
  const { hasBack = true, hasRight = true, centerCaps = false } = opts;
  const out = [];
  if (hasBack) {
    out.push(rect(PAD, 64, 30, 30, { fill: 'none', stroke: STROKE_LIGHT, rx: 7 }));
    out.push(line(PAD + 16, 75, PAD + 10, 79, { stroke: TEXT_MUTE, sw: 1.4 }));
    out.push(line(PAD + 10, 79, PAD + 16, 83, { stroke: TEXT_MUTE, sw: 1.4 }));
  }
  if (hasRight) {
    out.push(rect(W - PAD - 30, 64, 30, 30, { fill: 'none', stroke: STROKE_LIGHT, rx: 7 }));
    out.push(circle(W - PAD - 21, 79, 1.4, { fill: TEXT_MUTE, stroke: 'none' }));
    out.push(circle(W - PAD - 15, 79, 1.4, { fill: TEXT_MUTE, stroke: 'none' }));
    out.push(circle(W - PAD - 9, 79, 1.4, { fill: TEXT_MUTE, stroke: 'none' }));
  }
  out.push(text(W / 2, 84, centerLabel, { size: 12, anchor: 'middle', fill: TEXT_DARK, family: centerCaps ? MONO : SANS, upper: centerCaps, tracking: centerCaps ? 1.2 : 0 }));
  return out.join('');
};

// display heading + sub
const displayHeading = (x, y, title, sub) => [
  text(x, y, title, { size: 18, weight: 600, fill: TEXT_DARK }),
  text(x, y + 18, sub, { size: 11, fill: TEXT_MUTE }),
].join('');

// generic input field
const inputField = (x, y, w, label, value, opts = {}) => {
  const { h = 56, valFamily = MONO, statusLabel } = opts;
  const out = [];
  out.push(rect(x, y, w, h, { fill: FILL_PLATE, stroke: STROKE_GHOST, rx: 10 }));
  out.push(text(x + 14, y + 18, label, { size: 9, fill: TEXT_MUTE, upper: true, tracking: 1.2, family: MONO }));
  out.push(text(x + 14, y + 38, value, { size: 11, family: valFamily, fill: TEXT_DARK }));
  if (statusLabel) {
    const sw = Math.round(statusLabel.length * 5.8 + 22);
    out.push(rect(x + w - sw - 10, y + 10, sw, 16, { fill: 'none', stroke: STROKE_LIGHT, rx: 4 }));
    out.push(circle(x + w - sw - 10 + 7, y + 18, 2, { fill: TEXT_MUTE, stroke: 'none' }));
    out.push(text(x + w - sw - 10 + 14, y + 22, statusLabel, { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: .8 }));
  }
  return out.join('');
};

// button
const button = (x, y, w, label, opts = {}) => {
  const { variant = 'primary', h = 38 } = opts;
  const fill = variant === 'primary' ? FILL_LABEL : 'none';
  const stroke = variant === 'ghost' ? 'none' : (variant === 'primary' ? STROKE_LIGHT : STROKE_LIGHT);
  return [
    rect(x, y, w, h, { fill, stroke, rx: 9 }),
    text(x + w / 2, y + h / 2 + 4, label, { size: 12, anchor: 'middle', weight: 500, fill: variant === 'primary' ? TEXT_DARK : TEXT_DARK }),
  ].join('');
};

// caps section title
const sectionTitle = (x, y, label, count) => {
  const out = [text(x, y, label, { size: 10, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.4 })];
  if (count != null) out.push(text(W - PAD, y, count, { size: 10, family: MONO, fill: TEXT_GHOST, anchor: 'end' }));
  return out.join('');
};

// tag pill
const tag = (x, y, label, opts = {}) => {
  const { hasDot = false } = opts;
  const padX = 8;
  const labelLen = label.length * 5.5 + (hasDot ? 8 : 0);
  const w = padX * 2 + labelLen;
  const out = [rect(x, y, w, 16, { fill: 'none', stroke: STROKE_LIGHT, rx: 3 })];
  let tx = x + padX;
  if (hasDot) {
    out.push(circle(x + padX + 2, y + 8, 2, { fill: TEXT_MUTE, stroke: 'none' }));
    tx += 8;
  }
  out.push(text(tx, y + 11, label, { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: .6 }));
  return { svg: out.join(''), w };
};

// generic card
const card = (x, y, w, h, opts = {}) => rect(x, y, w, h, { fill: FILL_PLATE, stroke: STROKE_GHOST, rx: 10, ...opts });

// annotation (gray label outside the screen — for spec callouts inside SVG)
const annotation = (x, y, label) =>
  text(x, y, label, { size: 9, family: MONO, fill: TEXT_GHOST, upper: true, tracking: 1 });

/* wraps a screen body in a full SVG with chrome */
const wrap = (id, name, ac, body) => `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${W} ${H}" width="${W}" height="${H}">
  <title>${id} · ${name}</title>
  <desc>FeatureDoc wireframe — ${id} ${name} — ${ac}</desc>
  ${frame()}
  ${chrome()}
  ${body}
</svg>
`;

/* ===========================================================
   SCREEN DEFINITIONS
   =========================================================== */
const screens = {};

/* ---------- S01 · Credentials Setup ---------- */
screens['s01-credentials-setup'] = wrap('S01', 'Credentials Setup', 'AC4.1 · AC4.2 · AC4.3', [
  // brand mark + step counter
  text(PAD, 80, '● FEATUREDOC', { size: 9, family: MONO, fill: TEXT_MUTE, tracking: 1.4 }),
  text(W - PAD, 80, 'STEP 1 / 1', { size: 9, family: MONO, fill: TEXT_MUTE, anchor: 'end', tracking: 1.4 }),

  // display heading
  displayHeading(PAD, 130, 'Bring your own keys', '모든 LLM 호출과 GitHub 접근은 당신의 자격증명으로 수행돼요.'),

  // GitHub App connection
  inputField(PAD, 200, W - 2 * PAD, 'GITHUB APP', 'FeatureDoc · 3 repositories', { statusLabel: 'INSTALLED' }),
  // app permission tags
  (() => { const { svg, w } = tag(PAD, 268, 'contents:read'); return svg + tag(PAD + w + 6, 268, 'metadata:read').svg; })(),

  // LLM Provider selector
  rect(PAD, 295, W - 2 * PAD, 70, { fill: FILL_PLATE, stroke: STROKE_GHOST, rx: 10 }),
  text(PAD + 14, 313, 'LLM PROVIDER', { size: 9, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.2 }),
  // 3 provider options
  rect(PAD + 12, 322, (W - 2 * PAD - 36) / 3, 30, { fill: FILL_LABEL, stroke: STROKE_GHOST, rx: 6 }),
  text(PAD + 12 + (W - 2 * PAD - 36) / 6, 342, 'Anthropic', { size: 10, anchor: 'middle', fill: TEXT_DARK, weight: 500 }),
  rect(PAD + 12 + (W - 2 * PAD - 36) / 3 + 6, 322, (W - 2 * PAD - 36) / 3, 30, { fill: 'none', stroke: STROKE_GHOST, rx: 6 }),
  text(PAD + 12 + (W - 2 * PAD - 36) / 3 + 6 + (W - 2 * PAD - 36) / 6, 342, 'OpenAI', { size: 10, anchor: 'middle', fill: TEXT_GHOST }),
  rect(PAD + 12 + 2 * (W - 2 * PAD - 36) / 3 + 12, 322, (W - 2 * PAD - 36) / 3, 30, { fill: 'none', stroke: STROKE_GHOST, rx: 6 }),
  text(PAD + 12 + 2 * (W - 2 * PAD - 36) / 3 + 12 + (W - 2 * PAD - 36) / 6, 342, 'Google', { size: 10, anchor: 'middle', fill: TEXT_GHOST }),

  // API Key input
  inputField(PAD, 380, W - 2 * PAD, 'API KEY', 'sk-ant-•••••••••••••••', {}),
  // eye icon
  circle(W - PAD - 22, 397, 6, { stroke: TEXT_MUTE, sw: 1.2 }),
  circle(W - PAD - 22, 397, 1.5, { fill: TEXT_MUTE, stroke: 'none' }),

  // helper card
  rect(PAD, 450, W - 2 * PAD, 56, { fill: FILL_PLATE, stroke: STROKE_GHOST, rx: 10 }),
  rect(PAD + 12, 462, 24, 24, { fill: FILL_LABEL, stroke: 'none', rx: 5 }),
  text(PAD + 46, 472, '키는 봉투 암호화로 저장되며 호출 직전에만', { size: 10, fill: TEXT_MUTE }),
  text(PAD + 46, 488, '메모리에서 복호화됩니다.', { size: 10, fill: TEXT_MUTE }),

  // CTA row
  rect(PAD, 740, 50, 38, { fill: 'none', stroke: STROKE_LIGHT, rx: 9 }),
  line(PAD + 30, 753, PAD + 22, 759, { stroke: TEXT_DARK, sw: 1.4 }),
  line(PAD + 22, 759, PAD + 30, 765, { stroke: TEXT_DARK, sw: 1.4 }),
  button(PAD + 60, 740, W - 2 * PAD - 60, 'Continue', { variant: 'primary' }),

  // annotation
  annotation(PAD, 800, '01 — onboarding · single step'),
].join(''));

/* ---------- S02 · Home / Repositories ---------- */
screens['s02-home-repositories'] = wrap('S02', 'Home · Repositories', 'AC1.1 · AC1.5', [
  // brand + title
  text(PAD, 80, '● FEATUREDOC', { size: 9, family: MONO, fill: TEXT_MUTE, tracking: 1.4 }),
  text(PAD, 102, 'Repositories', { size: 18, weight: 600, fill: TEXT_DARK }),
  // settings icon
  rect(W - PAD - 30, 75, 30, 30, { fill: 'none', stroke: STROKE_LIGHT, rx: 7 }),
  circle(W - PAD - 15, 90, 4, { fill: 'none', stroke: TEXT_MUTE, sw: 1.2 }),

  // metrics grid
  rect(PAD, 128, W - 2 * PAD, 60, { fill: 'none', stroke: STROKE_LIGHT, rx: 8 }),
  line(PAD + (W - 2 * PAD) / 3, 128, PAD + (W - 2 * PAD) / 3, 188, { stroke: STROKE_LIGHT }),
  line(PAD + 2 * (W - 2 * PAD) / 3, 128, PAD + 2 * (W - 2 * PAD) / 3, 188, { stroke: STROKE_LIGHT }),
  text(PAD + 14, 145, 'REPOS', { size: 9, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.2 }),
  text(PAD + 14, 174, '3', { size: 16, family: MONO, weight: 500, fill: TEXT_DARK }),
  text(PAD + (W - 2 * PAD) / 3 + 14, 145, 'FEATURES', { size: 9, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.2 }),
  text(PAD + (W - 2 * PAD) / 3 + 14, 174, '47', { size: 16, family: MONO, weight: 500, fill: TEXT_DARK }),
  text(PAD + 2 * (W - 2 * PAD) / 3 + 14, 145, 'SPEND', { size: 9, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.2 }),
  text(PAD + 2 * (W - 2 * PAD) / 3 + 14, 174, '$2.14', { size: 16, family: MONO, weight: 500, fill: TEXT_DARK }),

  // section title
  sectionTitle(PAD, 218, 'Repositories', '3'),

  // repo card 1
  card(PAD, 232, W - 2 * PAD, 84),
  text(PAD + 14, 254, 'payments-api', { size: 13, weight: 600, fill: TEXT_DARK }),
  text(PAD + 14, 270, '⎇ main · last analyzed 4m ago', { size: 9, family: MONO, fill: TEXT_MUTE }),
  // tag
  rect(W - PAD - 70, 246, 60, 16, { fill: 'none', stroke: STROKE_LIGHT, rx: 3 }),
  circle(W - PAD - 64, 254, 2, { fill: TEXT_MUTE, stroke: 'none' }),
  text(W - PAD - 56, 257, 'SYNCED', { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: .6 }),
  // stats
  text(PAD + 14, 300, '23 features  ·  4 conflicts  ·  $0.84', { size: 10, family: MONO, fill: TEXT_MUTE }),

  // repo card 2 (analyzing)
  card(PAD, 326, W - 2 * PAD, 96),
  text(PAD + 14, 348, 'checkout-web', { size: 13, weight: 600, fill: TEXT_DARK }),
  text(PAD + 14, 364, '⎇ release/v3.2 · running', { size: 9, family: MONO, fill: TEXT_MUTE }),
  rect(W - PAD - 80, 340, 70, 16, { fill: 'none', stroke: STROKE_LIGHT, rx: 3 }),
  circle(W - PAD - 74, 348, 2, { fill: TEXT_MUTE, stroke: 'none' }),
  text(W - PAD - 66, 351, 'ANALYZING', { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: .6 }),
  // progress
  rect(PAD + 14, 384, W - 2 * PAD - 60, 4, { fill: FILL_BLOCK, stroke: 'none', rx: 2 }),
  rect(PAD + 14, 384, (W - 2 * PAD - 60) * 0.62, 4, { fill: TEXT_MUTE, stroke: 'none', rx: 2 }),
  text(W - PAD - 30, 388, '62%', { size: 10, family: MONO, fill: TEXT_MUTE, anchor: 'end' }),
  text(PAD + 14, 408, 'step 3 of 5 · feature extraction', { size: 9, family: MONO, fill: TEXT_MUTE }),

  // repo card 3 (outdated)
  card(PAD, 432, W - 2 * PAD, 84),
  text(PAD + 14, 454, 'notif-worker', { size: 13, weight: 600, fill: TEXT_DARK }),
  text(PAD + 14, 470, '⎇ main · 2 days ago', { size: 9, family: MONO, fill: TEXT_MUTE }),
  rect(W - PAD - 76, 446, 66, 16, { fill: 'none', stroke: STROKE_LIGHT, rx: 3 }),
  circle(W - PAD - 70, 454, 2, { fill: TEXT_MUTE, stroke: 'none' }),
  text(W - PAD - 62, 457, 'OUTDATED', { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: .6 }),
  text(PAD + 14, 500, '11 features  ·  +4 commits since', { size: 10, family: MONO, fill: TEXT_MUTE }),

  // tab bar
  line(0, 720, W, 720, { stroke: STROKE_LIGHT }),
  // 4 tab items
  ...['Repos', 'Activity', 'Keys', 'Settings'].flatMap((label, i) => {
    const cx = (W / 4) * i + W / 8;
    return [
      rect(cx - 9, 738, 18, 14, { fill: 'none', stroke: i === 0 ? TEXT_DARK : TEXT_MUTE, sw: 1.2, rx: 2 }),
      text(cx, 770, label, { size: 8, family: MONO, fill: i === 0 ? TEXT_DARK : TEXT_MUTE, anchor: 'middle', upper: true, tracking: 1, weight: i === 0 ? 600 : 400 }),
    ];
  }),

  annotation(PAD, 800, '02 — discovery · home view'),
].join(''));

/* ---------- S03 · Connect Repository ---------- */
screens['s03-connect-repository'] = wrap('S03', 'Connect Repository', 'AC1.1 · AC4.6', [
  appBar('New Repository'),
  displayHeading(PAD, 130, 'Connect a repository', '분석을 시작하면 횡단 관심사 → 탐색 전략 → feature 후보 순으로 진행돼요.'),

  // URL input
  inputField(PAD, 200, W - 2 * PAD, 'REPOSITORY URL', 'github.com/dlddu/payments-api'),

  // branch + app access row
  inputField(PAD, 270, (W - 2 * PAD - 8) / 2, 'BRANCH', '⎇ main'),
  inputField(PAD + (W - 2 * PAD - 8) / 2 + 8, 270, (W - 2 * PAD - 8) / 2, 'GITHUB APP', '✓ has access'),

  // pre-flight estimate card
  rect(PAD, 350, W - 2 * PAD, 156, { fill: FILL_PLATE, stroke: STROKE_GHOST, rx: 10 }),
  text(PAD + 14, 374, 'PRE-FLIGHT ESTIMATE', { size: 9, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.2 }),
  // ready tag
  rect(W - PAD - 60, 363, 50, 16, { fill: 'none', stroke: STROKE_LIGHT, rx: 3 }),
  circle(W - PAD - 54, 371, 2, { fill: TEXT_MUTE, stroke: 'none' }),
  text(W - PAD - 46, 374, 'READY', { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: .6 }),
  // 2-column metric grid
  text(PAD + 14, 412, '~$0.80', { size: 16, family: MONO, weight: 500, fill: TEXT_DARK }),
  text(PAD + 14, 428, 'EST. LLM COST', { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.2 }),
  text(PAD + (W - 2 * PAD) / 2, 412, '~6 min', { size: 16, family: MONO, weight: 500, fill: TEXT_DARK }),
  text(PAD + (W - 2 * PAD) / 2, 428, 'EST. DURATION', { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.2 }),
  // separator
  line(PAD + 14, 448, W - PAD - 14, 448, { stroke: STROKE_GHOST }),
  // detail rows
  text(PAD + 14, 468, 'Files to scan', { size: 11, fill: TEXT_DARK }),
  text(W - PAD - 14, 468, '847 · 2.3 MB', { size: 11, family: MONO, fill: TEXT_DARK, anchor: 'end' }),
  text(PAD + 14, 488, 'Est. LLM calls', { size: 11, fill: TEXT_DARK }),
  text(W - PAD - 14, 488, '~120', { size: 11, family: MONO, fill: TEXT_DARK, anchor: 'end' }),

  // CTAs
  button(PAD, 540, W - 2 * PAD, 'Start Analysis →', { variant: 'primary' }),
  button(PAD, 590, W - 2 * PAD, 'Save for later', { variant: 'ghost' }),

  annotation(PAD, 800, '01 — onboarding · final step'),
].join(''));

/* ---------- S04 · Analysis in Progress ---------- */
screens['s04-analysis-progress'] = wrap('S04', 'Analysis in Progress', 'AC1.5 · AC4.6', [
  appBar('payments-api'),
  text(W / 2, 100, 'main · run #14', { size: 9, family: MONO, fill: TEXT_MUTE, anchor: 'middle' }),

  // progress ring
  circle(W / 2, 200, 60, { fill: 'none', stroke: STROKE_GHOST, sw: 2 }),
  // arc (62%) — using stroke-dasharray on a circle
  `<circle cx="${W / 2}" cy="200" r="60" fill="none" stroke="${TEXT_DARK}" stroke-width="2" stroke-dasharray="377" stroke-dashoffset="143" stroke-linecap="round" transform="rotate(-90 ${W / 2} 200)"/>`,
  text(W / 2, 207, '62', { size: 28, weight: 500, fill: TEXT_DARK, anchor: 'middle' }),
  text(W / 2, 224, 'PERCENT COMPLETE', { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.2, anchor: 'middle' }),

  sectionTitle(PAD, 295, 'Pipeline', '3 of 5'),

  // 5 pipeline steps
  ...[
    { state: 'done', label: 'Fetch repository', sub: '847 files · 2.3 MB', time: '12s' },
    { state: 'done', label: 'Cross-cutting concerns', sub: 'infra · arch · framework', time: '3m 24s' },
    { state: 'active', label: 'Discovery strategy', sub: 'analyzing entry points…', time: '1m 08s' },
    { state: 'todo', label: 'Extract feature candidates', sub: 'scan via approved strategy', time: '~2m' },
    { state: 'todo', label: 'Acceptance & dependencies', sub: 'per-feature representation', time: '~4m' },
  ].flatMap((step, i) => {
    const y = 314 + i * 52;
    const out = [card(PAD, y, W - 2 * PAD, 46)];
    // status circle
    if (step.state === 'done') {
      out.push(circle(PAD + 22, y + 23, 8, { fill: TEXT_DARK, stroke: 'none' }));
      out.push(line(PAD + 18, y + 23, PAD + 21, y + 26, { stroke: '#fff', sw: 1.4 }));
      out.push(line(PAD + 21, y + 26, PAD + 27, y + 20, { stroke: '#fff', sw: 1.4 }));
    } else if (step.state === 'active') {
      out.push(circle(PAD + 22, y + 23, 8, { fill: 'none', stroke: TEXT_DARK, sw: 1.5 }));
      out.push(circle(PAD + 22, y + 23, 11, { fill: 'none', stroke: STROKE_LIGHT, sw: 1, dash: '2 3' }));
    } else {
      out.push(circle(PAD + 22, y + 23, 8, { fill: 'none', stroke: STROKE_LIGHT, sw: 1.5 }));
    }
    out.push(text(PAD + 42, y + 19, step.label, { size: 12, fill: TEXT_DARK }));
    out.push(text(PAD + 42, y + 34, step.sub, { size: 9, family: MONO, fill: TEXT_MUTE }));
    out.push(text(W - PAD - 14, y + 27, step.time, { size: 9, family: MONO, fill: TEXT_MUTE, anchor: 'end' }));
    return out;
  }),

  // spend card
  card(PAD, 596, W - 2 * PAD, 56),
  text(PAD + 14, 614, 'LLM SPEND', { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.2 }),
  text(PAD + 14, 638, '$0.32', { size: 14, family: MONO, weight: 500, fill: TEXT_DARK }),
  text(PAD + 60, 638, 'of est. $0.80', { size: 9, family: MONO, fill: TEXT_MUTE }),
  text(W - PAD - 14, 614, 'CALLS', { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.2, anchor: 'end' }),
  text(W - PAD - 14, 638, '47', { size: 14, family: MONO, weight: 500, fill: TEXT_DARK, anchor: 'end' }),

  // CTAs
  button(PAD, 670, W - 2 * PAD - 50, 'Run in background', { variant: 'secondary' }),
  rect(W - PAD - 38, 670, 38, 38, { fill: 'none', stroke: STROKE_LIGHT, rx: 9 }),
  line(W - PAD - 28, 680, W - PAD - 14, 698, { stroke: TEXT_MUTE, sw: 1.4 }),
  line(W - PAD - 14, 680, W - PAD - 28, 698, { stroke: TEXT_MUTE, sw: 1.4 }),

  annotation(PAD, 800, '02 — discovery · async progress'),
].join(''));

/* ---------- S05 · Cross-cutting Concerns ---------- */
screens['s05-cross-cutting-concerns'] = wrap('S05', 'Cross-cutting Concerns', 'AC1.2 · V2', [
  appBar('payments-api · cross-cutting', { centerCaps: true }),
  displayHeading(PAD, 130, 'Cross-cutting concerns', 'LLM이 코드 전체에서 추출한 5개 횡단 관심사. 근거 파일과 함께 보존돼요.'),

  // 4 dependency category cards
  ...[
    { title: 'INFRASTRUCTURE', items: [['PostgreSQL 15', 'infra/db.tf'], ['Redis (cache + queue)', 'k8s/redis.yaml'], ['Kubernetes (EKS)', 'infra/eks.tf'], ['S3 (receipts)', 'app/storage/s3.py']] },
    { title: 'ARCHITECTURE', items: [['Hexagonal · Ports & Adapters', 'app/{domain,adapters}'], ['Async event bus', 'app/events/bus.py']] },
    { title: 'FRAMEWORK', items: [['FastAPI 0.110', 'pyproject.toml'], ['SQLAlchemy 2.0', 'pyproject.toml'], ['Pydantic v2', 'pyproject.toml']] },
    { title: 'MIDDLEWARE', items: [['JWT auth (RS256)', 'middleware/auth.py'], ['OpenTelemetry tracing', 'observability/'], ['Idempotency keys', 'middleware/idem.py']] },
  ].reduce((acc, cat) => {
    const y = acc.y;
    const cardH = 32 + cat.items.length * 22;
    acc.svg.push(card(PAD, y, W - 2 * PAD, cardH));
    acc.svg.push(circle(PAD + 18, y + 18, 3, { fill: TEXT_MUTE, stroke: 'none' }));
    acc.svg.push(text(PAD + 28, y + 21, cat.title, { size: 9, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.4 }));
    cat.items.forEach((item, i) => {
      const ry = y + 40 + i * 22;
      acc.svg.push(text(PAD + 14, ry, item[0], { size: 11, fill: TEXT_DARK }));
      acc.svg.push(text(W - PAD - 14, ry, item[1], { size: 9, family: MONO, fill: TEXT_MUTE, anchor: 'end' }));
    });
    acc.y = y + cardH + 8;
    return acc;
  }, { svg: [], y: 200 }).svg,

  annotation(PAD, 800, '02 — discovery · concerns extracted'),
].join(''));

/* ---------- S06 · Discovery Strategy ---------- */
screens['s06-discovery-strategy'] = wrap('S06', 'Discovery Strategy', 'AC1.3 · V1·V2', [
  appBar('payments-api · strategy', { centerCaps: true }),
  // tags
  (() => {
    const t1 = tag(PAD, 124, 'DRAFT v2', { hasDot: true });
    const t2 = tag(PAD + t1.w + 6, 124, 'REGENERATED 1m AGO');
    return t1.svg + t2.svg;
  })(),
  displayHeading(PAD, 162, 'Discovery strategy', 'LLM이 횡단 관심사를 토대로 만든 탐색 전략. 검토 후 승인하면 다음 단계의 입력이 돼요.'),

  // YAML code block
  rect(PAD, 232, W - 2 * PAD, 232, { fill: '#fafafa', stroke: STROKE_GHOST, rx: 8 }),
  ...[
    '# where to look for user-facing features',
    'strategy:',
    '  entry_points:',
    '    - "app/api/routes/**/*.py"',
    '    - "app/cli/commands/*.py"',
    '    - "app/workers/handlers/*.py"',
    '  routing:',
    '    framework: fastapi',
    '    decorators: ["@router.*", "@app.*"]',
    '  exclude:',
    '    - "app/api/internal/**"  # ops only',
    '    - "app/api/health.py"',
    '  naming: "verb_noun from path"',
  ].flatMap((ln, i) => {
    const y = 252 + i * 16;
    return [
      text(PAD + 12, y, String(i + 1), { size: 8.5, family: MONO, fill: TEXT_GHOST, anchor: 'end' }),
      text(PAD + 24, y, ln, { size: 9, family: MONO, fill: ln.includes('#') ? TEXT_GHOST : TEXT_DARK }),
    ];
  }),

  // heads-up callout
  rect(PAD, 484, W - 2 * PAD, 56, { fill: FILL_PLATE, stroke: STROKE_GHOST, rx: 10 }),
  rect(PAD + 12, 496, 22, 22, { fill: FILL_LABEL, stroke: 'none', rx: 5 }),
  text(PAD + 44, 506, 'Heads up · app/api/internal/는 운영용으로', { size: 10, fill: TEXT_DARK }),
  text(PAD + 44, 522, '제외했어요. 포함하려면 편집하세요.', { size: 10, fill: TEXT_MUTE }),

  // CTAs
  button(PAD, 568, (W - 2 * PAD - 8) * 0.42, 'Regenerate', { variant: 'secondary' }),
  button(PAD + (W - 2 * PAD - 8) * 0.42 + 8, 568, (W - 2 * PAD - 8) * 0.58, 'Approve & Scan →', { variant: 'primary' }),

  annotation(PAD, 800, '02 — discovery · user reviews strategy'),
].join(''));

/* ---------- S07 · Feature Candidates ---------- */
screens['s07-feature-candidates'] = wrap('S07', 'Feature Candidates', 'AC1.4 · V1', [
  appBar('payments-api · candidates', { centerCaps: true }),
  displayHeading(PAD, 130, 'Feature candidates', '탐색 전략을 통해 발견된 후보 23개. 승인하거나 거부하세요.'),

  // status counts
  text(PAD, 178, '9 approved   ·   12 pending   ·   2 rejected', { size: 10, family: MONO, fill: TEXT_MUTE }),

  // filter chips
  (() => {
    const t1 = tag(PAD, 198, 'All · 23');
    const t2 = tag(PAD + t1.w + 6, 198, 'Pending · 12');
    const t3 = tag(PAD + t1.w + t2.w + 12, 198, 'Approved · 9');
    return t1.svg + t2.svg + t3.svg;
  })(),

  // 4 candidate cards
  ...[
    { name: 'Process card payment', loc: 'POST /v1/payments · routes/payments.py:42', why: '결제 토큰을 받아 PSP에 청구를 보내고 결과를 응답해요.', tags: ['HTTP', 'idempotent'] },
    { name: 'Refund a payment', loc: 'POST /v1/payments/{id}/refund · routes/refunds.py:18', why: '기존 결제에 대해 부분/전체 환불을 처리해요.', tags: ['HTTP'] },
    { name: 'Schedule recurring billing', loc: 'workers/handlers/billing.py:7 · scheduler/cron.yaml', why: '매월 활성 구독자에 대해 청구를 자동 생성. 2개 후보가 같은 기능 — 병합 권장.', tags: ['WORKER', 'MERGE 2'] },
    { name: 'Issue receipt PDF', loc: 'workers/handlers/receipt.py:11', why: '결제 성공 시 PDF 영수증을 생성해 S3에 저장하고 이메일로 발송.', tags: [] },
  ].flatMap((cand, i) => {
    const y = 232 + i * 110;
    const out = [card(PAD, y, W - 2 * PAD, 100)];
    out.push(text(PAD + 14, y + 22, cand.name, { size: 12, weight: 600, fill: TEXT_DARK }));
    out.push(text(PAD + 14, y + 38, cand.loc, { size: 9, family: MONO, fill: TEXT_MUTE }));
    // approve / reject icons
    out.push(rect(W - PAD - 70, y + 12, 28, 24, { fill: 'none', stroke: STROKE_LIGHT, rx: 5 }));
    out.push(line(W - PAD - 64, y + 24, W - PAD - 60, y + 28, { stroke: TEXT_MUTE, sw: 1.4 }));
    out.push(line(W - PAD - 60, y + 28, W - PAD - 50, y + 18, { stroke: TEXT_MUTE, sw: 1.4 }));
    out.push(rect(W - PAD - 36, y + 12, 28, 24, { fill: 'none', stroke: STROKE_LIGHT, rx: 5 }));
    out.push(line(W - PAD - 30, y + 18, W - PAD - 14, y + 30, { stroke: TEXT_MUTE, sw: 1.4 }));
    out.push(line(W - PAD - 14, y + 18, W - PAD - 30, y + 30, { stroke: TEXT_MUTE, sw: 1.4 }));
    // why text (wrapped)
    out.push(text(PAD + 14, y + 60, cand.why.slice(0, 40), { size: 10, fill: TEXT_DARK }));
    if (cand.why.length > 40) out.push(text(PAD + 14, y + 74, cand.why.slice(40, 80), { size: 10, fill: TEXT_DARK }));
    // tags
    let tx = PAD + 14;
    cand.tags.forEach(t => {
      const tg = tag(tx, y + 80, t);
      out.push(tg.svg);
      tx += tg.w + 4;
    });
    return out;
  }),

  text(W / 2, 695, '8 more candidates ↓', { size: 9, family: MONO, fill: TEXT_MUTE, anchor: 'middle' }),

  annotation(PAD, 800, '02 — discovery · approve / reject candidates'),
].join(''));

/* ---------- S08 · Feature · Acceptance ---------- */
screens['s08-feature-acceptance'] = wrap('S08', 'Feature · Acceptance', 'AC2.1 · AC2.2 · AC2.3', [
  appBar('payments-api'),
  // feature tag
  (() => tag(PAD, 124, 'FEATURE / F-014').svg)(),
  displayHeading(PAD, 156, 'Process card payment', '고객이 카드 정보로 결제 토큰을 보내면 PSP에 청구하고 결과를 응답해요.'),

  // tabs
  rect(PAD, 210, W - 2 * PAD, 36, { fill: 'none', stroke: STROKE_LIGHT, rx: 8 }),
  rect(PAD + 4, 214, (W - 2 * PAD - 8) / 3 - 2, 28, { fill: FILL_LABEL, stroke: STROKE_LIGHT, rx: 5 }),
  text(PAD + (W - 2 * PAD) / 6, 232, 'Acceptance · 4', { size: 10, anchor: 'middle', weight: 500, fill: TEXT_DARK }),
  text(PAD + (W - 2 * PAD) / 2, 232, 'Dependencies', { size: 10, anchor: 'middle', fill: TEXT_MUTE }),
  text(PAD + 5 * (W - 2 * PAD) / 6, 232, 'History', { size: 10, anchor: 'middle', fill: TEXT_MUTE }),

  // 4 GWT scenarios (compact)
  ...[
    { id: 'SCN-1', title: '유효한 카드로 정상 결제', src: 'routes/payments.py:42' },
    { id: 'SCN-2', title: '같은 멱등성 키로 재시도', src: 'middleware/idem.py:23' },
    { id: 'SCN-3', title: 'PSP 5xx 에러', conflict: true, src: '코드는 5회, 테스트는 3회 — 충돌' },
    { id: 'SCN-4', title: '잘못된 카드 토큰', src: 'routes/payments.py:71' },
  ].flatMap((s, i) => {
    const y = 268 + i * 102;
    const out = [card(PAD, y, W - 2 * PAD, 92)];
    out.push(text(PAD + 14, y + 18, s.id, { size: 8, family: MONO, fill: TEXT_GHOST, upper: true, tracking: .8 }));
    out.push(text(PAD + 50, y + 18, s.title, { size: 11, fill: TEXT_DARK, weight: 500 }));
    if (s.conflict) {
      out.push(rect(W - PAD - 60, y + 8, 50, 16, { fill: 'none', stroke: STROKE_LIGHT, rx: 3 }));
      out.push(circle(W - PAD - 54, y + 16, 2, { fill: TEXT_MUTE, stroke: 'none' }));
      out.push(text(W - PAD - 46, y + 19, 'CONFLICT', { size: 7, family: MONO, fill: TEXT_MUTE, upper: true, tracking: .6 }));
    }
    // GWT lines
    ['GIVEN', 'WHEN', 'THEN'].forEach((lbl, j) => {
      const ly = y + 38 + j * 14;
      out.push(text(PAD + 14, ly, lbl, { size: 7, family: MONO, fill: TEXT_GHOST, upper: true, tracking: 1.2 }));
      out.push(placeholderLine(PAD + 60, ly - 4, W - 2 * PAD - 80));
    });
    out.push(text(PAD + 14, y + 84, '🔗 ' + s.src, { size: 9, family: MONO, fill: TEXT_MUTE }));
    return out;
  }),

  // edit cta
  button(PAD, 690, W - 2 * PAD, 'Edit with LLM', { variant: 'secondary' }),

  annotation(PAD, 800, '03 — feature documents · acceptance tests'),
].join(''));

/* ---------- S09 · Feature · Dependencies ---------- */
screens['s09-feature-dependencies'] = wrap('S09', 'Feature · Dependencies', 'AC2.4 · AC2.5 · V5', [
  appBar('payments-api'),
  (() => tag(PAD, 124, 'FEATURE / F-014').svg)(),
  text(PAD, 162, 'Process card payment', { size: 18, weight: 600, fill: TEXT_DARK }),

  // tabs
  rect(PAD, 192, W - 2 * PAD, 36, { fill: 'none', stroke: STROKE_LIGHT, rx: 8 }),
  text(PAD + (W - 2 * PAD) / 6, 214, 'Acceptance · 4', { size: 10, anchor: 'middle', fill: TEXT_MUTE }),
  rect(PAD + (W - 2 * PAD) / 3 + 4, 196, (W - 2 * PAD - 8) / 3 - 2, 28, { fill: FILL_LABEL, stroke: STROKE_LIGHT, rx: 5 }),
  text(PAD + (W - 2 * PAD) / 2, 214, 'Dependencies · 11', { size: 10, anchor: 'middle', weight: 500, fill: TEXT_DARK }),
  text(PAD + 5 * (W - 2 * PAD) / 6, 214, 'History', { size: 10, anchor: 'middle', fill: TEXT_MUTE }),

  // dependency graph
  rect(PAD, 246, W - 2 * PAD, 140, { fill: FILL_PLATE, stroke: STROKE_GHOST, rx: 10 }),
  // central node
  circle(W / 2, 316, 22, { fill: '#fff', stroke: TEXT_DARK, sw: 1.5 }),
  text(W / 2, 314, 'F-014', { size: 9, family: MONO, anchor: 'middle', fill: TEXT_DARK, weight: 500 }),
  text(W / 2, 326, 'payment', { size: 7, family: MONO, anchor: 'middle', fill: TEXT_MUTE }),
  // 6 spokes + nodes
  ...[
    [PAD + 24, 270, 'postgres'],
    [PAD + 24, 316, 'redis'],
    [PAD + 24, 360, 'PSP api'],
    [W - PAD - 24, 270, 'JWT auth'],
    [W - PAD - 24, 316, 'idem mw'],
    [W - PAD - 24, 360, 'payment.py'],
  ].flatMap(([x, y, label]) => {
    const dx = x < W / 2 ? W / 2 - 22 : W / 2 + 22;
    return [
      line(x, y, dx, 316, { stroke: STROKE_LIGHT }),
      circle(x, y, 3, { fill: TEXT_MUTE, stroke: 'none' }),
      text(x, y < 316 ? y - 8 : y + 14, label, { size: 8, family: MONO, anchor: 'middle', fill: TEXT_MUTE }),
    ];
  }),

  // 4 dep category cards (compact)
  ...[
    { title: 'INFRASTRUCTURE · 3', items: [['PostgreSQL', 'payments tbl'], ['Redis', 'idempotency'], ['Stripe (PSP)', 'api.stripe.com']] },
    { title: 'DATA MODEL · 2', items: [['Payment', 'db/models/payment.py'], ['IdempotencyKey', 'db/models/idem.py']] },
    { title: 'MIDDLEWARE · 2', items: [['JWT auth', 'middleware/auth.py'], ['Idempotency', 'middleware/idem.py']] },
    { title: 'INTERFACES · 1', items: [['POST /v1/payments', 'routes/payments.py']] },
  ].reduce((acc, cat) => {
    const y = acc.y;
    const cardH = 32 + cat.items.length * 18;
    acc.svg.push(card(PAD, y, W - 2 * PAD, cardH));
    acc.svg.push(circle(PAD + 18, y + 18, 3, { fill: TEXT_MUTE, stroke: 'none' }));
    acc.svg.push(text(PAD + 28, y + 21, cat.title, { size: 9, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.4 }));
    cat.items.forEach((item, i) => {
      const ry = y + 38 + i * 18;
      acc.svg.push(text(PAD + 14, ry, item[0], { size: 10, fill: TEXT_DARK }));
      acc.svg.push(text(W - PAD - 14, ry, item[1], { size: 9, family: MONO, fill: TEXT_MUTE, anchor: 'end' }));
    });
    acc.y = y + cardH + 6;
    return acc;
  }, { svg: [], y: 400 }).svg,

  annotation(PAD, 800, '03 — feature documents · dependency view'),
].join(''));

/* ---------- S10 · LLM-assisted Edit ---------- */
screens['s10-llm-edit'] = wrap('S10', 'LLM-assisted Edit', 'AC3.1 · V3·V4·V7', [
  appBar('Edit · F-014'),
  text(W / 2, 100, 'Process card payment', { size: 9, family: MONO, fill: TEXT_MUTE, anchor: 'middle' }),

  // user prompt card
  card(PAD, 130, W - 2 * PAD, 80),
  rect(PAD + 12, 142, 24, 24, { fill: FILL_LABEL, stroke: 'none', rx: 5 }),
  text(PAD + 46, 152, 'YOU ASKED', { size: 8, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1.2 }),
  text(PAD + 46, 174, '에러 케이스에 네트워크 타임아웃', { size: 11, fill: TEXT_DARK }),
  text(PAD + 46, 192, '시나리오 추가해줘', { size: 11, fill: TEXT_DARK }),

  sectionTitle(PAD, 240, 'Proposed change · diff', '+1 scenario'),

  // diff card
  rect(PAD, 256, W - 2 * PAD, 200, { fill: FILL_PLATE, stroke: STROKE_GHOST, rx: 10 }),
  // diff header
  rect(PAD, 256, W - 2 * PAD, 28, { fill: FILL_BLOCK, stroke: 'none', rx: 10 }),
  text(PAD + 14, 274, 'SCN-5 · NEW', { size: 9, family: MONO, fill: TEXT_MUTE, upper: true, tracking: 1 }),
  rect(W - PAD - 80, 264, 70, 14, { fill: 'none', stroke: STROKE_LIGHT, rx: 3 }),
  circle(W - PAD - 74, 271, 2, { fill: TEXT_MUTE, stroke: 'none' }),
  text(W - PAD - 66, 274, '+12 LINES', { size: 7, family: MONO, fill: TEXT_MUTE, upper: true, tracking: .6 }),
  // diff lines
  ...[
    'scenario: PSP timeout',
    '  given:',
    '    PSP가 10초 내 응답하지',
    '    않는 상황에서',
    '  when:',
    '    결제 요청이 들어오면',
    '  then:',
    '    멱등성 키를 보존한 채',
    '    504 Gateway Timeout 반환',
  ].flatMap((ln, i) => {
    const y = 296 + i * 14;
    return [
      text(PAD + 14, y, '+', { size: 9, family: MONO, fill: TEXT_GHOST }),
      text(PAD + 28, y, ln, { size: 9, family: MONO, fill: TEXT_DARK }),
    ];
  }),

  // evidence card
  card(PAD, 472, W - 2 * PAD, 56),
  rect(PAD + 12, 484, 22, 22, { fill: FILL_LABEL, stroke: 'none', rx: 5 }),
  text(PAD + 44, 494, '근거 · PSP 클라이언트의 timeout 설정과', { size: 10, fill: TEXT_DARK }),
  text(PAD + 44, 510, '통합 테스트의 504 케이스에서 추출.', { size: 10, fill: TEXT_MUTE }),

  // CTA row: discard / retry / apply
  rect(PAD, 552, 50, 38, { fill: 'none', stroke: STROKE_LIGHT, rx: 9 }),
  line(PAD + 18, 565, PAD + 32, 580, { stroke: TEXT_MUTE, sw: 1.4 }),
  line(PAD + 32, 565, PAD + 18, 580, { stroke: TEXT_MUTE, sw: 1.4 }),
  button(PAD + 60, 552, 110, 'Retry', { variant: 'secondary' }),
  button(PAD + 178, 552, W - PAD - PAD - 178, 'Apply', { variant: 'primary' }),

  // composer
  card(PAD, 616, W - 2 * PAD, 86),
  text(PAD + 14, 638, '한 줄로 변경을 지시하세요…', { size: 11, fill: TEXT_GHOST }),
  // chips
  (() => {
    const t1 = tag(PAD + 14, 670, '+ scenario');
    const t2 = tag(PAD + 14 + t1.w + 6, 670, 'clarify');
    return t1.svg + t2.svg;
  })(),
  // send button
  rect(W - PAD - 38, 660, 28, 28, { fill: FILL_LABEL, stroke: 'none', rx: 6 }),
  line(W - PAD - 24, 668, W - PAD - 24, 680, { stroke: TEXT_DARK, sw: 1.6 }),
  line(W - PAD - 28, 672, W - PAD - 24, 668, { stroke: TEXT_DARK, sw: 1.6 }),
  line(W - PAD - 24, 668, W - PAD - 20, 672, { stroke: TEXT_DARK, sw: 1.6 }),

  annotation(PAD, 800, '03 — feature documents · llm-assisted edit'),
].join(''));

/* ===========================================================
   WRITE FILES
   =========================================================== */
const outDir = '../docs/wireframes';
fs.mkdirSync(outDir, { recursive: true });

let written = 0;
for (const [name, svg] of Object.entries(screens)) {
  fs.writeFileSync(path.join(outDir, name + '.svg'), svg);
  written++;
  console.log('  ✓', name + '.svg', `(${svg.length} chars)`);
}
console.log(`\nWrote ${written} wireframes to ${outDir}`);
