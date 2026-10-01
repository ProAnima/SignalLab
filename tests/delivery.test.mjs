import test from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { compareVersions } from "../scripts/release.mjs";
import { COPIES, MEMBERS, SEMVER, currentVersion, versionProblems } from "../scripts/version.mjs";
import { promoted, releaseNotes, section } from "../scripts/changelog.mjs";
import { STEPS } from "../scripts/check.mjs";
import { REPOSITORY, imageTags } from "../scripts/image.mjs";

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

  const cargo = `[workspace]\nmembers = ["engine"]\n\n[workspace.package]\nversion = "0.3.1"\nedition = "2021"\n\n[workspace.dependencies]\nserde = { version = "1" }\ntokio = { version = "1", features = [] }\n`;
  const manifest = copy("Cargo.toml").set(cargo, "0.4.0");
  assert.equal(copy("Cargo.toml").get(manifest), "0.4.0");
  assert.ok(manifest.includes('serde = { version = "1" }') && manifest.includes('tokio = { version = "1"'));

  for (const newline of ["\n", "\r\n"]) {
    const cargoLock = ["[[package]]", 'name = "serde"', 'version = "1.0.0"', "", "[[package]]", 'name = "signal-lab"', 'version = "0.3.1"', "dependencies = [", "]", "",
      "[[package]]", 'name = "signal-lab-engine"', 'version = "0.3.1"', "", "[[package]]", 'name = "signal-labyrinth"', 'version = "0.3.1"', ""].join(newline);
    const updated = copy("Cargo.lock").set(cargoLock, "0.4.0");
    assert.equal(copy("Cargo.lock").get(updated), "0.4.0", "every workspace package moves together");
    assert.ok(updated.includes(`name = "serde"${newline}version = "1.0.0"`) && updated.includes(`name = "signal-labyrinth"${newline}version = "0.3.1"`), "only the workspace's packages change");
    assert.equal(copy("Cargo.lock").get(cargoLock.replace('name = "signal-lab-engine"' + newline + 'version = "0.3.1"', 'name = "signal-lab-engine"' + newline + 'version = "0.3.0"')), "0.3.1 / 0.3.0", "a mix is reported");
  }
});

test("this checkout's versions agree, and Tauri reads package.json", () => {
  assert.deepEqual(versionProblems(), []);
  for (const member of MEMBERS.filter((path) => existsSync(new URL(`../${path}`, import.meta.url)))) {
    assert.match(read(member), /^version\.workspace = true$/m, `${member} takes the workspace version`);
  }
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
  assert.ok(notes.includes("ghcr.io/proanima/signallab:0.4.0") && notes.includes("--network host"), "the notes name the image");
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
    "versions agree", "UI unit tests", "UI type check and build", "Rust lints (clippy, warnings are errors)", "Rust tests",
  ]);
  const clippy = STEPS.find((step) => step.command === "cargo" && step.args[0] === "clippy");
  assert.ok(clippy.args.includes("warnings") && clippy.args.includes("--workspace"));
  assert.ok(STEPS.find((step) => step.command === "cargo" && step.args[0] === "test").args.includes("--workspace"));
  const ci = read(".github/workflows/ci.yml");
  assert.match(ci, /run: node scripts\/check\.mjs/);
  assert.match(ci, /windows-latest/);
  assert.match(ci, /ubuntu-22\.04/);
  assert.match(ci, /xvfb-run -a node scripts\/webkit\.mjs --native/, "tooltips are checked in the Linux webview");
  const release = read(".github/workflows/release.yml");
  assert.match(release, /uses: \.\/\.github\/workflows\/ci\.yml/);
  assert.match(release, /needs: checks/);
  assert.match(release, /scripts\/version\.mjs check --tag/);
  assert.match(release, /bundles: nsis,msi/);
  assert.match(release, /bundles: deb,rpm,appimage/);
});

test("a release's image tags never move backwards", () => {
  const release = (version, prerelease = false) => ({ version, prerelease });
  const published = ["0.3.0", "0.3.1", "0.4.0-rc.1", "0.4.0", "0.4.1", "0.5.0-beta"].map((version) => release(version, version.includes("-")));
  assert.deepEqual(imageTags("0.4.1", published), ["0.4.1", "0.4", "latest"]);
  assert.deepEqual(imageTags("0.4.0", published), ["0.4.0"], "an older patch keeps only its own tag");
  assert.deepEqual(imageTags("0.3.2", [...published, release("0.3.2")]), ["0.3.2", "0.3"], "a fix for an older line moves that line, not latest");
  assert.deepEqual(imageTags("0.5.0-beta", published), ["0.5.0-beta"], "a pre-release is never latest");
  assert.deepEqual(imageTags("1.0.0", published), ["1.0.0", "1.0", "latest"], "the version itself counts as published");
  // Marked pre-release on GitHub by a person: treated as one, and it does not hold back later releases.
  const marked = [...published, release("0.4.2", true)];
  assert.deepEqual(imageTags("0.4.2", marked), ["0.4.2"]);
  assert.deepEqual(imageTags("0.4.1", marked), ["0.4.1", "0.4", "latest"]);
  assert.equal(REPOSITORY, "ghcr.io/proanima/signallab", "registry names are lower case");
});

test("the image is built with the pinned toolchains and runs without privileges", () => {
  const dockerfile = read("Dockerfile").replace(/\r\n/g, "\n");
  const arg = (name, text) => new RegExp(`^ARG ${name}=(\\S+)$`, "m").exec(text)?.[1];
  const channel = /^channel\s*=\s*"([^"]+)"/m.exec(read("rust-toolchain.toml"))[1];
  assert.equal(arg("RUST_VERSION", dockerfile), channel, "Rust as in rust-toolchain.toml");
  assert.equal(arg("NODE_VERSION", dockerfile), arg("NODE_VERSION", read("docker/linux-builder/Dockerfile")), "Node as in the Linux builder");
  assert.match(dockerfile, /cargo build --release --locked -p signal-lab-server/, "only the server is built, from the lock file");
  assert.match(dockerfile, /^USER 10001:10001$/m);
  assert.match(dockerfile, /^HEALTHCHECK .*\n\s+CMD \["signal-lab-server", "healthcheck"\]$/m);
  assert.match(dockerfile, /SIGNALLAB_LISTEN=0\.0\.0\.0:1430/, "every address, which the server allows only with a token");
  assert.doesNotMatch(dockerfile, /SIGNALLAB_TOKEN=/, "no token is baked into the image");
  // Only what the build copies goes into its context.
  const ignore = read(".dockerignore").split(/\r?\n/).filter((line) => line && !line.startsWith("#"));
  assert.equal(ignore[0], "**");
  assert.ok(ignore.includes("!src-tauri/Cargo.toml"));
  assert.ok(!ignore.some((line) => /^!(src-tauri\/\*\*|target|node_modules|\.git|dist)/.test(line)), "no build output or local state");
  const compose = read("deploy/compose.yaml");
  assert.match(compose, /network_mode: host/);
  assert.match(compose, /SIGNALLAB_TOKEN_FILE: \/run\/secrets\//, "the token comes from a secret file");
});

test("CI smoke-tests the image, and only a published release reaches the registry", () => {
  const ci = read(".github/workflows/ci.yml");
  assert.match(ci, /node scripts\/image\.mjs smoke/);
  assert.match(ci, /ubuntu-24\.04-arm/);
  const image = read(".github/workflows/image.yml").replace(/\r\n/g, "\n");
  assert.match(image, /release:\n\s+types: \[published\]/);
  assert.doesNotMatch(image, /^\s+push:/m, "never on a push");
  assert.match(image, /scripts\/version\.mjs check --tag/);
  assert.ok(image.indexOf("image.mjs smoke") < image.indexOf("push-by-digest=true"), "smoke-tested before it is pushed");
  assert.match(image, /sbom: true/);
  assert.match(image, /actions\/attest-build-provenance/);
  assert.ok(image.includes(`IMAGE: ${REPOSITORY}\n`));
});
