import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { globToRegExp, classify, breakingOpenApiChanges } from '../lib/risk.mjs';

const rules = JSON.parse(readFileSync(new URL('../../.github/risk-rules.json', import.meta.url), 'utf8'));

test('globs', () => {
  assert.ok(globToRegExp('**/migrations/**').test('apps/api/migrations/1_init.sql'));
  assert.ok(globToRegExp('**/AGENTS.md').test('AGENTS.md'));
  assert.ok(globToRegExp('docs/*.md').test('docs/PRD.md'));
  assert.ok(!globToRegExp('docs/*.md').test('docs/reviews/PRD.md'));
  assert.ok(globToRegExp('file?.txt').test('file1.txt'));
  assert.ok(!globToRegExp('a.b').test('axb'), 'dots are literal');
});

test('ordinary app code is low risk', () => {
  const r = classify(['apps/api/src/routes/health.rs', 'apps/api/CHANGELOG.md', 'docs/reviews/PRD-review.md'], [], rules);
  assert.deepEqual(r, { level: 'low', reasons: [] });
});

test('sensitive paths, keywords and labels are high risk', () => {
  for (const f of ['apps/api/migrations/2_x.sql', '.github/workflows/ci.yml', 'scripts/agent.mjs', 'docs/FEATURES.md',
    'apps/api/Cargo.toml', 'apps/api/src/auth/mod.rs', 'apps/web/src/payments/form.tsx', 'apps/api/src/routes/staff_roles.rs']) {
    assert.equal(classify([f], [], rules).level, 'high', f);
  }
  // Keywords match whole path segments, not parts of words.
  assert.equal(classify(['apps/web/src/author.tsx'], [], rules).level, 'low');
  assert.equal(classify(['apps/web/src/a.ts'], ['review:security'], rules).level, 'high');
  assert.equal(classify(['apps/web/src/a.ts'], ['type:feature'], rules).level, 'low');
});

test('contract files are judged by their changes', () => {
  const f = 'packages/api-contract/openapi.json';
  assert.equal(classify([f], [], rules).level, 'low');
  assert.equal(classify([f], [], rules, { breakingContracts: [`${f}: path removed`] }).level, 'high');
});

test('breaking OpenAPI changes', () => {
  const base = {
    paths: { '/a': { get: { parameters: [] } }, '/b': { post: {} } },
    components: { schemas: { User: { properties: { id: {}, phone: {} }, required: ['id'] } } },
  };
  assert.deepEqual(breakingOpenApiChanges(base, base), []);
  const additive = structuredClone(base);
  additive.paths['/c'] = { get: {} };
  additive.components.schemas.User.properties.email = {};
  assert.deepEqual(breakingOpenApiChanges(base, additive), []);
  const breaking = structuredClone(base);
  delete breaking.paths['/b'];
  delete breaking.components.schemas.User.properties.phone;
  breaking.components.schemas.User.required.push('email');
  breaking.paths['/a'].get.parameters.push({ in: 'query', name: 'q', required: true });
  assert.deepEqual(breakingOpenApiChanges(base, breaking).sort(), [
    'new required parameter: query q on GET /a',
    'newly required: User.email',
    'path removed: /b',
    'property removed: User.phone',
  ]);
  assert.deepEqual(breakingOpenApiChanges(null, base), []);
  assert.deepEqual(breakingOpenApiChanges(base, null), ['contract file removed']);
});
