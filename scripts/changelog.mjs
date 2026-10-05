#!/usr/bin/env node
// CHANGELOG.md is the source of release notes (Keep a Changelog layout).
//
//   node scripts/changelog.mjs notes 1.2.3      release notes for that version
//   node scripts/changelog.mjs unreleased       what the next release would say
//
// `promote` (used by `npm run release`) turns **Unreleased** into the version's
// section and starts a new empty one.

import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { isMain, root } from "./lib.mjs";

const PATH = join(root, "CHANGELOG.md");
const REPO = "https://github.com/ProAnima/SignalLab";

const read = () => readFileSync(PATH, "utf8").replace(/\r\n/g, "\n");
const escape = (text) => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/** The body of `## [name]` (without its heading), or null. */
export function section(name, text = read()) {
  const heading = new RegExp(`^## \\[${escape(name)}\\][^\\n]*\\n`, "m").exec(text);
  if (!heading) return null;
  const rest = text.slice(heading.index + heading[0].length);
  const end = rest.search(/^## \[|^\[[^\]]+\]: /m);
  return (end < 0 ? rest : rest.slice(0, end)).trim();
}

/** The version of the newest released section. */
function latestRelease(text) {
  return /^## \[(\d[^\]]*)\]/m.exec(text)?.[1] ?? null;
}

/** `text` with Unreleased turned into `version` dated `date`, and an empty Unreleased above it. */
export function promoted(text, version, date) {
  const body = section("Unreleased", text);
  if (!body) throw new Error("CHANGELOG.md: the Unreleased section is empty — write what this release changes first");
  if (section(version, text) !== null) throw new Error(`CHANGELOG.md already has a section for ${version}`);
  const previous = latestRelease(text);
  const next = text.replace(/^## \[Unreleased\][^\n]*\n/m, `## [Unreleased]\n\n## [${version}] - ${date}\n`);
  const link = previous ? `${REPO}/compare/v${previous}...v${version}` : `${REPO}/releases/tag/v${version}`;
  return /^\[Unreleased\]: /m.test(next)
    ? next.replace(/^\[Unreleased\]: .*$/m, `[Unreleased]: ${REPO}/compare/v${version}...HEAD\n[${version}]: ${link}`)
    : `${next.trimEnd()}\n\n[Unreleased]: ${REPO}/compare/v${version}...HEAD\n[${version}]: ${link}\n`;
}

/** Promote Unreleased in CHANGELOG.md itself. */
export function promote(version, date) {
  writeFileSync(PATH, promoted(read(), version, date));
}

/** What a release page says: the section plus how to check what you downloaded. */
export function releaseNotes(version, text = read()) {
  const body = section(version, text);
  if (!body) throw new Error(`CHANGELOG.md has no section for ${version}`);
  return `${body}

---

**Downloads.** Windows: \`*-setup.exe\` (per-user or per-machine) or \`*.msi\`. Linux: \`.deb\`, \`.rpm\` or \`.AppImage\` (x64).
Verify a download against \`SHA256SUMS.txt\`. Installers are not code-signed yet, so Windows SmartScreen may warn about an unknown publisher.

**Server image.** \`docker pull ghcr.io/proanima/signallab:${version}\` (x64 and arm64), published a few minutes after this release. On a Linux host run it with \`--network host\` so broadcast, multicast and discovery reach the network; see [running a server](https://proanima.github.io/SignalLab/server/).

**Documentation.** https://proanima.github.io/SignalLab/ in eleven languages — and inside the app and the server: press F1.

**Responsible use.** Storm, Scanner and Broadcast send real traffic to real hosts. Point them only at equipment you own or are authorised to test.
`;
}

if (isMain(import.meta.url)) {
  const [command, version] = process.argv.slice(2);
  try {
    if (command === "notes" && version) process.stdout.write(releaseNotes(version.replace(/^v/, "")));
    else if (command === "unreleased") console.log(section("Unreleased") || "(empty)");
    else {
      console.error("usage: node scripts/changelog.mjs notes <version> | unreleased");
      process.exit(2);
    }
  } catch (error) {
    console.error(String(error.message ?? error));
    process.exit(1);
  }
}
