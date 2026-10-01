#!/usr/bin/env node
// Tooltips in WebKitGTK — the webview of the Linux desktop app — with real
// pointer and key events, through WebKitWebDriver.
//
//   npm run check:webkit                 any machine with Docker (Ubuntu 22.04's WebKitGTK)
//   node scripts/webkit.mjs --native     Linux with WebKitWebDriver and a display (CI: xvfb-run -a)
//
// The page, tests/webkit/harness.tsx, holds the real TooltipLayer and
// stylesheet with every kind of element a tooltip sits on: an enabled and a
// disabled button, a disabled fieldset, labels and their fields, a modal dialog.
// A screenshot is left in target/webkit-harness/webkitgtk.png.

import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join } from "node:path";
import { fail, isMain, root, run, seconds } from "./lib.mjs";

const OUT = join(root, "target", "webkit-harness");
const IMAGE = "signallab-webkit";
const ELEMENT = "element-6066-11e4-a52e-4f735466cecf";
const BROWSERS = ["/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/MiniBrowser", "/usr/lib/aarch64-linux-gnu/webkit2gtk-4.1/MiniBrowser", "/usr/libexec/webkit2gtk-4.1/MiniBrowser"];
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function bundle() {
  const esbuild = await import("esbuild");
  mkdirSync(OUT, { recursive: true });
  await esbuild.build({
    entryPoints: [join(root, "tests", "webkit", "harness.tsx")],
    bundle: true, outdir: OUT, format: "esm", jsx: "automatic", minify: true, logLevel: "warning",
    loader: { ".css": "css" }, define: { "process.env.NODE_ENV": '"production"' },
  });
  copyFileSync(join(root, "tests", "webkit", "index.html"), join(OUT, "index.html"));
}

/** The harness on a loopback port. */
function serve() {
  const types = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".css": "text/css" };
  const server = createServer((request, response) => {
    const name = request.url === "/" ? "index.html" : request.url.slice(1).split("?")[0];
    const file = join(OUT, name);
    if (name.includes("..") || !existsSync(file)) { response.writeHead(404).end(); return; }
    response.writeHead(200, { "content-type": types[extname(file)] ?? "application/octet-stream" }).end(readFileSync(file));
  });
  return new Promise((resolve) => server.listen(0, "127.0.0.1", () => resolve({ server, port: server.address().port })));
}

/** A minimal W3C WebDriver client. */
function driver(base) {
  const call = async (method, path, body) => {
    const response = await fetch(base + path, { method, headers: { "content-type": "application/json" }, body: body === undefined ? undefined : JSON.stringify(body) });
    const reply = await response.json();
    if (!response.ok) throw new Error(`${method} ${path}: ${reply.value?.message ?? response.status}`);
    return reply.value;
  };
  return { call };
}

async function check() {
  const browser = BROWSERS.find((path) => existsSync(path));
  if (!browser) fail("MiniBrowser not found — install libwebkit2gtk-4.1-0");
  if (!process.env.DISPLAY && !process.env.WAYLAND_DISPLAY) fail("no display — run under xvfb-run -a");
  const { server, port } = await serve();
  const webdriverPort = 4400 + Math.floor(Math.random() * 500);
  const webdriver = spawn("WebKitWebDriver", [`--port=${webdriverPort}`], { stdio: "ignore" });
  webdriver.on("error", () => fail("WebKitWebDriver not found — install webkit2gtk-driver"));
  const { call } = driver(`http://127.0.0.1:${webdriverPort}`);
  for (let i = 0; ; i++) {
    try { await call("GET", "/status"); break; } catch { if (i > 50) fail("WebKitWebDriver did not start"); await sleep(100); }
  }
  const { sessionId } = await call("POST", "/session", { capabilities: { alwaysMatch: { "webkitgtk:browserOptions": { binary: browser, args: ["--automation"] } } } });
  const S = `/session/${sessionId}`;
  const js = (script) => call("POST", `${S}/execute/sync`, { script, args: [] });
  const element = (css) => call("POST", `${S}/element`, { using: "css selector", value: css });
  const pointer = (...actions) => call("POST", `${S}/actions`, { actions: [{ type: "pointer", id: "mouse", parameters: { pointerType: "mouse" }, actions }] });
  const press = (value) => call("POST", `${S}/actions`, { actions: [{ type: "key", id: "keyboard", actions: [{ type: "keyDown", value }, { type: "keyUp", value }] }] });
  const tip = () => js("const t = document.getElementById('signal-lab-tooltip'); return t ? t.textContent : null;");
  const away = () => pointer({ type: "pointerMove", origin: "viewport", x: 5, y: 5, duration: 50 }, { type: "pause", duration: 700 });
  const hover = async (css) => { await away(); await pointer({ type: "pointerMove", origin: await element(css), x: 0, y: 0, duration: 150 }, { type: "pause", duration: 900 }); return tip(); };

  const results = [];
  const expect = (name, actual, expected) => {
    const ok = JSON.stringify(actual) === JSON.stringify(expected);
    results.push(ok);
    console.log(`${ok ? "✔" : "✖"} ${name}${ok ? "" : ` — got ${JSON.stringify(actual)}, expected ${JSON.stringify(expected)}`}`);
  };
  try {
    await call("POST", `${S}/url`, { url: `http://127.0.0.1:${port}/` });
    await sleep(1500);
    const engine = await js("return { ua: navigator.userAgent, popover: typeof HTMLElement.prototype.showPopover === 'function', focusVisible: CSS.supports('selector(:focus-visible)') };");
    console.log(`  ${engine.ua}`);
    expect("the popover top layer is available", engine.popover, true);
    expect(":focus-visible is supported", engine.focusVisible, true);
    expect("hovering a button", await hover("#enabled"), "Отправить запрос и сохранить ответ · Ctrl+Enter");
    expect("hovering a disabled button", await hover("#disabled"), "Нет полной копии кадра — повторить байт в байт нельзя");
    expect("hovering a button in a disabled fieldset", await hover("#inFieldset"), "Кнопка в отключённом fieldset");
    expect("hovering a field's label", await hover("#label"), "0 — без ограничения");
    expect("hovering inside a labelled field shows nothing", await hover("#wrapped"), null);
    await away();
    await js("document.activeElement?.blur(); return null;");
    await press("");
    await sleep(400);
    expect("Tab onto a control shows its tooltip at once", [await js("return document.activeElement.id;"), await tip()], ["enabled", "Отправить запрос и сохранить ответ · Ctrl+Enter"]);
    await press("");
    await sleep(300);
    expect("Escape hides it", await tip(), null);
    await call("POST", `${S}/element/${(await element("#open"))[ELEMENT]}/click`, {});
    await sleep(500);
    expect("hovering inside a modal dialog", await hover("#inDialog"), "Поверх модального окна");
    expect("…drawn above the dialog (top layer)", await js("return document.getElementById('signal-lab-tooltip')?.matches(':popover-open') ?? null;"), true);
    writeFileSync(join(OUT, "webkitgtk.png"), Buffer.from(await call("GET", `${S}/screenshot`), "base64"));
  } finally {
    await call("DELETE", S).catch(() => {});
    webdriver.kill();
    server.close();
  }
  if (results.includes(false)) fail("tooltips misbehave in WebKitGTK");
  console.log(`\n✔ tooltips work in WebKitGTK · screenshot: ${join(OUT, "webkitgtk.png")}`);
}

async function main() {
  const started = Date.now();
  const native = process.argv.includes("--native");
  if (!process.argv.includes("--prebuilt")) await bundle();
  if (native) return check();
  if (!run("docker", ["info"], { capture: true }).ok) fail("Docker is not running — start Docker Desktop first");
  console.log(`▶ image ${IMAGE} (Ubuntu 22.04's WebKitGTK; cached after the first build)`);
  if (!run("docker", ["build", "-t", IMAGE, join(root, "docker", "webkit")]).ok) fail("could not build the WebKitGTK image");
  // --init: as PID 1, xvfb-run never receives the X server's "ready" signal and waits forever.
  const result = run("docker", ["run", "--rm", "--init", "-v", `${root}:/work`, "-w", "/work", IMAGE, "xvfb-run", "-a", "node", "scripts/webkit.mjs", "--native", "--prebuilt"]);
  if (!result.ok) fail(`the WebKitGTK check failed after ${seconds(started)}`);
  console.log(`· ${seconds(started)}`);
}

if (isMain(import.meta.url)) main().catch((error) => fail(error.message ?? String(error)));
