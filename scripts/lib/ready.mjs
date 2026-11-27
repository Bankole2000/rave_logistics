// Definition of ready (docs/GITHUB.md) as a check, so AI Ready is computed, not guessed.
// Pure functions: the caller passes the issue body and the spec documents' text.
import { createHash } from 'node:crypto';

/** Pseudo-journey for work that belongs to no journey (setup, chores, cross-cutting work). */
export const PLATFORM = 'Platform';
// Apps whose tasks may be Platform work without feature, requirement and test IDs.
const TOOLING_APPS = new Set(['repo', 'infra']);

export const TASK_SECTIONS = [
  'Task key, feature, requirement and test IDs',
  'App',
  'Objective and context',
  'Requirements',
  'Acceptance criteria',
  'Out of scope and constraints',
  'Dependencies',
  'Testing',
  'Definition of done',
];
// Sections that may say "None"; the others need real content.
const MAY_BE_NONE = new Set(['Out of scope and constraints', 'Dependencies']);

/** Splits a markdown body into [{ heading, text }] for its `## ` sections, in order. */
export function sections(body) {
  const out = [];
  let cur = null;
  for (const line of (body ?? '').replace(/\r/g, '').split('\n')) {
    const m = line.match(/^##\s+(.+?)\s*$/);
    if (m) { cur = { heading: m[1], text: '' }; out.push(cur); continue; }
    if (cur) cur.text += `${line}\n`;
  }
  return out;
}

const strip = (t) => t.replace(/<!--[\s\S]*?-->/g, '').trim();

/** Rows of the first markdown table after `## <heading>`, keyed by the header row. */
export function tableRows(doc, heading) {
  const lines = doc.replace(/\r/g, '').split('\n');
  const start = lines.findIndex((l) => l.trim() === `## ${heading}`);
  if (start < 0) return [];
  let header = null;
  const rows = [];
  for (let i = start + 1; i < lines.length && !lines[i].startsWith('## '); i++) {
    const l = lines[i].trim();
    if (!l.startsWith('|')) { if (header && rows.length) break; continue; }
    const cells = l.slice(1, l.endsWith('|') ? -1 : undefined).split('|').map((c) => c.trim());
    if (!header) { header = cells; continue; }
    if (cells.every((c) => /^:?-+:?$/.test(c))) continue;
    rows.push(Object.fromEntries(header.map((h, j) => [h, cells[j] ?? ''])));
  }
  return rows;
}

/**
 * What the spec documents define: features (status and journey), requirements, tests,
 * and journeys (name, milestone key such as `M1`, features).
 */
export function specIndex({ features = '', prd = '', testing = '' }) {
  const rows = tableRows(features, 'Features');
  const ft = new Map(rows.map((r) => [r.ID, (r.Status || '').toLowerCase()]));
  const ftJourney = new Map(rows.map((r) => [r.ID, (r.Journey || '').match(/J-\d+/)?.[0] ?? '']));
  const journeys = new Map(tableRows(features, 'Journeys').map((r) => [r.ID, {
    name: r.Name ?? '',
    milestone: (r.Milestone || '').match(/M\d+/)?.[0] ?? '',
    features: (r['Features, in order'] || '').match(/FT-\d+/g) ?? [],
  }]));
  const fr = new Set(tableRows(prd, 'MVP scope and functional requirements').map((r) => r.ID));
  const tc = new Set(tableRows(testing, 'Acceptance tests').map((r) => r.ID));
  return { ft, ftJourney, journeys, fr, tc };
}

/** `M1` from a milestone title such as "M1 - Vendors and drivers can join". */
export const milestoneKey = (title) => (title ?? '').match(/^M\d+/)?.[0] ?? '';

const idsOf = (body) => strip(sections(body).find((s) => s.heading === TASK_SECTIONS[0])?.text ?? '');

/** Journeys of the features an issue names, sorted, without repeats. */
function featureJourneys(body, index) {
  const fts = [...new Set(idsOf(body).match(/FT-\d{3,}/g) ?? [])];
  return [...new Set(fts.map((f) => index.ftJourney?.get(f)).filter(Boolean))].sort();
}

/** The journey an issue belongs to: that of its features, or Platform when it names none. */
export function journeyOf(body, index) {
  return featureJourneys(body, index)[0] ?? PLATFORM;
}

/**
 * Fingerprint of what a human approved: the body without comments, blank lines, indentation
 * or ticked checkboxes, so ticking criteria keeps the approval but changing the scope doesn't.
 */
export function approvalHash(body) {
  const norm = strip((body ?? '').replace(/\r/g, ''))
    .split('\n')
    .map((l) => l.trim().replace(/^([-*]\s*)\[[xX]\]/, '$1[ ]'))
    .filter(Boolean)
    .join('\n');
  return createHash('sha256').update(norm).digest('hex').slice(0, 16);
}

/**
 * Checks an issue body against the task template and the spec documents.
 * Pass `journey` (the Journey field) and `milestone` (its title) to check them too.
 * @returns {string[]} problems; empty means ready
 */
export function lintTask(body, index, { apps = [], journey, milestone } = {}) {
  const problems = [];
  const found = sections(body);
  const byName = new Map(found.map((s) => [s.heading, strip(s.text)]));
  let last = -1;
  for (const name of TASK_SECTIONS) {
    const pos = found.findIndex((s) => s.heading === name);
    if (pos < 0) { problems.push(`missing section "## ${name}"`); continue; }
    if (pos < last) problems.push(`section "## ${name}" is out of order`);
    last = pos;
    if (name === 'Definition of done') continue;
    const text = byName.get(name) ?? '';
    // Bare "-" or "- [ ]" lines left over from the template don't count as content.
    const empty = !text.replace(/^[-*]\s*(\[[ xX]\])?\s*$/gm, '').trim();
    if (empty) problems.push(`section "## ${name}" is empty`);
    else if (/^([-*]\s*)?none\.?$/i.test(text) &&!MAY_BE_NONE.has(name)) problems.push(`section "## ${name}" can't be "None"`);
  }

  const ids = byName.get(TASK_SECTIONS[0]) ?? '';
  if (!/TASK-\d{3,}/.test(ids)) problems.push('no task key (TASK-###)');
  const want = (re) => [...new Set(ids.match(re) ?? [])];
  const fts = want(/FT-\d{3,}/g), frs = want(/FR-\d{3,}/g), tcs = want(/TC-\d{3,}/g);
  const app = (byName.get('App') ?? '').replace(/[`*]/g, '').trim().split(/[\s,(]/)[0];
  // Repository and infrastructure work may be Platform work, without product IDs.
  const platformWork = !fts.length && TOOLING_APPS.has(app) && /\bPlatform\b/i.test(ids);
  if (!platformWork) {
    if (!fts.length) problems.push('no feature ID (FT-###); repo or infra work without one says "Platform"');
    if (!frs.length) problems.push('no requirement ID (FR-###)');
    if (!tcs.length) problems.push('no acceptance test ID (TC-###)');
  }
  for (const id of fts) {
    if (!index.ft.has(id)) problems.push(`${id} is not in docs/FEATURES.md`);
    else if (index.ft.get(id) !== 'approved') problems.push(`${id} is ${index.ft.get(id) || 'not approved'}, not approved`);
  }
  for (const id of frs) if (!index.fr.has(id)) problems.push(`${id} is not in docs/PRD.md`);
  for (const id of tcs) if (!index.tc.has(id)) problems.push(`${id} is not in docs/TESTING.md`);

  const criteria = byName.get('Acceptance criteria') ?? '';
  if (!/^[-*]\s*\[[ xX]\]\s*\S/m.test(criteria)) problems.push('acceptance criteria need at least one "- [ ] …" item');

  if (apps.length && app && !apps.includes(app)) problems.push(`app "${app}" is not one of: ${apps.join(', ')}`);

  // Journey: the features share one journey (docs/FEATURES.md), which matches the issue's
  // Journey field and milestone when they are given.
  const fj = featureJourneys(body, index);
  if (fj.length > 1) problems.push(`features belong to several journeys (${fj.join(', ')}); split the task`);
  const expected = fj[0] ?? PLATFORM;
  if (journey !== undefined) {
    if (!journey) problems.push(`no Journey set (expected ${expected})`);
    else if (journey !== expected) problems.push(`Journey is ${journey}, but the task belongs to ${expected}`);
  }
  if (milestone !== undefined) {
    const mk = milestoneKey(milestone);
    const jm = index.journeys?.get(expected)?.milestone;
    if (!mk) problems.push('no milestone');
    else if (expected !== PLATFORM && jm && jm !== mk) problems.push(`milestone is ${mk}, but ${expected} is delivered in ${jm}`);
  }
  return problems;
}

/** Branch name for an issue: `<number>-<slug of the title without "[Task] TASK-###:">`. */
export function branchName(number, title) {
  const slug = title
    .replace(/^\[[^\]]*\]\s*/, '')
    .replace(/^TASK-\d+:\s*/i, '')
    .toLowerCase()
    .normalize('NFKD').replace(/[̀-ͯ]/g, '')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 40)
    .replace(/-+$/, '');
  return `${number}-${slug || 'task'}`;
}

/** Orders candidate tasks: priority, then milestone (M0 first), then oldest issue. */
export function compareTasks(a, b) {
  const rank = { Critical: 0, High: 1, Medium: 2, Low: 3 };
  const pa = rank[a.priority] ?? 2, pb = rank[b.priority] ?? 2;
  if (pa !== pb) return pa - pb;
  const ma = Number(milestoneKey(a.milestone).slice(1) || 99);
  const mb = Number(milestoneKey(b.milestone).slice(1) || 99);
  if (ma !== mb) return ma - mb;
  return a.number - b.number;
}
