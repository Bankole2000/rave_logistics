#!/usr/bin/env node
// Task workflow for agents and the owner (docs/GITHUB.md, "Agent workflow"). Uses the GitHub
// CLI (gh) with the signed-in login; needs the project scope (gh auth refresh -s project).
//
//   next                          The next task to work on: approved, ready, unclaimed, unblocked
//   claim <issue>                 Claim it: create its branch (the lock), assign, In Progress
//   pr --title <t> --body-file <f> [--draft]
//                                 Push the branch and open its PR; ready PRs go to In Review
//   block <issue> --reason <text> Status Blocked, needs:human-input, with the reason
//   ask <issue> --question <text> Ask the owner a question on the issue (needs:human-input)
//   ready [<issue>… | --all]      Re-run the definition-of-ready check; set AI Ready, and undo
//                                 approvals whose issue changed since it was approved
//   plan --file <plan.json> [--dry-run]
//                                 Create or update issues from a plan (idempotent by task key)
//   review <pr> --body-file <f>   Post an agent review ("Verdict: …") and re-run the risk gate
//   risk [--base origin/main]     Risk of the current branch, as the risk gate will judge it
//   digest                        What needs the owner: approvals, questions, PRs to merge
//
// Human only (refused inside Claude Code, and blocked by .claude/hooks/guard.mjs):
//   approve-journey <J-xx|Platform> [--milestone M1] [--dry-run]
//   approve-milestone <M1> [--dry-run]
import { execFileSync } from 'node:child_process';
import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { hostname } from 'node:os';
import {
  PLATFORM, specIndex, lintTask, journeyOf, approvalHash, branchName, compareTasks, milestoneKey,
} from './lib/ready.mjs';
import { classify, breakingOpenApiChanges, matchesAny } from './lib/risk.mjs';
import { claimBranch, releaseBranch, git } from './lib/claim.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const read = (p) => (existsSync(join(ROOT, p)) ? readFileSync(join(ROOT, p), 'utf8') : '');
const RULES = JSON.parse(read('.github/risk-rules.json') || '{}');
const APPROVAL_MARK = /<!-- agent:approval journey=(\S+) hash=([0-9a-f]+) -->/;
const VERDICT_FIRST_LINE = /^\s*Verdict:\s*(Approve|Request changes|Comment only)\s*$/i;

// ---------- gh ----------
function gh(args, { input, json = false, allowFail = false } = {}) {
  try {
    const out = execFileSync('gh', args, { cwd: ROOT, encoding: 'utf8', input, stdio: ['pipe', 'pipe', 'pipe'], maxBuffer: 64 << 20 });
    return json ? JSON.parse(out || 'null') : out.trim();
  } catch (e) {
    if (allowFail) return null;
    throw new Error(`gh ${args.slice(0, 3).join(' ')} failed: ${String(e.stderr || e.message).trim()}`);
  }
}
const graphql = (query, vars = {}) => {
  const args = ['api', 'graphql', '-f', `query=${query}`];
  for (const [k, v] of Object.entries(vars)) args.push(typeof v === 'number' ? '-F' : '-f', `${k}=${v}`);
  return gh(args, { json: true }).data;
};

let repoCache;
function repo() {
  if (!repoCache) {
    const nwo = gh(['repo', 'view', '--json', 'nameWithOwner', '--jq', '.nameWithOwner']);
    const [owner, name] = nwo.split('/');
    repoCache = { owner, name, nwo };
  }
  return repoCache;
}

/** Project from docs/GITHUB.md ("- Project: <owner> #<number>"), or none (label fallback). */
function projectConfig() {
  const m = read('docs/GITHUB.md').match(/^- Project:\s*@?([\w.-]+)\s+#(\d+)/m);
  return m ? { owner: m[1], number: Number(m[2]) } : null;
}

// ---------- fields: Project #N, or labels without a Project (docs/GITHUB.md) ----------
const FIELD_LABELS = {
  Status: { Todo: 'status:todo', 'In Progress': 'status:in-progress', 'In Review': 'status:in-review', Blocked: 'status:blocked' },
  'Human Review': { Approved: 'human:approved', 'Changes Requested': 'human:changes-requested' },
  'AI Ready': { Yes: 'ai:ready' },
};

class Board {
  constructor() {
    this.project = projectConfig();
    this.items = new Map(); // issue number -> { itemId, fields: {name: value} }
    if (!this.project) return;
    const { owner, number } = this.project;
    const meta = gh(['project', 'field-list', String(number), '--owner', owner, '--format', 'json', '--limit', '50'], { json: true });
    this.fields = new Map(meta.fields.map((f) => [f.name, f]));
    this.projectId = gh(['project', 'view', String(number), '--owner', owner, '--format', 'json', '--jq', '.id']);
    this.load();
  }

  load() {
    const { owner, number } = this.project;
    const q = `query($o:String!,$n:Int!,$after:String){repositoryOwner(login:$o){... on ProjectV2Owner{projectV2(number:$n){items(first:100,after:$after){pageInfo{hasNextPage endCursor}nodes{id content{... on Issue{number repository{nameWithOwner}}}fieldValues(first:20){nodes{... on ProjectV2ItemFieldSingleSelectValue{name field{... on ProjectV2FieldCommon{name}}}}}}}}}}}`;
    let after = '';
    for (;;) {
      const vars = { o: owner, n: number };
      if (after) vars.after = after;
      const page = graphql(q, vars).repositoryOwner.projectV2.items;
      for (const node of page.nodes) {
        if (node.content?.repository?.nameWithOwner?.toLowerCase() !== repo().nwo.toLowerCase()) continue;
        const fields = {};
        for (const v of node.fieldValues.nodes) if (v?.field?.name) fields[v.field.name] = v.name;
        this.items.set(node.content.number, { itemId: node.id, fields });
      }
      if (!page.pageInfo.hasNextPage) break;
      after = page.pageInfo.endCursor;
    }
  }

  /** Field value of an issue; labels stand in for fields without a Project. */
  get(issue, name) {
    if (this.project) return this.items.get(issue.number)?.fields[name] ?? '';
    const labels = issue.labels.map((l) => l.name ?? l);
    if (name === 'Journey') return labels.find((l) => l.startsWith('journey:'))?.slice(8) ?? '';
    for (const [value, label] of Object.entries(FIELD_LABELS[name] ?? {})) if (labels.includes(label)) return value;
    return { Status: 'Todo', 'Human Review': 'Required', 'AI Ready': 'No' }[name] ?? '';
  }

  ensureItem(issue) {
    let item = this.items.get(issue.number);
    if (!item) {
      const { owner, number } = this.project;
      const added = gh(['project', 'item-add', String(number), '--owner', owner, '--url', issue.url, '--format', 'json'], { json: true });
      item = { itemId: added.id, fields: {} };
      this.items.set(issue.number, item);
    }
    return item;
  }

  set(issue, name, value) {
    if (this.get(issue, name) === value) return;
    if (!this.project) return this.setLabel(issue, name, value);
    const field = this.fields.get(name);
    if (!field) throw new Error(`Project #${this.project.number} has no "${name}" field: run scripts/setup-github.sh --project ${this.project.number}`);
    const option = field.options?.find((o) => o.name === value);
    if (!option) throw new Error(`Project field "${name}" has no option "${value}": run scripts/setup-github.sh --project ${this.project.number}`);
    const item = this.ensureItem(issue);
    gh(['project', 'item-edit', '--id', item.itemId, '--project-id', this.projectId, '--field-id', field.id, '--single-select-option-id', option.id]);
    item.fields[name] = value;
  }

  setLabel(issue, name, value) {
    const labels = issue.labels.map((l) => l.name ?? l);
    const all = name === 'Journey' ? labels.filter((l) => l.startsWith('journey:')) : Object.values(FIELD_LABELS[name] ?? {});
    const want = name === 'Journey' ? (value ? `journey:${value}` : null) : FIELD_LABELS[name]?.[value] ?? null;
    const remove = all.filter((l) => l !== want && labels.includes(l));
    const args = ['issue', 'edit', String(issue.number)];
    for (const l of remove) args.push('--remove-label', l);
    if (want && !labels.includes(want)) args.push('--add-label', want);
    if (args.length > 3) gh(args);
    issue.labels = [...labels.filter((l) => !remove.includes(l)), ...(want ? [want] : [])].map((n) => ({ name: n }));
  }
}

// ---------- issues ----------
const ISSUE_FIELDS = 'number,title,body,state,labels,milestone,assignees,url,id';
const getIssue = (n) => gh(['issue', 'view', String(n), '--json', ISSUE_FIELDS], { json: true });
const openTasks = () => gh(['issue', 'list', '--state', 'open', '--limit', '500', '--search', '"[Task]" in:title', '--json', ISSUE_FIELDS], { json: true })
  .filter((i) => i.title.startsWith('[Task]'));
const taskKey = (title) => title.match(/\bTASK-\d+\b/)?.[0] ?? null;
const issueId = (n) => gh(['api', `repos/${repo().nwo}/issues/${n}`, '--jq', '.id']);

function spec() {
  return specIndex({ features: read('docs/FEATURES.md'), prd: read('docs/PRD.md'), testing: read('docs/TESTING.md') });
}
const appNames = () => {
  const dir = join(ROOT, 'apps');
  const apps = existsSync(dir) ? readdirSync(dir, { withFileTypes: true }).filter((d) => d.isDirectory()).map((d) => d.name) : [];
  return [...apps, 'packages', 'infra', 'repo'];
};

function lint(issue, board, index) {
  return lintTask(issue.body ?? '', index, { apps: appNames(), journey: board.get(issue, 'Journey'), milestone: issue.milestone?.title ?? '' });
}

/** Issues this one is blocked by that are still open (GitHub issue dependencies). */
function openBlockers(n) {
  const deps = gh(['api', `repos/${repo().nwo}/issues/${n}/dependencies/blocked_by`, '--paginate'], { json: true, allowFail: true });
  if (deps) return deps.filter((d) => d.state === 'open').map((d) => d.number);
  // Dependencies API unavailable: fall back to "#123" links in the Dependencies section.
  const dep = (getIssue(n).body ?? '').split(/^## Dependencies\s*$/m)[1]?.split(/^## /m)[0] ?? '';
  return [...new Set((dep.match(/#(\d+)/g) ?? []).map((x) => Number(x.slice(1))))]
    .filter((d) => getIssue(d).state === 'OPEN');
}

/** The latest approval recorded on an issue: { journey, hash } or null. */
function latestApproval(n) {
  const comments = gh(['api', `repos/${repo().nwo}/issues/${n}/comments`, '--paginate'], { json: true }) ?? [];
  for (const c of comments.reverse()) {
    const m = (c.body ?? '').match(APPROVAL_MARK);
    if (m) return { journey: m[1], hash: m[2] };
  }
  return null;
}

/** True if the issue is approved and unchanged since; undoes a stale approval. */
function approvalHolds(issue, board, { fix = true } = {}) {
  if (board.get(issue, 'Human Review') !== 'Approved') return false;
  const a = latestApproval(issue.number);
  if (a && a.hash === approvalHash(issue.body ?? '')) return true;
  if (fix) {
    board.set(issue, 'Human Review', 'Required');
    comment(issue.number, a
      ? 'The task changed after it was approved, so it needs approval again (Human Review = Required). The owner approves it with `scripts/agent approve-journey`.'
      : 'Human Review was Approved without an approval record from `scripts/agent approve-journey`; set back to Required.');
  }
  return false;
}

const comment = (n, body) => gh(['issue', 'comment', String(n), '--body-file', '-'], { input: body });
const claimant = () => process.env.AGENT_NAME || `${process.env.CLAUDECODE ? 'Claude Code' : 'agent'} on ${hostname()}`;

// ---------- commands ----------
const commandsTable = {
  next() {
    const board = new Board();
    const index = spec();
    const candidates = [];
    for (const issue of openTasks()) {
      if (board.get(issue, 'Status') !== 'Todo' || issue.assignees.length) continue;
      if (board.get(issue, 'AI Ready') !== 'Yes' || board.get(issue, 'Human Review') !== 'Approved') continue;
      if (lint(issue, board, index).length || !approvalHolds(issue, board)) continue;
      if (openBlockers(issue.number).length) continue;
      candidates.push({ ...issue, priority: board.get(issue, 'Priority'), milestone: issue.milestone?.title });
    }
    candidates.sort(compareTasks);
    if (!candidates.length) return console.log('No task is ready: nothing is approved, ready, unclaimed and unblocked. See `scripts/agent digest`.');
    for (const c of candidates.slice(0, 5)) console.log(`#${c.number}  ${c.title}  [${c.priority || 'Medium'}, ${milestoneKey(c.milestone) || 'no milestone'}, ${board.get(c, 'Journey')}]`);
    console.log(`\nClaim the first with: scripts/agent claim ${candidates[0].number}`);
  },

  claim([n]) {
    if (!n) throw new Error('Usage: scripts/agent claim <issue>');
    const board = new Board();
    const issue = getIssue(n);
    const problems = [];
    if (issue.state !== 'OPEN') problems.push('the issue is closed');
    if (board.get(issue, 'Status') !== 'Todo') problems.push(`Status is ${board.get(issue, 'Status') || 'unset'}, not Todo`);
    if (board.get(issue, 'AI Ready') !== 'Yes') problems.push('AI Ready is not Yes (scripts/agent ready)');
    if (!approvalHolds(issue, board)) problems.push('not approved: the owner approves its journey with scripts/agent approve-journey');
    const lintProblems = lint(issue, board, spec());
    if (lintProblems.length) problems.push(`not ready: ${lintProblems.join('; ')}`);
    const blockers = openBlockers(issue.number);
    if (blockers.length) problems.push(`blocked by open ${blockers.map((b) => `#${b}`).join(', ')} (early start: AGENTS.md, Execution rules 2)`);
    const linked = gh(['pr', 'list', '--state', 'open', '--search', `${n} in:body`, '--json', 'number,body'], { json: true })
      .filter((p) => new RegExp(`#${n}\\b`).test(p.body));
    if (linked.length) problems.push(`already has an open PR (#${linked[0].number})`);
    if (problems.length) throw new Error(`Can't claim #${n}:\n- ${problems.join('\n- ')}`);

    const branch = branchName(issue.number, issue.title);
    git(['fetch', '--quiet', 'origin'], { cwd: ROOT });
    const def = gh(['repo', 'view', '--json', 'defaultBranchRef', '--jq', '.defaultBranchRef.name']);
    const who = claimant();
    const result = claimBranch({ cwd: ROOT, base: `origin/${def}`, branch, claimant: who });
    if (!result.ok) throw new Error(`Can't claim #${n}: ${result.reason}. Pick another task (scripts/agent next).`);
    try {
      gh(['issue', 'edit', String(n), '--add-assignee', '@me']);
      board.set(issue, 'Status', 'In Progress');
      comment(n, `Claimed by ${who} on branch \`${branch}\`.`);
    } catch (e) {
      releaseBranch({ cwd: ROOT, branch, sha: result.sha });
      throw e;
    }
    console.log(`Claimed #${n} on branch ${branch} (checked out). Status: In Progress.`);
  },

  pr(_, opts) {
    if (!opts.title || !opts['body-file']) throw new Error('Usage: scripts/agent pr --title "<type(scope): …>" --body-file <file> [--draft]');
    const branch = git(['branch', '--show-current'], { cwd: ROOT });
    const n = Number(branch.match(/^(\d+)-/)?.[1]);
    if (!n) throw new Error(`Branch ${branch} isn't named <issue>-<title>; claim the issue with scripts/agent claim <issue>`);
    const body = readFileSync(opts['body-file'], 'utf8');
    if (!new RegExp(`#${n}\\b`).test(body)) throw new Error(`The PR body must link the issue (#${n}), under "Related issue".`);
    git(['push', '--quiet', '-u', 'origin', branch], { cwd: ROOT });
    const args = ['pr', 'create', '--title', opts.title, '--body-file', opts['body-file'], '--head', branch];
    if (opts.draft) args.push('--draft');
    console.log(gh(args));
    if (!opts.draft) new Board().set(getIssue(n), 'Status', 'In Review');
    const risk = localRisk(opts.base);
    console.log(`Risk: ${risk.level}${risk.reasons.length ? ` (${risk.reasons.slice(0, 3).join('; ')})` : ''}`);
    console.log(risk.level === 'high'
      ? 'High risk: the owner approves the merge (human:merge-ok, or merges it).'
      : 'Low risk: once reviewed, post it with scripts/agent review <pr> --body-file <file>.');
    if (risk.level === 'low' && RULES.autoMerge && !opts.draft) {
      gh(['pr', 'merge', branch, '--auto', '--squash'], { allowFail: true });
      console.log('Auto-merge enabled: it merges when ci-ok and risk-gate pass.');
    }
  },

  block([n], opts) {
    if (!n || !opts.reason) throw new Error('Usage: scripts/agent block <issue> --reason "<problem and the input needed>"');
    const issue = getIssue(n);
    new Board().set(issue, 'Status', 'Blocked');
    gh(['issue', 'edit', String(n), '--add-label', 'needs:human-input']);
    comment(n, `**Blocked.** ${opts.reason}`);
    console.log(`#${n} is Blocked and waiting on the owner.`);
  },

  ask([n], opts) {
    if (!n || !opts.question) throw new Error('Usage: scripts/agent ask <issue> --question "<question>"');
    gh(['issue', 'edit', String(n), '--add-label', 'needs:human-input']);
    comment(n, `**Question for the owner:** ${opts.question}\n\nAnswer here; remove \`needs:human-input\` when answered.`);
    console.log(`Asked on #${n}.`);
  },

  ready(nums, opts) {
    const board = new Board();
    const index = spec();
    const issues = opts.all || !nums.length ? openTasks() : nums.map(getIssue);
    let bad = 0;
    for (const issue of issues) {
      const problems = lint(issue, board, index);
      board.set(issue, 'AI Ready', problems.length ? 'No' : 'Yes');
      approvalHolds(issue, board);
      if (problems.length) bad++;
      console.log(`#${issue.number} ${problems.length ? `not ready:\n    - ${problems.join('\n    - ')}` : 'ready'}`);
    }
    if (bad) process.exitCode = 1;
  },

  plan(_, opts) {
    if (!opts.file) throw new Error('Usage: scripts/agent plan --file <plan.json> [--dry-run]');
    const plan = JSON.parse(readFileSync(opts.file, 'utf8'));
    const tasks = Array.isArray(plan) ? plan : plan.tasks;
    const index = spec();
    const milestones = gh(['api', `repos/${repo().nwo}/milestones?state=open&per_page=100`], { json: true }).map((m) => m.title);
    // Check everything before writing anything.
    const errors = [];
    for (const t of tasks) {
      if (!/^TASK-\d{3,}$/.test(t.key ?? '')) errors.push(`${t.key ?? '(no key)'}: key must be TASK-###`);
      if (!t.title || !t.body) errors.push(`${t.key}: needs a title and a body`);
      const ms = milestones.find((m) => milestoneKey(m) === milestoneKey(t.milestone));
      if (!ms) errors.push(`${t.key}: milestone "${t.milestone}" doesn't exist on GitHub (scripts/setup-github.sh --milestones)`);
      else t.milestone = ms;
      const problems = lintTask(t.body ?? '', index, { apps: appNames(), journey: journeyOf(t.body ?? '', index), milestone: t.milestone });
      if (problems.length) errors.push(`${t.key}: ${problems.join('; ')}`);
    }
    if (errors.length) throw new Error(`The plan has problems; nothing was written:\n- ${errors.join('\n- ')}`);

    const existing = new Map(openTasks().map((i) => [taskKey(i.title), i]));
    if (opts['dry-run']) {
      for (const t of tasks) console.log(`${existing.has(t.key) ? 'update' : 'create'} ${t.key}: ${t.title} [${journeyOf(t.body, index)}, ${milestoneKey(t.milestone)}]`);
      return;
    }
    const board = new Board();
    const byKey = new Map();
    for (const t of tasks) {
      const title = `[Task] ${t.key}: ${t.title}`;
      const labels = [...new Set(['ai:planned', ...(t.labels ?? [])])];
      let issue = existing.get(t.key);
      if (issue) {
        const args = ['issue', 'edit', String(issue.number), '--title', title, '--body-file', '-', '--milestone', t.milestone];
        for (const l of labels) args.push('--add-label', l);
        gh(args, { input: t.body });
        issue = getIssue(issue.number);
        console.log(`updated #${issue.number} ${title}`);
      } else {
        const url = gh(['issue', 'create', '--title', title, '--body-file', '-', '--milestone', t.milestone, '--label', labels.join(',')], { input: t.body });
        issue = getIssue(Number(url.split('/').pop()));
        console.log(`created #${issue.number} ${title}`);
        board.set(issue, 'Status', 'Todo');
        board.set(issue, 'Human Review', 'Required');
      }
      board.set(issue, 'Journey', journeyOf(t.body, index));
      if (t.priority) board.set(issue, 'Priority', t.priority);
      if (t.complexity) board.set(issue, 'Complexity', t.complexity);
      board.set(issue, 'AI Ready', lint(issue, board, index).length ? 'No' : 'Yes');
      approvalHolds(issue, board);
      byKey.set(t.key, issue);
    }
    // Relationships once every issue exists: parent/sub-issue and blocked-by.
    const ref = (x) => (typeof x === 'number' ? x : (byKey.get(x) ?? existing.get(x))?.number);
    for (const t of tasks) {
      const issue = byKey.get(t.key);
      if (t.parent && ref(t.parent)) {
        gh(['api', '-X', 'POST', `repos/${repo().nwo}/issues/${ref(t.parent)}/sub_issues`, '-F', `sub_issue_id=${issueId(issue.number)}`], { allowFail: true });
      }
      for (const b of t.blockedBy ?? []) {
        const num = ref(b);
        if (!num) { console.warn(`  ${t.key}: unknown dependency ${b}`); continue; }
        gh(['api', '-X', 'POST', `repos/${repo().nwo}/issues/${issue.number}/dependencies/blocked_by`, '-F', `issue_id=${issueId(num)}`], { allowFail: true });
      }
    }
    console.log('\nThe owner approves the work per journey: scripts/agent approve-journey <J-xx> (or approve-milestone <M#>).');
  },

  'approve-journey'([journey], opts) {
    humanOnly();
    if (!journey) throw new Error('Usage: scripts/agent approve-journey <J-xx|Platform> [--milestone M1] [--dry-run]');
    const index = spec();
    if (journey !== PLATFORM && !index.journeys.has(journey)) throw new Error(`${journey} isn't in docs/FEATURES.md (Journeys)`);
    const wantMs = journey === PLATFORM ? milestoneKey(opts.milestone) : index.journeys.get(journey).milestone;
    if (journey === PLATFORM && !wantMs) throw new Error('Platform work is approved per milestone: add --milestone M1');
    approveJourney(journey, wantMs, new Board(), index, opts['dry-run']);
  },

  'approve-milestone'([ms], opts) {
    humanOnly();
    const key = milestoneKey(ms);
    if (!key) throw new Error('Usage: scripts/agent approve-milestone <M1> [--dry-run]');
    const index = spec();
    const board = new Board();
    const journeys = [...index.journeys].filter(([, j]) => j.milestone === key).map(([id]) => id);
    for (const j of [...journeys, PLATFORM]) approveJourney(j, key, board, index, opts['dry-run']);
  },

  review([pr], opts) {
    if (!pr || !opts['body-file']) throw new Error('Usage: scripts/agent review <pr> --body-file <file>');
    const body = readFileSync(opts['body-file'], 'utf8');
    const first = body.replace(/\r/g, '').split('\n').find((l) => l.trim());
    if (!VERDICT_FIRST_LINE.test(first ?? '')) throw new Error('The review must start with "Verdict: Approve | Request changes | Comment only" (AGENTS.md, Review output).');
    // A comment review: GitHub doesn't let the PR's author approve it, and agents use the owner's login.
    gh(['pr', 'review', String(pr), '--comment', '--body-file', opts['body-file']]);
    const labels = gh(['pr', 'view', String(pr), '--json', 'labels', '--jq', '.labels[].name']).split('\n');
    const label = RULES.agentReviewLabel ?? 'agent:reviewed';
    // Adding the label (with a person's token) re-runs the risk gate from the default branch.
    if (labels.includes(label)) gh(['pr', 'edit', String(pr), '--remove-label', label]);
    gh(['pr', 'edit', String(pr), '--add-label', label]);
    console.log(`Review posted on #${pr}; the risk gate runs again.`);
  },

  risk(_, opts) {
    const r = localRisk(opts.base);
    console.log(`Risk: ${r.level}`);
    for (const x of r.reasons) console.log(`- ${x}`);
  },

  digest() {
    const board = new Board();
    const index = spec();
    const tasks = openTasks();
    const out = [];
    const section = (title, lines) => { if (lines.length) out.push(`## ${title}`, ...lines, ''); };
    const prs = gh(['pr', 'list', '--state', 'open', '--json', 'number,title,isDraft,labels,url'], { json: true });
    const names = (p) => p.labels.map((l) => l.name);
    section('Pull requests waiting for your merge or human:merge-ok', prs
      .filter((p) => !p.isDraft && !names(p).includes(RULES.humanOkLabel) && !names(p).includes(RULES.riskLabels?.low))
      .map((p) => `- #${p.number} ${p.title} (${names(p).find((l) => l.startsWith('risk:')) ?? 'risk not judged yet'})`));
    section('Questions and blocked tasks (needs:human-input)', tasks
      .filter((i) => i.labels.some((l) => l.name === 'needs:human-input'))
      .map((i) => `- #${i.number} ${i.title} (${board.get(i, 'Status')})`));
    const awaiting = new Map();
    for (const i of tasks) {
      if (board.get(i, 'Human Review') === 'Approved' && approvalHolds(i, board, { fix: false })) continue;
      const ready = !lint(i, board, index).length;
      const j = board.get(i, 'Journey') || journeyOf(i.body ?? '', index);
      const key = j === PLATFORM ? `${PLATFORM} ${milestoneKey(i.milestone?.title) || ''}`.trim() : j;
      const e = awaiting.get(key) ?? { ready: 0, notReady: 0 };
      e[ready ? 'ready' : 'notReady']++;
      awaiting.set(key, e);
    }
    section('Journeys waiting for approval', [...awaiting].sort().map(([j, e]) => {
      const name = index.journeys.get(j)?.name;
      const cmd = j.startsWith(PLATFORM) ? `approve-journey Platform --milestone ${j.split(' ')[1] ?? 'M?'}` : `approve-journey ${j}`;
      return `- ${j}${name ? ` ${name}` : ''}: ${e.ready} ready${e.notReady ? `, ${e.notReady} not ready yet` : ''} → scripts/agent ${cmd}`;
    }));
    section('In progress', tasks.filter((i) => board.get(i, 'Status') === 'In Progress').map((i) => `- #${i.number} ${i.title}`));
    console.log(out.length ? out.join('\n') : 'Nothing needs you right now.');
  },
};

function approveJourney(journey, ms, board, index, dry) {
  const issues = openTasks().filter((i) => {
    const j = board.get(i, 'Journey') || journeyOf(i.body ?? '', index);
    return j === journey && (journey !== PLATFORM || milestoneKey(i.milestone?.title) === ms);
  });
  const label = journey === PLATFORM ? `${PLATFORM} (${ms})` : `${journey} ${index.journeys.get(journey)?.name ?? ''}`.trim();
  if (!issues.length) return console.log(`${label}: no open tasks.`);
  console.log(`${label}${dry ? ' (dry run)' : ''}:`);
  for (const issue of issues) {
    const problems = lint(issue, board, index);
    if (problems.length) { console.log(`  skipped #${issue.number} ${issue.title}: ${problems.join('; ')}`); continue; }
    if (approvalHolds(issue, board, { fix: false })) { console.log(`  already approved #${issue.number} ${issue.title}`); continue; }
    if (!dry) {
      board.set(issue, 'Journey', journey);
      board.set(issue, 'AI Ready', 'Yes');
      board.set(issue, 'Human Review', 'Approved');
      comment(issue.number, `<!-- agent:approval journey=${journey} hash=${approvalHash(issue.body ?? '')} -->\nApproved for work as part of ${label}. Changing the task's scope undoes this approval.`);
    }
    console.log(`  approved #${issue.number} ${issue.title}`);
  }
}

function humanOnly() {
  if (process.env.CLAUDECODE || process.env.CLAUDE_CODE_ENTRYPOINT) {
    throw new Error('Approving is a human action: run this in your own terminal, not through an agent.');
  }
}

/** Risk of the current branch against base, as the risk gate judges it (labels aside). */
function localRisk(base = 'origin/main') {
  const files = git(['diff', '--name-status', '-M', `${base}...HEAD`], { cwd: ROOT }).split('\n').filter(Boolean)
    .flatMap((l) => l.split('\t').slice(1));
  const breaking = [];
  for (const f of files.filter((x) => matchesAny(x, RULES.contractFiles ?? []))) {
    const before = (() => { try { return JSON.parse(git(['show', `${base}:${f}`], { cwd: ROOT })); } catch { return null; } })();
    const after = (() => { try { return JSON.parse(git(['show', `HEAD:${f}`], { cwd: ROOT })); } catch { return null; } })();
    const changes = breakingOpenApiChanges(before, after);
    if (changes.length) breaking.push(`${f}: ${changes.slice(0, 3).join('; ')}`);
  }
  return classify([...new Set(files)], [], RULES, { breakingContracts: breaking });
}

// ---------- main ----------
function parse(argv) {
  const pos = [], opts = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (!a.startsWith('--')) { pos.push(a); continue; }
    const [k, v] = a.slice(2).split(/=(.*)/s);
    if (v !== undefined) opts[k] = v;
    else if (argv[i + 1] !== undefined && !argv[i + 1].startsWith('--')) opts[k] = argv[++i];
    else opts[k] = true;
  }
  return { pos, opts };
}

const [cmd, ...rest] = process.argv.slice(2);
const run = commandsTable[cmd];
if (!run) {
  console.log(readFileSync(fileURLToPath(import.meta.url), 'utf8').split('\n').slice(1, 22).map((l) => l.replace(/^\/\/ ?/, '')).join('\n'));
  process.exit(cmd && cmd !== 'help' && cmd !== '--help' ? 2 : 0);
}
const { pos, opts } = parse(rest);
try {
  run(pos, opts);
} catch (e) {
  console.error(e.message);
  process.exit(1);
}
