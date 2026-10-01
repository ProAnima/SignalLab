import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { compareVersions } from "../scripts/release.mjs";
import { COPIES, SEMVER, currentVersion, versionProblems } from "../scripts/version.mjs";
import { promoted, releaseNotes, section } from "../scripts/changelog.mjs";
import { STEPS } from "../scripts/check.mjs";

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const copy = (path) => COPIES.find((item) => item.path === path);

test("versions order like semantic versioning, pre-releases before their release", () => {
  const ordered = ["0.3.1", "0.4.0-alpha", "0.4.0-alpha.2", "0.4.0-beta", "0.4.0-rc.1", "0.4.0-rc.10", "0.4.0", "0.10.0", "1.0.0"];
  for (let i = 0; i < ordered.length - 1; i++) {
    assert.ok(compareVersions(ordered[i], ordered[i + 1]) < 0, `${ordered[i]} < ${ordered[i + 1]}`);
    assert.ok(compareVersions(ordered[i + 1], ordered[i]) > 0);
  }
  assert.equal(compareVersions("0.4.0", "0.4.0"), 0);
  for (const good of ["0.4.0", "1.2.3-rc.1", "10.0.0-beta.2"]) assert.ok(SEMVER.test(good), good);
  for (const bad of ["1.2", "v1.2.3", "01.2.3", "1.2.3-", "1.2.3 "]) assert.ok(!SEMVER.test(bad), bad);
});

test("the version is rewritten only where it is ours, never in dependencies", () => {
  const lock = `{\n  "name": "signal-lab",\n  "version": "0.3.1",\n  "lockfileVersion": 3,\n  "packages": {\n    "": {\n      "name": "signal-lab",\n      "version": "0.3.1"\n    },\n    "node_modules/react": {\n      "version": "19.1.0"\n    }\n  }\n}\n`;
  const written = copy("package-lock.json").set(lock, "0.4.0");
  assert.equal(copy("package-lock.json").get(written), "0.4.0");
  assert.ok(written.includes('"version": "19.1.0"'), "a dependency keeps its version");

  const cargo = `[package]\nname = "signal-lab"\nversion = "0.3.1"\n\n[dependencies]\nserde = { version = "1" }\ntauri = { version = "2", features = [] }\n`;
  const manifest = copy("src-tauri/Cargo.toml").set(cargo, "0.4.0");
  assert.equal(copy("src-tauri/Cargo.toml").get(manifest), "0.4.0");
  assert.ok(manifest.includes('serde = { version = "1" }') && manifest.includes('tauri = { version = "2"'));

  for (const newline of ["\n", "\r\n"]) {
    const cargoLock = ["[[package]]", 'name = "serde"', 'version = "1.0.0"', "", "[[package]]", 'name = "signal-lab"', 'version = "0.3.1"', "dependencies = [", "]", "", "[[package]]", 'name = "signal-lab-helper"', 'version = "0.3.1"', ""].join(newline);
    const updated = copy("src-tauri/Cargo.lock").set(cargoLock, "0.4.0");
    assert.equal(copy("src-tauri/Cargo.lock").get(updated), "0.4.0");
    assert.ok(updated.includes(`name = "serde"${newline}version = "1.0.0"`) && updated.includes(`name = "signal-lab-helper"${newline}version = "0.3.1"`), "only signal-lab changes");
  }
});

test("this checkout's versions agree, and Tauri reads package.json", () => {
  assert.deepEqual(versionProblems(), []);
  assert.equal(JSON.parse(read("src-tauri/tauri.conf.json")).version, "../package.json");
  assert.ok(versionProblems("v0.0.0").some((problem) => problem.includes("does not match")));
  assert.ok(SEMVER.test(currentVersion()));
});

test("a release turns Unreleased into a dated section with a fresh Unreleased above it", () => {
  const text = `# Changelog\n\n## [Unreleased]\n\n### Added\n\n- Waits.\n\n## [0.3.1] - 2026-08-27\n\n### Fixed\n\n- Stop.\n\n[Unreleased]: https://github.com/ProAnima/SignalLab/compare/v0.3.1...HEAD\n[0.3.1]: https://github.com/ProAnima/SignalLab/compare/v0.3.0...v0.3.1\n`;
  const next = promoted(text, "0.4.0", "2026-10-02");
  assert.equal(section("Unreleased", next), "");
  assert.equal(section("0.4.0", next), "### Added\n\n- Waits.");
  assert.match(next, /^## \[0\.4\.0\] - 2026-10-02$/m);
  assert.match(next, /^\[Unreleased\]: .*compare\/v0\.4\.0\.\.\.HEAD$/m);
  assert.match(next, /^\[0\.4\.0\]: .*compare\/v0\.3\.1\.\.\.v0\.4\.0$/m);
  assert.equal(section("0.3.1", next), "### Fixed\n\n- Stop.", "older sections are untouched");
  assert.throws(() => promoted(next, "0.4.1", "2026-10-03"), /Unreleased section is empty/);
  assert.throws(() => promoted(text, "0.3.1", "2026-10-03"), /already has a section/);
  const notes = releaseNotes("0.4.0", next);
  assert.ok(notes.startsWith("### Added") && notes.includes("SHA256SUMS.txt") && notes.includes("not code-signed"));
});

test("the changelog has an Unreleased section and notes for the current and past versions", () => {
  const changelog = read("CHANGELOG.md").replace(/\r\n/g, "\n");
  // Empty right after a release, so only its presence is required; `npm run release` refuses an empty one.
  assert.notEqual(section("Unreleased", changelog), null, "an Unreleased section exists");
  assert.match(changelog, /^\[Unreleased\]: /m);
  const released = [...changelog.matchAll(/^## \[(\d[^\]]*)\] - \d{4}-\d{2}-\d{2}$/gm)].map((found) => found[1]);
  assert.ok(released.length >= 3, "dated sections for past releases");
  for (const version of released) assert.ok(section(version, changelog), `${version} has notes`);
  // The version in package.json was released (has notes) unless it is the next one being prepared.
  assert.ok(released.includes(currentVersion()) || section("Unreleased", changelog), `${currentVersion()} has notes`);
});

test("CI runs the one list of checks, and a release runs CI first", () => {
  assert.deepEqual(STEPS.map((step) => step.name), [
    "versions agree", "UI unit tests", "UI type check and build", "engine lints (clippy, warnings are errors)", "engine tests",
  ]);
  assert.ok(STEPS.find((step) => step.command === "cargo" && step.args[0] === "clippy").args.includes("warnings"));
  const ci = read(".github/workflows/ci.yml");
  assert.match(ci, /run: node scripts\/check\.mjs/);
  assert.match(ci, /windows-latest/);
  assert.match(ci, /ubuntu-22\.04/);
  const release = read(".github/workflows/release.yml");
  assert.match(release, /uses: \.\/\.github\/workflows\/ci\.yml/);
  assert.match(release, /needs: checks/);
  assert.match(release, /scripts\/version\.mjs check --tag/);
  assert.match(release, /bundles: nsis,msi/);
  assert.match(release, /bundles: deb,rpm,appimage/);
});
