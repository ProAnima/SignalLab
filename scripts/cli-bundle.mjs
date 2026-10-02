#!/usr/bin/env node
// The command line `signallab`, built for the desktop installers before Tauri
// bundles them (tauri.conf.json → build.beforeBundleCommand). The Windows
// setup and the MSI take target/release/signallab.exe
// (src-tauri/installer/hooks.nsh, cli.wxs); the .deb and .rpm take
// target/release/signallab into /usr/bin (bundle.linux.*.files).
//
// Always a release build, whatever the app's profile: it is what people run.

import { existsSync } from "node:fs";
import { join } from "node:path";
import { fail, root, run, seconds } from "./lib.mjs";

const started = Date.now();
const built = run("cargo", ["build", "--release", "--locked", "-p", "signal-lab-cli"]);
if (!built.ok) fail("could not build signallab for the installers");
const binary = join(root, "target", "release", process.platform === "win32" ? "signallab.exe" : "signallab");
if (!existsSync(binary)) fail(`${binary} is missing after the build`);
console.log(`✔ signallab for the installers · ${seconds(started)}`);
