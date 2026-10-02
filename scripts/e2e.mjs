#!/usr/bin/env node
// Every screen, end to end, in the real app: the server in a browser and the
// desktop app, each walked through a tour of every tab (tests/e2e/tour.ts)
// that sends real traffic to loopback fixtures (scripts/e2e/fixtures.mjs) and
// checks what arrived at the far end.
//
//   npm run e2e                               here: the server in a browser and the desktop app
//   npm run e2e -- --target server            only the server (Edge or Chrome; WebKitGTK on Linux)
//   npm run e2e -- --target desktop           only the desktop app (WebView2; WebKitGTK on Linux)
//   npm run e2e:linux                         both on Linux, in Docker (Ubuntu 22.04, WebKitGTK)
//   npm run e2e:linux -- --image signallab:dev   …the server from that image instead of a build
//   node scripts/e2e.mjs --native             inside the Linux container, or on Linux CI
//
//   --steps osc,mqtt     only these steps (the first, shell, always runs)
//   --no-build           use what the last run built
//   --server-url URL     a server that is already running (its data folder: --data-dir)
//
// Screenshots of every step and report.md land in artifacts/e2e/<platform>-<target>/.

import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { freePort, startFixtures } from "./e2e/fixtures.mjs";
import { currentVersion } from "./version.mjs";
import { fail, isMain, root, run, seconds } from "./lib.mjs";

const OUT = join(root, "artifacts", "e2e");
const BUILD = join(root, "target", "e2e");
const EXE = process.platform === "win32" ? ".exe" : "";
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const STEP_TIMEOUT = 120_000;
/** No single call to the browser takes longer: a hung page fails the tour instead of stalling it. */
const CALL_TIMEOUT = 60_000;

// ---- options ------------------------------------------------------------------------

function options(argv) {
  const value = (name) => { const at = argv.indexOf(name); return at >= 0 ? argv[at + 1] : undefined; };
  const target = value("--target");
  if (target && !["server", "desktop"].includes(target)) fail("--target is server or desktop");
  return {
    targets: target ? [target] : ["server", "desktop"],
    steps: value("--steps")?.split(",").map((name) => name.trim()).filter(Boolean) ?? null,
    build: !argv.includes("--no-build"),
    native: argv.includes("--native"),
    linux: argv.includes("--linux"),
    image: value("--image"),
    serverUrl: value("--server-url"),
    dataDir: value("--data-dir"),
  };
}

// ---- building -----------------------------------------------------------------------

async function bundleTour() {
  const esbuild = await import("esbuild");
  mkdirSync(BUILD, { recursive: true });
  const file = join(BUILD, "tour.js");
  await esbuild.build({
    entryPoints: [join(root, "tests", "e2e", "tour.ts")],
    bundle: true, outfile: file, format: "iife", target: "es2020", logLevel: "warning",
  });
  return readFileSync(file, "utf8");
}

function build(opts) {
  const step = (name, command, args, env) => {
    const at = Date.now();
    console.log(`▶ ${name}`);
    if (!run(command, args, { env }).ok) fail(`${name} failed`);
    console.log(`✔ ${name} · ${seconds(at)}`);
  };
  if (opts.targets.includes("server") && !opts.serverUrl) {
    step("interface (npm run build)", "npm", ["run", "build"]);
    step("server (cargo build)", "cargo", ["build", "--locked", "-p", "signal-lab-server"]);
  }
  // A debug build of the real app, its interface built in, no installers.
  if (opts.targets.includes("desktop")) step("desktop app (tauri build --debug)", "npx", ["tauri", "build", "--debug", "--no-bundle"]);
}

// ---- processes ----------------------------------------------------------------------

/** A child process whose output is kept for the report (and shown when it fails). */
function start(command, args, env = {}) {
  const child = spawn(command, args, { cwd: root, env: { ...process.env, ...env }, stdio: ["ignore", "pipe", "pipe"] });
  const output = [];
  const keep = (chunk) => { output.push(String(chunk)); if (output.length > 400) output.shift(); };
  child.stdout.on("data", keep);
  child.stderr.on("data", keep);
  child.on("error", (error) => output.push(`spawn failed: ${error.message}`));
  return { child, output: () => output.join(""), stop: () => { if (child.exitCode === null) child.kill(); } };
}

async function waitFor(what, probe, timeout = 60_000) {
  const deadline = Date.now() + timeout;
  for (;;) {
    try { const value = await probe(); if (value) return value; } catch { /* not yet */ }
    if (Date.now() > deadline) throw new Error(`${what}: not ready after ${timeout / 1000} s`);
    await sleep(250);
  }
}

// ---- Chrome DevTools Protocol (Edge, Chrome, the WebView2 of the desktop app) -------

async function devtools(port, pick) {
  // What the port said last, for a timeout to report: nothing listening, or the pages it had.
  let seen = "nothing answered on the port";
  const targets = await waitFor("the webview's DevTools", async () => {
    try {
      const list = await (await fetch(`http://127.0.0.1:${port}/json/list`, { signal: AbortSignal.timeout(5000) })).json();
      seen = `pages: ${list.map((target) => `${target.type} ${target.url}`).join(", ") || "none"}`;
      return list.find((target) => target.type === "page" && pick(target.url));
    } catch (error) {
      seen = `nothing answered on the port (${error.cause?.code ?? error.message})`;
      throw error;
    }
  }).catch((error) => { throw new Error(`${error.message} — ${seen}`); });
  const socket = new WebSocket(targets.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error("DevTools did not open its connection within 10 s")), 10_000);
    socket.addEventListener("open", () => { clearTimeout(timer); resolve(); }, { once: true });
    socket.addEventListener("error", () => { clearTimeout(timer); reject(new Error("DevTools refused the connection")); }, { once: true });
  });
  let next = 0;
  const pending = new Map();
  socket.addEventListener("message", (message) => {
    const reply = JSON.parse(String(message.data));
    const waiter = reply.id !== undefined && pending.get(reply.id);
    if (!waiter) return;
    pending.delete(reply.id);
    if (reply.error) waiter.reject(new Error(`${waiter.method}: ${reply.error.message}`));
    else waiter.resolve(reply.result);
  });
  // A browser that goes away answers nothing more: say so to every call still waiting.
  socket.addEventListener("close", () => {
    for (const [id, waiter] of pending) { pending.delete(id); waiter.reject(new Error(`${waiter.method}: the browser closed its DevTools connection`)); }
  });
  const send = (method, params = {}) => new Promise((resolve, reject) => {
    const id = ++next;
    const timer = setTimeout(() => { pending.delete(id); reject(new Error(`${method}: no answer from the browser within ${CALL_TIMEOUT / 1000} s`)); }, CALL_TIMEOUT);
    pending.set(id, { resolve: (value) => { clearTimeout(timer); resolve(value); }, reject: (error) => { clearTimeout(timer); reject(error); }, method });
    socket.send(JSON.stringify({ id, method, params }));
  });
  await send("Page.enable");
  await send("Runtime.enable");
  return {
    async evaluate(expression) {
      const reply = await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
      if (reply.exceptionDetails) throw new Error(reply.exceptionDetails.exception?.description ?? reply.exceptionDetails.text);
      return reply.result.value;
    },
    async navigate(url) { await send("Page.navigate", { url }); },
    async screenshot() { return Buffer.from((await send("Page.captureScreenshot", { format: "png" })).data, "base64"); },
    async size(width, height) { await send("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 1, mobile: false }); },
    close() { socket.close(); },
  };
}

function findBrowser() {
  const candidates = process.platform === "win32"
    ? [`${process.env["ProgramFiles(x86)"]}\\Microsoft\\Edge\\Application\\msedge.exe`, `${process.env.ProgramFiles}\\Microsoft\\Edge\\Application\\msedge.exe`,
      `${process.env.ProgramFiles}\\Google\\Chrome\\Application\\chrome.exe`, `${process.env.LOCALAPPDATA}\\Google\\Chrome\\Application\\chrome.exe`]
    : ["/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser", "/usr/bin/microsoft-edge"];
  return candidates.find((path) => path && existsSync(path));
}

/**
 * Close a Chromium through DevTools. Edge and Chrome may hand their work to
 * another process at start and leave the one we started, so killing that
 * one can leave the browser running on its profile — and a browser already
 * on a profile takes over the next start there instead of starting afresh.
 */
async function closeBrowser(port) {
  try {
    const { webSocketDebuggerUrl } = await (await fetch(`http://127.0.0.1:${port}/json/version`, { signal: AbortSignal.timeout(3000) })).json();
    const socket = new WebSocket(webSocketDebuggerUrl);
    await new Promise((resolve) => {
      const timer = setTimeout(resolve, 3000);
      socket.addEventListener("open", () => socket.send(JSON.stringify({ id: 1, method: "Browser.close" })), { once: true });
      socket.addEventListener("close", () => { clearTimeout(timer); resolve(); }, { once: true });
      socket.addEventListener("error", () => { clearTimeout(timer); resolve(); }, { once: true });
    });
  } catch {
    // Already gone.
  }
}

/** A headless Chromium on a fresh profile of its own, so nothing of a person's browser — or an earlier run's — is touched. */
async function chromium(url, base) {
  const binary = findBrowser();
  if (!binary) throw new Error("no Edge or Chrome found");
  rmSync(base, { recursive: true, force: true });
  const profile = join(base, `${process.pid}-${Date.now().toString(36)}`);
  mkdirSync(profile, { recursive: true });
  const browser = start(binary, ["--headless=new", "--remote-debugging-port=0", `--user-data-dir=${profile}`, "--no-first-run", "--no-default-browser-check",
    "--window-size=1400,900", "--lang=en-US", "about:blank"]);
  const port = await waitFor("the browser", () => { const file = join(profile, "DevToolsActivePort"); return existsSync(file) && Number(readFileSync(file, "utf8").split("\n")[0]); }, 30_000);
  const page = await devtools(port, (address) => address === "about:blank");
  await page.size(1400, 900);
  await page.navigate(url);
  return { page, name: binary.split(/[\\/]/).pop().replace(/\.exe$/, ""), stop: async () => { page.close(); await closeBrowser(port); browser.stop(); } };
}

// ---- W3C WebDriver (WebKitWebDriver: WebKitGTK, the Linux desktop app's webview) -----

const MINIBROWSERS = ["/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/MiniBrowser", "/usr/lib/aarch64-linux-gnu/webkit2gtk-4.1/MiniBrowser", "/usr/libexec/webkit2gtk-4.1/MiniBrowser"];

async function webkit({ binary, args = [], url, env = {} }) {
  if (!process.env.DISPLAY && !process.env.WAYLAND_DISPLAY) throw new Error("no display — run under xvfb-run -a");
  const port = await freePort();
  // The app checks TAURI_WEBVIEW_AUTOMATION before it lets WebDriver drive its webview.
  const driver = start("WebKitWebDriver", [`--port=${port}`], env);
  const base = `http://127.0.0.1:${port}`;
  const call = async (method, path, body) => {
    const response = await fetch(base + path, { method, headers: { "content-type": "application/json" }, body: body === undefined ? undefined : JSON.stringify(body), signal: AbortSignal.timeout(CALL_TIMEOUT) });
    const reply = await response.json();
    if (!response.ok) throw new Error(`${method} ${path}: ${reply.value?.message ?? response.status}`);
    return reply.value;
  };
  await waitFor("WebKitWebDriver", () => call("GET", "/status"), 15_000);
  const { sessionId } = await call("POST", "/session", { capabilities: { alwaysMatch: { "webkitgtk:browserOptions": { binary, args } } } });
  const S = `/session/${sessionId}`;
  await call("POST", `${S}/window/rect`, { width: 1400, height: 900 }).catch(() => {});
  if (url) await call("POST", `${S}/url`, { url });
  const page = {
    evaluate: (expression) => call("POST", `${S}/execute/sync`, { script: `return (${expression});`, args: [] }),
    async inject(source) { return call("POST", `${S}/execute/sync`, { script: `${source}\n;return true;`, args: [] }); },
    async screenshot() { return Buffer.from(await call("GET", `${S}/screenshot`), "base64"); },
    close: () => call("DELETE", S).catch(() => {}),
  };
  return { page, name: "WebKitGTK", stop: async () => { await page.close(); driver.stop(); }, output: driver.output };
}

// ---- the targets -----------------------------------------------------------------------

async function launchServer(opts, work) {
  if (opts.serverUrl) return { url: opts.serverUrl, dataDir: opts.dataDir ?? "/data", stop: () => {}, output: () => "" };
  const port = await freePort();
  const dataDir = join(work, "data");
  const server = start(join(root, "target", "debug", `signal-lab-server${EXE}`),
    ["--listen", `127.0.0.1:${port}`, "--ui-dir", join(root, "dist"), "--data-dir", dataDir, "--log", "warn"]);
  const url = `http://127.0.0.1:${port}`;
  await waitFor("the server", async () => (await fetch(`${url}/api/health`, { signal: AbortSignal.timeout(5000) })).ok, 30_000).catch((error) => { throw new Error(`${error.message}\n${server.output()}`); });
  return { url: `${url}/`, dataDir, stop: server.stop, output: server.output };
}

async function openServer(opts, work) {
  const server = await launchServer(opts, work);
  const browser = process.platform === "linux"
    ? await webkit({ binary: MINIBROWSERS.find((path) => existsSync(path)) ?? fail("MiniBrowser not found"), args: ["--automation"], url: server.url })
    : await chromium(server.url, join(work, "profile"));
  return { page: browser.page, engine: browser.name, dataDir: server.dataDir, stop: async () => { await browser.stop(); server.stop(); }, output: server.output };
}

/** The whole screen, as a person at the machine would see it — a dialog in the way, say. Windows only. */
function screenshot(file) {
  const script = `Add-Type -AssemblyName System.Windows.Forms,System.Drawing; $b=[System.Windows.Forms.SystemInformation]::VirtualScreen; $i=New-Object System.Drawing.Bitmap $b.Width,$b.Height; $g=[System.Drawing.Graphics]::FromImage($i); $g.CopyFromScreen($b.Left,$b.Top,0,0,$i.Size); $i.Save('${file.replaceAll("'", "''")}')`;
  spawnSync("powershell", ["-NoProfile", "-NonInteractive", "-Command", script], { stdio: "ignore", timeout: 20_000 });
}

async function openDesktop(opts, work) {
  const dataDir = join(work, "data");
  const app = join(root, "target", "debug", `signal-lab${EXE}`);
  if (!existsSync(app)) throw new Error(`${app} is not built`);
  if (process.platform === "linux") {
    const browser = await webkit({ binary: app, env: { TAURI_WEBVIEW_AUTOMATION: "true", SIGNALLAB_DATA_DIR: dataDir } });
    return { page: browser.page, engine: "WebKitGTK (Tauri)", dataDir, stop: browser.stop, output: browser.output };
  }
  // WebView2 reads both from the environment: DevTools on a port, and a profile of its own.
  const port = await freePort();
  const profile = join(work, "webview2");
  rmSync(profile, { recursive: true, force: true });
  // The debug build opens DevTools on this port itself (src-tauri/src/lib.rs): WebView2 does
  // not take WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS everywhere (a CI runner ignored it).
  const desktop = start(app, [], { SIGNALLAB_E2E_DEVTOOLS_PORT: String(port), WEBVIEW2_USER_DATA_FOLDER: profile, SIGNALLAB_DATA_DIR: dataDir });
  const page = await devtools(port, (address) => /tauri\.localhost|^tauri:/.test(address)).catch((error) => {
    const state = desktop.child.exitCode === null ? "the app is still running" : `the app exited with ${desktop.child.exitCode}`;
    // What the webview's processes were started with: whether the DevTools port reached them.
    const webviews = spawnSync("powershell", ["-NoProfile", "-NonInteractive", "-Command",
      "Get-CimInstance Win32_Process -Filter \"Name='msedgewebview2.exe'\" | Where-Object { $_.CommandLine -notmatch '--type=' } | ForEach-Object { $_.CommandLine }"],
      { encoding: "utf8", timeout: 20_000 }).stdout?.trim();
    // A window that is there but not answering: the screen shows why (a dialog, an error page).
    if (opts.out) screenshot(join(opts.out, "screen.png"));
    desktop.stop();
    throw new Error(`${error.message}; ${state}\nwebview: ${webviews || "no msedgewebview2 browser process"}\n${desktop.output()}`);
  });
  return { page, engine: "WebView2 (Tauri)", dataDir, stop: () => { page.close(); desktop.stop(); }, output: desktop.output };
}

// ---- the tour ---------------------------------------------------------------------------

/** The steps in order, with what each needs and what the runner checks at the far end. */
function plan(ports, fixtures, mode, dataDir) {
  const { counts } = fixtures;
  const at = {};
  const mark = () => { at.udp = counts.udp; at.http = counts.http; at.tcp = counts.tcpBytes; at.pings = counts.pings; at.polls = counts.statusPolls; };
  return [
    { name: "shell", args: { mode, version: currentVersion() } },
    { name: "inspectArm" },
    { name: "osc", args: { port: ports.osc } },
    { name: "signals", args: { port: ports.osc } },
    { name: "oscStop" },
    { name: "mqtt", args: { port: ports.mqtt }, after: (expect) => {
      expect("the broker received the publish and the clear", counts.mqttPublishes >= 2 && counts.mqttTopics.has("e2e/lights/hall"), `${counts.mqttPublishes} publishes`);
      expect("the broker saw both subscriptions", counts.mqttSubscriptions.includes("#") && counts.mqttSubscriptions.includes("e2e/+/hall"), counts.mqttSubscriptions.join(", "));
    } },
    { name: "broadcast", args: { port: ports.discovery } },
    { name: "netsimStart", args: { relay: ports.relay, sink: ports.sink }, after: async () => { mark(); await fixtures.sendUdp(ports.relay, 20); } },
    { name: "netsimCheck", args: { n: 20 }, after: (expect) => expect("all 20 arrived at the relay's target", counts.udp - at.udp === 20, `${counts.udp - at.udp} arrived`) },
    { name: "storm", args: { sink: ports.sink, tcp: ports.tcp }, before: mark, after: async (expect, data) => {
      await sleep(300);
      expect("the UDP sink received what the storm counted", counts.udp - at.udp >= data.udp * 0.95, `${counts.udp - at.udp} of ${data.udp}`);
      expect("the TCP sink received bytes", counts.tcpBytes - at.tcp > 0, `${counts.tcpBytes - at.tcp} B`);
    } },
    { name: "scan", args: { port: ports.http } },
    { name: "http", args: { port: ports.http }, before: mark, after: (expect) => expect("the API answered every request (GET, POST, 40 in the burst)", counts.http - at.http >= 42, `${counts.http - at.http} requests`) },
    { name: "library", args: { port: ports.http }, after: async (expect) => {
      // On this machine the library file is right here: the folders it ended with, and no test signal left.
      const file = join(dataDir, "signals.json");
      if (!existsSync(file)) return;
      const read = () => JSON.parse(readFileSync(file, "utf8"));
      // The interface writes the file shortly after the last change.
      await waitFor("the library file", () => (read().folders ?? []).includes("API/Old"), 5000).catch(() => {});
      const saved = read();
      expect("the library file is version 2 and the test signal is gone", saved.version === 2 && !saved.signals.some((signal) => signal.name === "E2E check"), JSON.stringify(saved.folders));
      expect("…and keeps the folder that is now empty", (saved.folders ?? []).includes("API/Old"), JSON.stringify(saved.folders));
    } },
    { name: "emulators", args: { port: ports.emulator } },
    { name: "emulatorMqtt", args: { port: ports.broker } },
    { name: "experimentHttp", args: { port: ports.http } },
    { name: "experimentOsc", args: { device: ports.device, pong: ports.pong }, before: mark, after: (expect) => expect("the device was pinged", counts.pings - at.pings >= 1) },
    { name: "experimentRepeat", args: { device: ports.device, pong: ports.pong }, before: mark, after: (expect) => expect("the device got three pings", counts.pings - at.pings === 3, `${counts.pings - at.pings} pings`) },
    { name: "experimentLoop", args: { device: ports.device }, before: mark, after: (expect) => expect("the device was polled three times", counts.statusPolls - at.polls === 3, `${counts.statusPolls - at.polls} polls`) },
    { name: "experimentEmulator", args: { port: ports.emulatorRun } },
    // The layout step works on the parallel flows this one leaves open.
    { name: "experimentParallel", args: { port: ports.http } },
    { name: "experimentExport", args: { mode, dataDir } },
    { name: "layout" },
    { name: "inspectCheck", args: { mode, dataDir } },
    { name: "russian" },
    { name: "cleanup" },
  ];
}

async function tour(target, opts, source) {
  const platform = process.platform === "win32" ? "windows" : process.platform;
  const label = `${platform}-${target}`;
  const out = join(OUT, label);
  const work = join(BUILD, label);
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });
  mkdirSync(work, { recursive: true });
  rmSync(join(work, "data"), { recursive: true, force: true });

  const pong = await freePort("udp");
  const fixtures = await startFixtures({ pongPort: pong });
  const ports = {
    ...fixtures.ports, osc: await freePort("udp"), discovery: await freePort("udp"), relay: await freePort("udp"),
    // The emulators the tour makes listen on TCP ports of their own.
    emulator: await freePort(), emulatorRun: await freePort(), broker: await freePort(),
  };
  const started = Date.now();
  console.log(`\n▶ ${label}`);
  let app;
  const lines = [];
  let failed = 0;
  let passed = 0;
  const record = (step, check) => {
    if (check.ok) passed += 1; else failed += 1;
    const line = `${check.ok ? "✔" : "✖"} ${step}: ${check.name}${check.detail ? ` — ${check.detail}` : ""}`;
    lines.push(line);
    if (!check.ok) console.log(`  ${line}`);
  };
  try {
    app = target === "server" ? await openServer(opts, work) : await openDesktop({ ...opts, out }, work);
    console.log(`  ${app.engine}`);
    await waitFor("the interface", () => app.page.evaluate("!!document.querySelector('.sidebar .nav-item')"), 30_000);
    if (app.page.inject) await app.page.inject(source);
    else await app.page.evaluate(`${source}\n;true`);
    const steps = plan(ports, fixtures, target, opts.dataDir ?? app.dataDir);
    const chosen = opts.steps ? steps.filter((step) => step.name === "shell" || opts.steps.includes(step.name)) : steps;
    for (const [index, step] of chosen.entries()) {
      const at = Date.now();
      const expect = (name, ok, detail) => record(step.name, { name, ok, detail: ok ? undefined : detail });
      await step.before?.();
      const id = await app.page.evaluate(`window.__signalLabTour.start(${JSON.stringify(step.name)}, ${JSON.stringify(step.args ?? {})})`);
      let state;
      for (;;) {
        state = await app.page.evaluate(`window.__signalLabTour.poll(${id})`);
        if (state.state !== "running") break;
        if (Date.now() - at > STEP_TIMEOUT) { state = { ...state, state: "failed", error: `still running after ${STEP_TIMEOUT / 1000} s` }; break; }
        await sleep(150);
      }
      for (const check of state.checks) record(step.name, check);
      if (state.state === "failed") record(step.name, { name: "the step completes", ok: false, detail: state.error });
      else await step.after?.(expect, state.data ?? {});
      for (const error of await app.page.evaluate("window.__signalLabTour.errors()")) record(step.name, { name: "no error on the page", ok: false, detail: error });
      writeFileSync(join(out, `${String(index + 1).padStart(2, "0")}-${step.name}.png`), await app.page.screenshot());
      console.log(`  ${state.state === "failed" ? "✖" : "✔"} ${step.name} · ${state.checks.length} checks · ${seconds(at)}`);
    }
  } catch (error) {
    record("setup", { name: "the app starts and takes the tour", ok: false, detail: error.message });
  } finally {
    await app?.stop();
    await fixtures.close();
  }
  const log = app?.output?.() ?? "";
  const panicked = /panicked/.test(log);
  if (panicked) record("engine", { name: "the engine never panicked", ok: false, detail: log.split("\n").find((line) => line.includes("panicked")) });
  const report = [`# ${label} · ${app?.engine ?? "?"}`, "", `${passed} passed, ${failed} failed · ${seconds(started)}`, "", ...lines, "", "## Log", "", "```", log.trim().slice(-8000), "```", ""].join("\n");
  writeFileSync(join(out, "report.md"), report);
  console.log(`${failed ? "✖" : "✔"} ${label}: ${passed} passed, ${failed} failed · ${seconds(started)} · ${out}`);
  return failed === 0;
}

// ---- Linux, from any machine with Docker ---------------------------------------------------

const LINUX_IMAGE = "signallab-linux-e2e";
const VOLUMES = {
  "signallab-linux-node-modules": "/work/node_modules",
  "signallab-linux-target": "/work/target",
  "signallab-linux-cargo-registry": "/usr/local/cargo/registry",
  "signallab-linux-npm-cache": "/root/.npm",
};

function linux(opts, argv) {
  if (!run("docker", ["info"], { capture: true }).ok) fail("Docker is not running — start Docker Desktop first");
  const text = readFileSync(join(root, "rust-toolchain.toml"), "utf8");
  const channel = /^channel\s*=\s*"([^"]+)"/m.exec(text)?.[1] ?? "stable";
  const components = [...(/^components\s*=\s*\[([^\]]*)\]/m.exec(text)?.[1] ?? "").matchAll(/"([^"]+)"/g)].map((found) => found[1]).join(" ");
  console.log(`▶ image ${LINUX_IMAGE} (the Linux builder with WebKitWebDriver; cached after the first build)`);
  if (!run("docker", ["build", "--target", "e2e", "-t", LINUX_IMAGE, "--build-arg", `RUST_TOOLCHAIN=${channel}`, "--build-arg", `RUST_COMPONENTS=${components}`, join(root, "docker", "linux-builder")]).ok) {
    fail("could not build the Linux e2e image");
  }
  const forward = argv.filter((arg, index) => !["--linux", "--image"].includes(arg) && argv[index - 1] !== "--image");
  const mounts = [`${root}:/work`, ...Object.entries(VOLUMES).map(([volume, path]) => `${volume}:${path}`)].flatMap((mount) => ["-v", mount]);
  const inner = (args) => `npm ci --no-audit --no-fund --loglevel=error && xvfb-run -a node scripts/e2e.mjs --native ${args.join(" ")}`;
  if (!opts.image) {
    const result = run("docker", ["run", "--rm", "--init", ...mounts, "-w", "/work", LINUX_IMAGE, "bash", "-lc", inner(forward)]);
    if (!result.ok) process.exitCode = 1;
    return;
  }
  // The server image itself, sharing the tour's network namespace so its loopback is the fixtures' loopback.
  const box = `signallab-e2e-${process.pid}`;
  const server = `${box}-server`;
  try {
    if (!run("docker", ["run", "-d", "--init", "--name", box, ...mounts, "-w", "/work", LINUX_IMAGE, "sleep", "infinity"], { capture: true }).ok) fail("could not start the e2e container");
    if (!run("docker", ["run", "-d", "--name", server, "--network", `container:${box}`, "--read-only", "--cap-drop", "ALL", "--security-opt", "no-new-privileges",
      "-e", "SIGNALLAB_LISTEN=127.0.0.1:1430", opts.image], { capture: true }).ok) fail(`could not start ${opts.image}`);
    const desktop = forward.includes("--target") ? [] : ["--target", "server"];
    const result = run("docker", ["exec", box, "bash", "-lc", inner([...forward, ...desktop, "--server-url", "http://127.0.0.1:1430", "--data-dir", "/data"])]);
    const logs = run("docker", ["logs", server], { capture: true });
    if (/panicked/.test(logs.stdout + logs.stderr)) { console.error(`✖ ${opts.image} panicked:\n${logs.stderr}`); process.exitCode = 1; }
    if (!result.ok) process.exitCode = 1;
  } finally {
    run("docker", ["rm", "--force", server], { capture: true });
    run("docker", ["rm", "--force", box], { capture: true });
  }
}

async function main() {
  const argv = process.argv.slice(2);
  const opts = options(argv);
  if (opts.linux) return linux(opts, argv);
  // Whatever hangs in the end — a browser that never answers — the run ends and says so.
  setTimeout(() => { console.error("✖ the tour did not finish within 20 minutes"); process.exit(1); }, 20 * 60_000).unref();
  const started = Date.now();
  const source = await bundleTour();
  if (opts.build) build(opts);
  const results = [];
  for (const target of opts.targets) results.push(await tour(target, opts, source));
  console.log(`\n${results.every(Boolean) ? "✔ every tour passed" : "✖ a tour failed"} · ${seconds(started)} · reports in ${OUT}`);
  if (!results.every(Boolean)) process.exitCode = 1;
}

// No process.exit after network calls: on Windows it can abort Node while sockets close.
if (isMain(import.meta.url)) main().catch((error) => { console.error(`\n✖ ${error.message ?? error}`); process.exitCode = 1; });
