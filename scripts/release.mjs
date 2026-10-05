#!/usr/bin/env node
// Cut a release from this machine. GitHub Actions then builds the installers
// for Windows and Linux from the tag into a draft release; you review the
// draft and publish it. See docs/develop/delivery.md.
//
//   npm run release -- 1.2.3 --dry-run   check everything, change nothing
//   npm run release -- 1.2.3             checks, version, changelog, commit, tag — local only
//   npm run release -- 1.2.3 --push      the same, then push main and the tag together
//
// Refuses unless: on main, nothing uncommitted, not behind origin/main, the tag
// is new, the version is higher than the current one, CHANGELOG.md has
// something under Unreleased, and `npm run check` passes.

import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { check } from "./check.mjs";
import { promote, releaseNotes, section } from "./changelog.mjs";
import { fail, isMain, must, root, run } from "./lib.mjs";
import { SEMVER, currentVersion, setVersion, versionProblems } from "./version.mjs";

const REPO = "https://github.com/ProAnima/SignalLab";
const FILES = ["package.json", "package-lock.json", "Cargo.toml", "Cargo.lock", "CHANGELOG.md"];

/** Semantic-version order: numbers, then a pre-release sorts before its release. */
export function compareVersions(a, b) {
  const parse = (version) => { const [core, pre] = version.split("-", 2); return { core: core.split(".").map(Number), pre: pre ? pre.split(".") : [] }; };
  const [x, y] = [parse(a), parse(b)];
  for (let i = 0; i < 3; i++) if (x.core[i] !== y.core[i]) return x.core[i] - y.core[i];
  if (!x.pre.length || !y.pre.length) return y.pre.length - x.pre.length;
  for (let i = 0; i < Math.max(x.pre.length, y.pre.length); i++) {
    const [p, q] = [x.pre[i], y.pre[i]];
    if (p === undefined) return -1;
    if (q === undefined) return 1;
    const [m, n] = [/^\d+$/.test(p), /^\d+$/.test(q)];
    if (m && n && Number(p) !== Number(q)) return Number(p) - Number(q);
    if (m !== n) return m ? -1 : 1;
    if (p !== q) return p < q ? -1 : 1;
  }
  return 0;
}

/** A git command whose output is read, not shown. */
const git = (...args) => must("git", args, { capture: true });

function preconditions(version, tag) {
  const problems = [];
  if (!SEMVER.test(version)) problems.push(`"${version}" is not a semantic version (1.2.3 or 1.2.3-rc.1)`);
  else if (compareVersions(version, currentVersion()) <= 0) problems.push(`${version} is not higher than the current version ${currentVersion()}`);
  problems.push(...versionProblems());
  const branch = git("branch", "--show-current");
  if (branch !== "main") problems.push(`releases are cut from main, not ${branch || "a detached HEAD"}`);
  const dirty = git("status", "--porcelain");
  if (dirty) problems.push(`uncommitted changes — commit or stash them first:\n${dirty.split("\n").slice(0, 10).map((line) => `      ${line}`).join("\n")}`);
  git("fetch", "origin", "--tags", "--quiet");
  const behind = Number(git("rev-list", "--count", "HEAD..origin/main"));
  if (behind > 0) problems.push(`main is ${behind} commit(s) behind origin/main — pull first`);
  if (run("git", ["rev-parse", "-q", "--verify", `refs/tags/${tag}`], { capture: true }).ok) problems.push(`tag ${tag} already exists`);
  if (!section("Unreleased")) problems.push("CHANGELOG.md has nothing under Unreleased — write what this release changes");
  return problems;
}

function main() {
  const args = process.argv.slice(2);
  const version = (args.find((arg) => !arg.startsWith("--")) ?? "").replace(/^v/, "");
  const push = args.includes("--push");
  const dryRun = args.includes("--dry-run");
  if (!version) fail("usage: npm run release -- <version> [--dry-run | --push]");
  const tag = `v${version}`;

  console.log(`Release ${tag}${dryRun ? " (dry run)" : ""}\n`);
  const problems = preconditions(version, tag);
  if (problems.length) fail(`not releasing:\n${problems.map((problem) => `  - ${problem}`).join("\n")}`);
  console.log("✔ preconditions: main, clean, up to date, new tag, changelog written");

  if (!check()) fail("the checks failed; nothing was changed");

  if (dryRun) {
    console.log(`\nDry run: would set ${version} in ${FILES.join(", ")}, date the changelog, commit "Release ${version}", tag ${tag}${push ? " and push both" : ""}.`);
    console.log(`\nRelease notes would be:\n\n${section("Unreleased")}`);
    return;
  }

  setVersion(version);
  promote(version, new Date().toISOString().slice(0, 10));
  // The lock file was edited by hand above; cargo must still accept it as is.
  must("cargo", ["metadata", "--locked", "--format-version", "1", "--no-deps"], { cwd: root, capture: true });

  releaseNotes(version); // throws if the promoted section is missing
  const folder = mkdtempSync(join(tmpdir(), "signallab-release-"));
  const message = join(folder, "message.txt");
  try {
    writeFileSync(message, `Release ${version}\n\n${section(version)}\n`);
    git("add", ...FILES);
    git("commit", "--quiet", "-F", message);
    git("tag", "-a", tag, "-F", message);
  } finally {
    rmSync(folder, { recursive: true, force: true });
  }
  console.log(`\n✔ committed "Release ${version}" and tagged ${tag}`);

  if (push) {
    // Atomic: main and the tag land together or not at all.
    must("git", ["push", "--atomic", "origin", "main", tag]);
    console.log(`✔ pushed main and ${tag}`);
    console.log(`\nGitHub Actions is building the installers: ${REPO}/actions/workflows/release.yml`);
    console.log(`When it finishes, review and publish the draft: ${REPO}/releases`);
  } else {
    console.log(`\nNothing was pushed. To publish this release:\n  git push --atomic origin main ${tag}`);
    console.log(`To undo it instead:\n  git tag -d ${tag} && git reset --hard HEAD~1`);
  }
}

if (isMain(import.meta.url)) main();
