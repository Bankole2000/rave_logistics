// The risk-gate check (.github/workflows/risk-gate.yml), run by actions/github-script.
// It runs from the default branch (pull_request_target), so a pull request can't change
// the rules that judge it, and it never checks out or runs the pull request's code.
//
// Passes when:
//   - the PR has the human:merge-ok label (a human approved this exact head commit), or
//   - the PR is low risk AND an agent review of its head commit says "Verdict: Approve".
// New commits remove human:merge-ok and agent:reviewed, so approvals never carry over to
// code nobody looked at.
import { readFileSync } from 'node:fs';
import { classify, breakingOpenApiChanges, matchesAny } from './risk.mjs';

const VERDICT = /^\s*Verdict:\s*(Approve|Request changes|Comment only)\b/im;

export async function run({ github, context, core, rulesPath = '.github/risk-rules.json' }) {
  const rules = JSON.parse(readFileSync(rulesPath, 'utf8'));
  const { owner, repo } = context.repo;
  const number = context.payload.pull_request.number;
  const { data: pr } = await github.rest.pulls.get({ owner, repo, pull_number: number });
  const head = pr.head.sha;
  let labels = pr.labels.map((l) => l.name);

  // New commits invalidate earlier approvals and reviews.
  if (context.payload.action === 'synchronize') {
    for (const l of [rules.humanOkLabel, rules.agentReviewLabel]) {
      if (labels.includes(l)) {
        await github.rest.issues.removeLabel({ owner, repo, issue_number: number, name: l }).catch(() => {});
        labels = labels.filter((x) => x !== l);
        core.info(`Removed ${l}: new commits need a new review.`);
      }
    }
  }

  const files = [];
  for await (const page of github.paginate.iterator(github.rest.pulls.listFiles, { owner, repo, pull_number: number, per_page: 100 })) {
    for (const f of page.data) {
      files.push(f.filename);
      if (f.previous_filename) files.push(f.previous_filename);
    }
  }

  const breakingContracts = [];
  for (const f of files.filter((x) => matchesAny(x, rules.contractFiles ?? []))) {
    const [base, next] = await Promise.all([
      readJson(github, owner, repo, f, pr.base.sha),
      readJson(github, owner, repo, f, head),
    ]);
    const changes = breakingOpenApiChanges(base, next);
    if (changes.length) breakingContracts.push(`${f}: ${changes.slice(0, 5).join('; ')}${changes.length > 5 ? '; …' : ''}`);
  }

  const risk = classify([...new Set(files)], labels, rules, { breakingContracts });
  const want = rules.riskLabels[risk.level];
  const drop = rules.riskLabels[risk.level === 'high' ? 'low' : 'high'];
  if (!labels.includes(want)) await github.rest.issues.addLabels({ owner, repo, issue_number: number, labels: [want] });
  if (labels.includes(drop)) await github.rest.issues.removeLabel({ owner, repo, issue_number: number, name: drop }).catch(() => {});

  const reviews = await github.paginate(github.rest.pulls.listReviews, { owner, repo, pull_number: number, per_page: 100 });
  const onHead = reviews.filter((r) => r.commit_id === head && VERDICT.test(r.body ?? ''));
  const latest = onHead.at(-1);
  const verdict = latest ? latest.body.match(VERDICT)[1] : null;
  const humanOk = labels.includes(rules.humanOkLabel);

  const lines = [
    `Risk: **${risk.level}**`,
    ...risk.reasons.slice(0, 20).map((r) => `- ${r}`),
    `Agent review of ${head.slice(0, 7)}: ${verdict ?? 'none yet'}`,
    `Human approval (${rules.humanOkLabel}): ${humanOk ? 'yes' : 'no'}`,
  ];
  await core.summary.addRaw(lines.join('\n')).write();
  core.info(lines.join('\n'));

  const result = decide({ draft: pr.draft, humanOk, risk, verdict, rules });
  core.info(`${result.state}: ${result.description}`);
  // The result is a commit status on the head commit, the "risk-gate" required check. The
  // job itself only fails on errors: a pull_request_target run belongs to the base branch.
  await github.rest.repos.createCommitStatus({
    owner, repo, sha: head, state: result.state, context: rules.statusContext ?? 'risk-gate',
    description: result.description.slice(0, 140),
    target_url: `${context.serverUrl}/${owner}/${repo}/actions/runs/${context.runId}`,
  });
  return result;
}

/** The gate's decision, as a commit status: success, failure or pending. */
export function decide({ draft, humanOk, risk, verdict, rules }) {
  if (draft) return { state: 'pending', description: 'Draft: checked again when marked ready.' };
  if (humanOk) return { state: 'success', description: `Approved by a human (${rules.humanOkLabel}).` };
  if (risk.level === 'high') {
    return { state: 'failure', description: `High risk: needs ${rules.humanOkLabel} or a human merge. ${risk.reasons[0] ?? ''}` };
  }
  if (verdict !== 'Approve') {
    return verdict
      ? { state: 'failure', description: `Agent review of the head commit: ${verdict}.` }
      : { state: 'pending', description: 'Waiting for an agent review of the head commit (scripts/agent review).' };
  }
  return { state: 'success', description: 'Low risk, agent review approved.' };
}

async function readJson(github, owner, repo, path, ref) {
  try {
    const { data } = await github.rest.repos.getContent({ owner, repo, path, ref });
    return JSON.parse(Buffer.from(data.content, 'base64').toString('utf8'));
  } catch (e) {
    if (e.status === 404) return null;
    throw e;
  }
}
