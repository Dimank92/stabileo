const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const {test} = require('node:test');
const report = require('../../.github/scripts/nightly-findings.cjs');

const finding = {test: 'opensees/mismatch', seed: 42, model: {loads: [1], elements: {a: {}}},
  command: 'python3 scripts/nightly-models.py --model model.json', reduction: '1-minimal', details: {status: 'mismatch'}};

async function fixture(fn) {
  const previous = process.cwd();
  const dry = process.env.FINDINGS_DRY_RUN;
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'nightly-reporter-'));
  process.chdir(tmp);
  fs.mkdirSync('nightly-findings');
  const calls = [];
  const issues = [];
  const comments = [];
  const logs = [];
  const api = Object.fromEntries(['getLabel', 'createLabel', 'listForRepo', 'listComments', 'create', 'createComment', 'update'].map(name =>
    [name, async args => {
      calls.push({name, args});
      if (name === 'getLabel') throw Object.assign(new Error('missing label'), {status: 404});
      if (name === 'create') return {data: {...args, number: 1, state: 'open'}};
      return {data: {}};
    }]));
  const args = {github: {rest: {issues: api}, paginate: async method => method === api.listForRepo ? issues : comments},
    context: {repo: {owner: 'owner', repo: 'repo'}, serverUrl: 'https://github.com', runId: 123, sha: 'abc'},
    core: {info: message => logs.push(message)}};
  const save = (name, value = finding) => fs.writeFileSync(`nightly-findings/${name}.finding.json`, JSON.stringify(value));
  try {
    process.env.FINDINGS_DRY_RUN = 'false';
    await fn({args, calls, issues, comments, logs, save});
  } finally {
    process.chdir(previous);
    if (dry === undefined) delete process.env.FINDINGS_DRY_RUN;
    else process.env.FINDINGS_DRY_RUN = dry;
    fs.rmSync(tmp, {recursive: true, force: true});
  }
}

test('dry run deduplicates reordered models without writing labels, issues or comments', () => fixture(async ({args, calls, logs, save}) => {
  save('a');
  save('b', {...finding, model: {elements: {a: {}}, loads: [1]}});
  process.env.FINDINGS_DRY_RUN = 'true';
  await report(args);
  assert.equal(calls.length, 0);
  assert.equal(logs.length, 1);
  assert.match(logs[0], /\[dry run\] create opensees\/mismatch/);
  assert.ok(logs[0].includes(finding.command));
  assert.deepEqual(JSON.parse(logs[0].split('```json\n')[1].split('\n```')[0]), finding.model);
}));

test('write creates one issue per test/model and required labels', () => fixture(async ({args, calls, save}) => {
  save('a');
  save('b', {...finding, model: {elements: {a: {}}, loads: [1]}});
  save('c', {...finding, test: 'opensees/false_reject'});
  await report(args);
  assert.deepEqual(calls.filter(c => c.name === 'createLabel').map(c => c.args.name), ['safety', 'nightly-finding']);
  const created = calls.filter(c => c.name === 'create');
  assert.equal(created.length, 2);
  assert.match(created[0].args.body, /Seed: `42`/);
  assert.match(created[0].args.body, /actions\/runs\/123/);
  assert.deepEqual(JSON.parse(created[0].args.body.split('```json\n')[1].split('\n```')[0]), finding.model);
  assert.deepEqual(created[0].args.labels, ['safety', 'nightly-finding']);
}));

test('existing closed issue gets occurrence comment and reopens; repeated run is idempotent', () => fixture(async ({args, calls, issues, comments, save}) => {
  save('a');
  await report(args);
  const created = calls.find(c => c.name === 'create');
  issues.length = 0;
  issues.push({number: 7, state: 'closed', body: created.args.body.replace(/<!-- nightly-run:[^>]+ -->/, '<!-- nightly-run:old:1 -->')});
  calls.length = 0;
  await report(args);
  const comment = calls.find(c => c.name === 'createComment');
  assert.equal(comment.args.issue_number, 7);
  assert.ok(comment.args.body.includes(finding.command));
  assert.equal(calls.find(c => c.name === 'update').args.state, 'open');
  assert.equal(calls.filter(c => c.name === 'create').length, 0);
  comments.push({body: comment.args.body});
  issues[0].state = 'open';
  calls.length = 0;
  await report(args);
  assert.equal(calls.filter(c => ['create', 'createComment', 'update'].includes(c.name)).length, 0);
  process.env.FINDINGS_DRY_RUN = 'true';
  calls.length = 0;
  await report(args);
  assert.equal(calls.length, 0);
}));

test('empty artifacts produce no writes and malformed findings fail', () => fixture(async ({args, calls, logs, save}) => {
  await report(args);
  assert.equal(calls.length, 0);
  assert.match(logs[0], /No structured numerical findings/);
  save('bad', {test: 'missing model'});
  await assert.rejects(report(args), /Invalid finding/);
  assert.equal(calls.length, 0);
}));
