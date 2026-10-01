#!/usr/bin/env node
// The release version lives in package.json. Tauri reads it from there
// (tauri.conf.json points at it), the UI gets it at build time (vite.config.ts),
// and the Rust manifests carry a copy this script writes and checks.
//
//   node scripts/version.mjs                 print the version
//   node scripts/version.mjs check [--tag v1.2.3]
//                                            every copy agrees (and with the tag)
//   node scripts/version.mjs set 1.2.3       write it everywhere
//
// Used by `npm run check`, CI, the release workflow and `npm run release`.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { isMain, root } from "./lib.mjs";

const file = (path) => join(root, path);
const read = (path) => readFileSync(file(path), "utf8");
const write = (path, text) => writeFileSync(file(path), text);

/** Semantic version: 1.2.3 or 1.2.3-rc.1. */
export const SEMVER = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$/;

const CARGO_WORKSPACE = /(\[workspace\.package\][^[]*?\nversion\s*=\s*")([^"]+)(")/;
/** The workspace's own packages in Cargo.lock: signal-lab, signal-lab-engine, signal-lab-server. */
const CARGO_LOCK = /(\[\[package\]\]\r?\nname = "signal-lab(?:-[a-z]+)?"\r?\nversion = ")([^"]+)(")/g;
/** Crates of the workspace; each takes its version from the workspace. */
export const MEMBERS = ["engine/Cargo.toml", "src-tauri/Cargo.toml", "server/Cargo.toml"];

/** Every place a version is written, and how to read and replace it. */
export const COPIES = [
  {
    path: "package.json",
    get: (text) => JSON.parse(text).version,
    set: (text, version) => text.replace(/("version"\s*:\s*")[^"]+(")/, `$1${version}$2`),
  },
  {
    path: "package-lock.json",
    get: (text) => { const lock = JSON.parse(text); return lock.version === lock.packages?.[""]?.version ? lock.version : `${lock.version} / ${lock.packages?.[""]?.version}`; },
    // The root entry and packages[""] are the first two "version" keys of the file.
    set: (text, version) => { let left = 2; return text.replace(/("version"\s*:\s*")[^"]+(")/g, (match, a, b) => left-- > 0 ? `${a}${version}${b}` : match); },
  },
  {
    path: "Cargo.toml",
    get: (text) => CARGO_WORKSPACE.exec(text)?.[2],
    set: (text, version) => text.replace(CARGO_WORKSPACE, `$1${version}$3`),
  },
  {
    path: "Cargo.lock",
    // Every workspace package must carry the same version; a mix reads as such.
    get: (text) => { const found = [...new Set([...text.matchAll(CARGO_LOCK)].map((match) => match[2]))]; return found.length ? found.join(" / ") : undefined; },
    set: (text, version) => text.replace(CARGO_LOCK, `$1${version}$3`),
  },
];

export function currentVersion() {
  return JSON.parse(read("package.json")).version;
}

/** Problems with the versions on disk; empty when everything agrees. */
export function versionProblems(tag) {
  const version = currentVersion();
  const problems = [];
  if (!SEMVER.test(version)) problems.push(`package.json: "${version}" is not a semantic version`);
  for (const copy of COPIES) {
    const found = copy.get(read(copy.path));
    if (found !== version) problems.push(`${copy.path}: ${found ?? "no version"} (package.json says ${version})`);
  }
  for (const member of MEMBERS.filter((path) => existsSync(file(path)))) {
    if (!/^version\.workspace\s*=\s*true$/m.test(read(member))) problems.push(`${member}: must take the workspace version (version.workspace = true)`);
  }
  const tauri = JSON.parse(read("src-tauri/tauri.conf.json")).version;
  if (tauri !== "../package.json") problems.push(`src-tauri/tauri.conf.json: "version" must be "../package.json", found "${tauri}"`);
  if (tag !== undefined && tag !== `v${version}`) problems.push(`tag ${tag} does not match version ${version} (expected v${version})`);
  return problems;
}

export function setVersion(version) {
  if (!SEMVER.test(version)) throw new Error(`"${version}" is not a semantic version (1.2.3 or 1.2.3-rc.1)`);
  for (const copy of COPIES) {
    const text = read(copy.path);
    const next = copy.set(text, version);
    if (copy.get(next) !== version) throw new Error(`could not write the version into ${copy.path}`);
    write(copy.path, next);
  }
}

if (isMain(import.meta.url)) {
  const [command, ...rest] = process.argv.slice(2);
  try {
    if (!command) {
      console.log(currentVersion());
    } else if (command === "check") {
      const at = rest.indexOf("--tag");
      const problems = versionProblems(at >= 0 ? rest[at + 1] : undefined);
      if (problems.length) {
        console.error("Version mismatch:\n" + problems.map((problem) => `  - ${problem}`).join("\n"));
        console.error("Fix with: node scripts/version.mjs set <version>");
        process.exit(1);
      }
      console.log(`version ${currentVersion()} — all copies agree`);
    } else if (command === "set" && rest[0]) {
      setVersion(rest[0]);
      console.log(`version set to ${rest[0]}`);
    } else {
      console.error("usage: node scripts/version.mjs [check [--tag vX.Y.Z] | set X.Y.Z]");
      process.exit(2);
    }
  } catch (error) {
    console.error(String(error.message ?? error));
    process.exit(1);
  }
}
