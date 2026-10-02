#!/usr/bin/env node
// The server image (Dockerfile), from any machine with Docker.
//
//   npm run check:image                    build signallab:dev and smoke-test it
//   node scripts/image.mjs build [name]    build only
//   node scripts/image.mjs smoke <name>    smoke-test an image that is already built (CI does this)
//   node scripts/image.mjs tags vX.Y.Z     the registry tags a published release gets (image workflow)
//
// The smoke test runs the container the way it is meant to run — a token from
// a file, a read-only root filesystem, no capabilities, a named volume for
// /data — and checks what a browser and a script rely on: the refusal without
// a token when making one is off, a bare start that makes one and keeps it,
// health, sign-in, commands, the event stream, writes to /data, the command
// line `signallab` (here and against the server), and a clean exit on SIGTERM.

import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { compareVersions } from "./release.mjs";
import { currentVersion } from "./version.mjs";
import { fail, isMain, root, run, seconds } from "./lib.mjs";

export const REPOSITORY = "ghcr.io/proanima/signallab";
const LOCAL = "signallab:dev";
const TOKEN_PATH = "/run/secrets/signallab_token";

/**
 * Tags for `version` among the published releases (`{version, prerelease}`):
 * always the version itself; for a stable release also `X.Y` while it is the
 * newest of its line, and `latest` while it is the newest overall — so
 * rebuilding an older release never moves a tag backwards. A pre-release —
 * by its version (`-rc.1`) or because a person marked it so on GitHub — gets
 * only its own tag.
 */
export function imageTags(version, published) {
  const stableRelease = (release) => !release.prerelease && !release.version.includes("-");
  const tags = [version];
  const self = published.find((release) => release.version === version) ?? { version, prerelease: false };
  if (!stableRelease(self)) return tags;
  const stable = [...published.filter(stableRelease).map((release) => release.version), version];
  const newest = (list) => list.every((other) => compareVersions(version, other) >= 0);
  const lineOf = (other) => other.split(".").slice(0, 2).join(".");
  if (newest(stable.filter((other) => lineOf(other) === lineOf(version)))) tags.push(lineOf(version));
  if (newest(stable)) tags.push("latest");
  return tags;
}

const docker = (args) => run("docker", args, { capture: true });

function mustDocker(args, what) {
  const result = docker(args);
  if (!result.ok) throw new Error(`${what}: docker ${args[0]} failed (exit ${result.status})\n${(result.stderr || result.stdout).trim()}`);
  return result.stdout.trim();
}

function check(condition, message) {
  if (!condition) throw new Error(message);
}

function build(name) {
  const started = Date.now();
  console.log(`▶ docker build -t ${name} (the first build compiles every dependency; later ones reuse them)`);
  if (!run("docker", ["build", "-t", name, root]).ok) fail("the image did not build");
  const size = Number(mustDocker(["image", "inspect", "--format", "{{.Size}}", name], "image size"));
  console.log(`✔ built ${name} · ${(size / 1048576).toFixed(0)} MiB · ${seconds(started)}`);
}

async function waitHealthy(container) {
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    const state = JSON.parse(mustDocker(["inspect", "--format", "{{json .State}}", container], "container state"));
    if (!state.Running) throw new Error(`the container stopped (exit ${state.ExitCode})\n${docker(["logs", container]).stderr}`);
    if (state.Health?.Status === "healthy") return;
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error(`not healthy after 60 s\n${docker(["logs", container]).stderr}`);
}

/** The event stream as a page sees it; `next(test)` is the first event that passes. */
function openEvents(url, headers) {
  // Node's WebSocket (undici) takes headers, so the session cookie goes with the upgrade.
  const socket = new WebSocket(url, { headers });
  const opened = new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", () => reject(new Error("the event stream refused the connection")), { once: true });
  });
  const next = (test, timeout = 10_000) =>
    new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`no matching event within ${timeout / 1000} s`)), timeout);
      socket.addEventListener("message", function listener(message) {
        const event = JSON.parse(String(message.data));
        if (!test(event)) return;
        clearTimeout(timer);
        socket.removeEventListener("message", listener);
        resolve(event);
      });
    });
  return { opened, next, close: () => socket.close() };
}

async function smoke(image) {
  const started = Date.now();
  const version = currentVersion();
  const steps = [];
  const step = async (name, body) => {
    await body();
    steps.push(name);
    console.log(`✔ ${name}`);
  };

  console.log(`▶ smoke test of ${image}`);
  await step(`reports version ${version}`, () => {
    const printed = mustDocker(["run", "--rm", image, "--version"], "--version");
    check(printed === `signal-lab-server ${version}`, `--version printed "${printed}"`);
  });
  await step("refuses to listen on every address without a token when it may not make one (exit 2)", () => {
    const result = docker(["run", "--rm", "--env", "SIGNALLAB_GENERATE_TOKEN=false", image]);
    check(result.status === 2 && result.stderr.includes("without a token"), `exit ${result.status}: ${result.stderr.trim()}`);
  });
  await step("has the command line: signallab version, validate, run (read-only, /tmp in memory)", () => {
    const cli = (...args) => docker(["run", "--rm", "--read-only", "--tmpfs", "/tmp", "--cap-drop", "ALL", "--entrypoint", "signallab", image, ...args]);
    const printed = cli("version");
    check(printed.ok && printed.stdout.trim() === `signallab ${version}`, `signallab version printed "${printed.stdout.trim()}"`);
    const valid = cli("validate", "empty", "http-check");
    check(valid.status === 0, `validate → exit ${valid.status}: ${valid.stderr.trim()}`);
    const ran = cli("--json", "run", "empty");
    const summary = ran.stdout.trim().split(/\r?\n/).map((line) => JSON.parse(line)).at(-1);
    check(ran.status === 0 && summary?.type === "summary" && summary.passed === 1, `run → exit ${ran.status}: ${ran.stdout}${ran.stderr}`);
    const unknown = cli("run", "no-such-thing");
    check(unknown.status === 2, `an unknown experiment → exit ${unknown.status}`);
  });
  let token = "";
  await step("generates tokens", () => {
    token = mustDocker(["run", "--rm", image, "token"], "token");
    check(/^[0-9a-f]{64}$/.test(token), "the token is not 64 hex characters");
  });

  // Under target/ (ignored by git): an ASCII path Docker Desktop can always mount.
  const scratch = join(root, "target", "image-smoke");
  const container = `signallab-smoke-${process.pid}-${Date.now().toString(36)}`;
  const volume = `${container}-data`;
  mkdirSync(scratch, { recursive: true });
  const tokenFile = join(scratch, `${container}.token`);
  // Readable by the container's user (uid 10001), as a Docker secret is.
  writeFileSync(tokenFile, `${token}\n`, { mode: 0o644 });
  try {
    await step("starts hardened: read-only root, no capabilities, token from a file", async () => {
      mustDocker([
        "run", "--detach", "--name", container,
        "--read-only", "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
        "--publish", "127.0.0.1::1430",
        "--mount", `type=bind,source=${tokenFile},target=${TOKEN_PATH},readonly`,
        "--mount", `type=volume,source=${volume},target=/data`,
        "--env", `SIGNALLAB_TOKEN_FILE=${TOKEN_PATH}`,
        image,
      ], "start");
      await waitHealthy(container);
    });

    const port = mustDocker(["port", container, "1430/tcp"], "port").split(/\r?\n/)[0].split(":").pop();
    const base = `http://127.0.0.1:${port}`;
    const bearer = { authorization: `Bearer ${token}` };
    const invoke = (command, args, headers) =>
      fetch(`${base}/api/invoke/${command}`, { method: "POST", headers: { "content-type": "application/json", ...headers }, body: JSON.stringify(args ?? null) });

    await step("health answers without a token, with the version", async () => {
      const health = await (await fetch(`${base}/api/health`)).json();
      check(health.status === "ok" && health.version === version && health.auth === true, JSON.stringify(health));
    });
    await step("commands need the token", async () => {
      const denied = await invoke("app_info");
      check(denied.status === 401 && (await denied.json()).code === "auth.required", `status ${denied.status}`);
      const info = await (await invoke("app_info", null, bearer)).json();
      check(info.mode === "server" && info.data_dir === "/data" && info.secrets_writable === false, JSON.stringify(info));
    });
    let cookie = "";
    await step("a browser is sent to sign in, and a session cookie follows", async () => {
      const page = await fetch(`${base}/`, { redirect: "manual" });
      check(page.status === 303 && page.headers.get("location") === "/login", `GET / → ${page.status}`);
      check((await (await fetch(`${base}/login`)).text()).includes("<form"), "no sign-in form");
      const form = { method: "POST", redirect: "manual", headers: { "content-type": "application/x-www-form-urlencoded" } };
      const wrong = await fetch(`${base}/login`, { ...form, body: "token=wrong" });
      check(wrong.status === 401, `a wrong token → ${wrong.status}`);
      const signedIn = await fetch(`${base}/login`, { ...form, body: `token=${token}` });
      const set = signedIn.headers.get("set-cookie") ?? "";
      check(signedIn.status === 303 && set.includes("HttpOnly") && set.includes("SameSite=Strict"), `sign-in → ${signedIn.status}`);
      cookie = set.split(";")[0];
      check((await fetch(`${base}/`, { headers: { cookie } })).status === 200, "the interface is not served after sign-in");
    });
    await step("documents are written to the /data volume", async () => {
      const document = await (await invoke("experiment_load", null, { cookie })).json();
      const saved = await invoke("experiment_save", { document }, { cookie });
      check(saved.status === 200 && (await saved.json()) === "/data/experiment.json", `experiment_save → ${saved.status}`);
      const owner = mustDocker(["exec", container, "stat", "-c", "%u", "/data/experiment.json"], "stat");
      check(owner === "10001", `the file belongs to uid ${owner}`);
    });
    await step("events reach a signed-in page over the WebSocket", async () => {
      const events = openEvents(`ws://127.0.0.1:${port}/api/events`, { cookie });
      try {
        await events.opened;
        const batch = events.next((item) => item.event === "inspect://batch" && item.payload.frames.some((frame) => frame.summary === "/smoke"));
        await invoke("inspect_set_enabled", { enabled: true }, { cookie });
        const sent = await invoke("osc_send", { target: "127.0.0.1:9", address: "/smoke", args: [] }, { cookie });
        check(sent.status === 200, `osc_send → ${sent.status}`);
        await batch;
      } finally {
        events.close();
      }
    });
    await step("signallab runs on the server: signallab run --server, with the token", () => {
      const remote = docker(["run", "--rm", "--network", `container:${container}`, "--env", `SIGNALLAB_TOKEN=${token}`, "--entrypoint", "signallab", image,
        "--json", "run", "empty", "--server", "http://127.0.0.1:1430"]);
      const summary = remote.stdout.trim().split(/\r?\n/).map((line) => JSON.parse(line)).at(-1);
      check(remote.status === 0 && summary?.passed === 1, `exit ${remote.status}: ${remote.stdout}${remote.stderr}`);
      const refused = docker(["run", "--rm", "--network", `container:${container}`, "--entrypoint", "signallab", image, "run", "empty", "--server", "http://127.0.0.1:1430"]);
      check(refused.status === 3, `without the token → exit ${refused.status}`);
    });
    await step("stops cleanly on SIGTERM (exit 0)", () => {
      const at = Date.now();
      mustDocker(["stop", "--time", "20", container], "stop");
      const code = mustDocker(["inspect", "--format", "{{.State.ExitCode}}", container], "exit code");
      check(code === "0", `exit ${code} after ${seconds(at)} — SIGTERM was not handled`);
    });
    const logs = docker(["logs", container]);
    check(!`${logs.stdout}${logs.stderr}`.includes("panicked"), `the server panicked:\n${logs.stderr}`);
  } catch (error) {
    const logs = docker(["logs", "--tail", "40", container]);
    throw new Error(`${error.message}${logs.ok ? `\n--- container log ---\n${`${logs.stdout}${logs.stderr}`.trim()}` : ""}`);
  } finally {
    docker(["rm", "--force", container]);
    docker(["volume", "rm", "--force", volume]);
    rmSync(tokenFile, { force: true });
  }

  // Nothing set up at all: `docker run` with a volume, as the README shows it.
  const bare = `${container}-bare`;
  const bareVolume = `${bare}-data`;
  try {
    await step("a bare start makes a token, shows it once and keeps it across a restart", async () => {
      mustDocker([
        "run", "--detach", "--name", bare, "--read-only", "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
        "--publish", "127.0.0.1::1430", "--mount", `type=volume,source=${bareVolume},target=/data`, image,
      ], "start without a token");
      await waitHealthy(bare);
      const shown = /Sign in with it:\s+([0-9a-f]{64})/.exec(docker(["logs", bare]).stderr)?.[1];
      check(!!shown, "the first start shows the token it made");
      const kept = mustDocker(["exec", bare, "cat", "/data/token"], "cat /data/token");
      check(kept === shown, "/data/token holds the token that was shown");
      const mode = mustDocker(["exec", bare, "stat", "-c", "%a %u", "/data/token"], "stat");
      check(mode === "600 10001", `/data/token is ${mode}, not 600 for uid 10001`);
      const port = mustDocker(["port", bare, "1430/tcp"], "port").split(/\r?\n/)[0].split(":").pop();
      const call = (headers) => fetch(`http://127.0.0.1:${port}/api/invoke/app_info`, { method: "POST", headers: { "content-type": "application/json", ...headers }, body: "null" });
      check((await call({})).status === 401, "a command without the token is refused");
      check((await call({ authorization: `Bearer ${shown}` })).status === 200, "the token that was shown opens it");
      mustDocker(["restart", "--time", "20", bare], "restart");
      await waitHealthy(bare);
      const logs = docker(["logs", bare]);
      const text = `${logs.stdout}${logs.stderr}`;
      check(text.split("Sign in with it:").length === 2, "the token is shown once, not again after the restart");
      check(text.includes("access token from the data folder"), "the restart says where the token came from");
      check(mustDocker(["exec", bare, "cat", "/data/token"], "cat /data/token") === shown, "the restart keeps the token");
    });
  } catch (error) {
    const logs = docker(["logs", "--tail", "40", bare]);
    throw new Error(`${error.message}${logs.ok ? `\n--- container log ---\n${`${logs.stdout}${logs.stderr}`.trim()}` : ""}`);
  } finally {
    docker(["rm", "--force", bare]);
    docker(["volume", "rm", "--force", bareVolume]);
  }
  console.log(`\n✔ ${image}: ${steps.length} checks passed · ${seconds(started)}`);
}

async function tags(tag) {
  const { publishedReleases } = await import("./github-release.mjs");
  const version = tag.replace(/^v/, "");
  const published = await publishedReleases();
  if (!published.some((release) => release.version === version)) throw new Error(`${tag} is not a published release; the image follows publication`);
  for (const name of imageTags(version, published)) console.log(name);
}

async function main() {
  const [mode, name] = process.argv.slice(2);
  if (mode === "tags") {
    if (!/^v\d/.test(name ?? "")) fail("usage: node scripts/image.mjs tags vX.Y.Z");
    return tags(name);
  }
  if (!["check", "build", "smoke"].includes(mode) || (mode === "smoke" && !name)) {
    fail("usage: node scripts/image.mjs check | build [name] | smoke <name> | tags vX.Y.Z");
  }
  if (!docker(["info"]).ok) fail("Docker is not running — start Docker Desktop first");
  if (mode !== "smoke") build(name ?? LOCAL);
  if (mode !== "build") await smoke(name ?? LOCAL);
}

// No process.exit after network calls: on Windows it can abort Node while
// sockets are still closing. Setting the exit code lets them finish.
if (isMain(import.meta.url)) {
  main().catch((error) => {
    console.error(`\n✖ ${error.message ?? error}`);
    process.exitCode = 1;
  });
}
