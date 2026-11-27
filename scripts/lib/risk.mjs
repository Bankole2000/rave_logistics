// Risk classification for pull requests (docs/GITHUB.md, "Merging").
// Pure functions, shared by the risk-gate workflow (scripts/lib/risk-gate.mjs) and
// `node scripts/agent.mjs risk`, so both classify a change the same way.
// The rules live in .github/risk-rules.json.

/** Converts a glob (`**` any path, `*` anything but `/`, `?` one character) to a RegExp. */
export function globToRegExp(glob) {
  let re = '';
  for (let i = 0; i < glob.length; i++) {
    const c = glob[i];
    if (c === '*') {
      if (glob[i + 1] === '*') {
        // `**/` matches zero or more folders; a trailing `**` matches the rest.
        if (glob[i + 2] === '/') { re += '(?:.*/)?'; i += 2; } else { re += '.*'; i += 1; }
      } else {
        re += '[^/]*';
      }
    } else if (c === '?') {
      re += '[^/]';
    } else {
      re += c.replace(/[.+^${}()|[\]\\]/g, '\\$&');
    }
  }
  return new RegExp(`^${re}$`);
}

export function matchesAny(path, globs) {
  return globs.find((g) => globToRegExp(g).test(path));
}

/** Labels may be written with a trailing `*` (e.g. `review:*`). */
function labelMatches(label, pattern) {
  return pattern.endsWith('*') ? label.startsWith(pattern.slice(0, -1)) : label === pattern;
}

/**
 * Classifies a change.
 * @param {string[]} files   changed paths (renamed files: both old and new path)
 * @param {string[]} labels  labels on the pull request
 * @param {object}   rules   .github/risk-rules.json
 * @param {{breakingContracts?: string[]}} [extra]  contract files with breaking changes
 * @returns {{level: 'high'|'low', reasons: string[]}}
 */
export function classify(files, labels, rules, extra = {}) {
  const reasons = [];
  const contractGlobs = rules.contractFiles ?? [];
  for (const f of files) {
    if (matchesAny(f, contractGlobs)) continue; // judged by content below
    const g = matchesAny(f, rules.highRiskPaths ?? []);
    if (g) { reasons.push(`${f} (matches ${g})`); continue; }
    if (rules.keywordScope && matchesAny(f, rules.keywordScope)) {
      const kw = new RegExp(`(^|[/_.-])(${(rules.highRiskKeywords ?? []).join('|')})([/_.-]|$)`, 'i');
      const m = (rules.highRiskKeywords ?? []).length ? f.match(kw) : null;
      if (m) reasons.push(`${f} (sensitive area: ${m[2].toLowerCase()})`);
    }
  }
  for (const c of extra.breakingContracts ?? []) reasons.push(`${c} (breaking API contract change)`);
  for (const l of labels) {
    if ((rules.highRiskLabels ?? []).some((p) => labelMatches(l, p))) reasons.push(`label ${l}`);
  }
  return { level: reasons.length ? 'high' : 'low', reasons };
}

/**
 * Lists breaking changes between two OpenAPI documents: removed paths or operations,
 * removed schemas or schema properties, newly required properties or parameters.
 * Additive changes are not breaking (docs/TECHNICAL.md, compatibility rules).
 */
export function breakingOpenApiChanges(base, head) {
  if (!base) return [];
  if (!head) return ['contract file removed'];
  const out = [];
  const methods = ['get', 'put', 'post', 'delete', 'patch', 'head', 'options'];
  for (const [p, ops] of Object.entries(base.paths ?? {})) {
    const h = head.paths?.[p];
    if (!h) { out.push(`path removed: ${p}`); continue; }
    for (const m of methods) {
      if (!ops[m]) continue;
      if (!h[m]) { out.push(`operation removed: ${m.toUpperCase()} ${p}`); continue; }
      const baseReq = new Set((ops[m].parameters ?? []).filter((x) => x.required).map((x) => `${x.in}:${x.name}`));
      for (const x of h[m].parameters ?? []) {
        if (x.required && !baseReq.has(`${x.in}:${x.name}`)) out.push(`new required parameter: ${x.in} ${x.name} on ${m.toUpperCase()} ${p}`);
      }
    }
  }
  const bs = base.components?.schemas ?? {};
  const hs = head.components?.schemas ?? {};
  for (const [name, s] of Object.entries(bs)) {
    const t = hs[name];
    if (!t) { out.push(`schema removed: ${name}`); continue; }
    for (const prop of Object.keys(s.properties ?? {})) {
      if (!(t.properties ?? {})[prop]) out.push(`property removed: ${name}.${prop}`);
    }
    const req = new Set(s.required ?? []);
    for (const r of t.required ?? []) if (!req.has(r)) out.push(`newly required: ${name}.${r}`);
  }
  return out;
}
