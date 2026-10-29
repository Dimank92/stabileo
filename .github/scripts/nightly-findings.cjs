// Artifact data is never executed. This is the only nightly job with issue access.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const hash = x => crypto.createHash('sha256').update(x).digest('hex');
const canonical = x => Array.isArray(x) ? `[${x.map(canonical).join(',')}]` :
  x !== null && typeof x === 'object' ? `{${Object.keys(x).sort().map(k => `${JSON.stringify(k)}:${canonical(x[k])}`).join(',')}}` : JSON.stringify(x);
function files(dir) {
  if (!fs.existsSync(dir)) return [];
  return fs.readdirSync(dir, {withFileTypes: true}).flatMap(e =>
    e.isDirectory() ? files(path.join(dir, e.name)) : e.isFile() && e.name.endsWith('.finding.json') ? [path.join(dir, e.name)] : []);
}
module.exports = async ({github, context, core}) => {
  const {owner, repo} = context.repo;
  const dry = process.env.FINDINGS_DRY_RUN === 'true';
  const findings = new Map();
  for (const file of files('nightly-findings')) {
    const f = JSON.parse(fs.readFileSync(file, 'utf8'));
    if (typeof f.test !== 'string' || typeof f.command !== 'string' || f.seed === undefined || !f.model) throw Error(`Invalid finding: ${file}`);
    const modelHash = hash(canonical(f.model));
    const key = hash(`${f.test}\n${modelHash}`);
    findings.set(key, {...f, modelHash});
  }
  if (!findings.size) { core.info('No structured numerical findings; see job logs for infrastructure failures.'); return; }
  if (!dry) for (const name of ['safety', 'nightly-finding']) {
    try { await github.rest.issues.getLabel({owner, repo, name}); }
    catch (e) {
      if (e.status !== 404) throw e;
      await github.rest.issues.createLabel({owner, repo, name, color: 'b60205'});
    }
  }
  const issues = await github.paginate(github.rest.issues.listForRepo, {owner, repo, labels: 'nightly-finding', state: 'all', per_page: 100});
  const run = `${context.serverUrl}/${owner}/${repo}/actions/runs/${context.runId}`;
  for (const [key, f] of findings) {
    const marker = `<!-- nightly-finding:${key} -->`;
    const occurrence = `<!-- nightly-run:${context.runId}:${process.env.GITHUB_RUN_ATTEMPT || '1'} -->`;
    const model = JSON.stringify(f.model, null, 2);
    // Large JSON remains complete in the artifact: never silently truncate it.
    const inline = model.length < 40000 ? `Save as model.json:\n\n\`\`\`json\n${model}\n\`\`\`` : `Model JSON exceeds the issue body limit; download the finding JSON from the [run artifacts](${run}) and save its model field as model.json.`;
    const body = `${marker}\n${occurrence}\nTest: **${f.test}**\n\nSeed: \`${f.seed}\` · model SHA-256: \`${f.modelHash}\`\n\n[Run and artifacts](${run}) · commit \`${context.sha}\`\n\nReduction: ${f.reduction}\n\nBuild/replay setup: [NIGHTLY.md](${context.serverUrl}/${owner}/${repo}/blob/${context.sha}/docs/verification/NIGHTLY.md).\n\n\`\`\`sh\n${f.command}\n\`\`\`\n\n${inline}\n\nDetails:\n\n\`\`\`text\n${JSON.stringify(f.details).slice(0, 4000)}\n\`\`\``;
    const existing = issues.find(i => !i.pull_request && i.body?.includes(marker));
    if (dry) { core.info(`[dry run] ${existing ? `update #${existing.number}` : 'create'} ${f.test}\n${body}`); continue; }
    if (existing) {
      const comments = await github.paginate(github.rest.issues.listComments, {owner, repo, issue_number: existing.number, per_page: 100});
      if (!existing.body.includes(occurrence) && !comments.some(c => c.body?.includes(occurrence))) {
        await github.rest.issues.createComment({owner, repo, issue_number: existing.number, body});
      }
      if (existing.state === 'closed') await github.rest.issues.update({owner, repo, issue_number: existing.number, state: 'open'});
    } else {
      const result = await github.rest.issues.create({owner, repo, title: `Nightly finding: ${f.test} [${key.slice(0, 12)}]`.slice(0, 250), body, labels: ['safety', 'nightly-finding']});
      issues.push(result.data);
    }
  }
};
