#!/usr/bin/env node
// Gives svelte-check the TypeScript it can actually run against.
//
// svelte-check (up to the latest release, 4.7.7) loads the classic TypeScript
// JS API via `require('typescript')` and refuses to start when that resolves
// to TypeScript >= 7 (see bin/ts-version-check.js in the svelte-check package).
// TypeScript 7 is a native compiler: it no longer ships lib/typescript.js, so
// no svelte-check release can use it through that entry point. svelte-check's
// own TS7 migration keeps typescript@~6 at the `typescript` name and takes TS7
// through the `@typescript/native` alias — the opposite of this project's
// layout, where `typescript` is 7 and drives the tsc gate
// (scripts/typecheck.mjs), Vite and Vitest.
//
// Both layouts coexist here: package.json carries a `typescript6` alias
// (npm:typescript@^6), and this script links it into svelte-check's own
// resolution path at node_modules/svelte-check/node_modules/typescript, so
// svelte-check — and only svelte-check — sees the classic 6.x API. Everything
// else keeps resolving `typescript` to 7 at the root.
//
// npm cannot express this tree (it reports ERESOLVE rather than nesting a
// second copy, even with overrides), so the link is created here, on demand,
// by the check/check:watch/check:gate scripts. Idempotent: a correct existing
// link is left alone, as is a real directory npm may have placed there.

import { existsSync, lstatSync, mkdirSync, readFileSync, readlinkSync, symlinkSync, rmSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const webRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const aliasDir = join(webRoot, 'node_modules', 'typescript6')
const svelteCheckModules = join(webRoot, 'node_modules', 'svelte-check', 'node_modules')
const linkPath = join(svelteCheckModules, 'typescript')
const linkTarget = join('..', '..', 'typescript6')

if (!existsSync(join(aliasDir, 'package.json'))) {
	console.error(
		'ensure-svelte-check-ts6: node_modules/typescript6 is missing — run `npm ci` in web/.',
	)
	process.exit(2)
}

const major = Number(
	JSON.parse(readFileSync(join(aliasDir, 'package.json'), 'utf8')).version.split('.')[0],
)
if (major < 5 || major > 6) {
	console.error(
		`ensure-svelte-check-ts6: the typescript6 alias resolved to ${major}.x; svelte-check requires TypeScript 5 or 6.`,
	)
	process.exit(2)
}

if (existsSync(linkPath) || lstatSync(linkPath, { throwIfNoEntry: false })?.isSymbolicLink()) {
	// A real directory (npm nesting) already shadows the root for svelte-check;
	// a correct symlink needs no work. Only a stale symlink is replaced.
	if (lstatSync(linkPath).isSymbolicLink()) {
		if (readlinkSync(linkPath) === linkTarget) process.exit(0)
		rmSync(linkPath)
	} else {
		process.exit(0)
	}
}

mkdirSync(svelteCheckModules, { recursive: true })
// 'junction' is required for directory links on Windows and ignored on POSIX.
symlinkSync(linkTarget, linkPath, 'junction')
console.log('ensure-svelte-check-ts6: linked svelte-check/node_modules/typescript -> typescript6')
