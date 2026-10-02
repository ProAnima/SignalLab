import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
const windows = conf.bundle.windows;

/** Width, height and bits per pixel of a BMP file, from its header. */
function bmp(path) {
  const bytes = readFileSync(path);
  assert.equal(bytes.toString("latin1", 0, 2), "BM", `${path} is a BMP`);
  return { width: bytes.readInt32LE(18), height: Math.abs(bytes.readInt32LE(22)), bits: bytes.readUInt16LE(28) };
}

test("the installers' artwork exists at the sizes NSIS and WiX draw it", () => {
  const expected = [
    [windows.nsis.sidebarImage, 164, 314],
    [windows.nsis.headerImage, 150, 57],
    [windows.nsis.uninstallerHeaderImage, 150, 57],
    [windows.wix.dialogImagePath, 493, 312],
    [windows.wix.bannerPath, 493, 58],
  ];
  for (const [file, width, height] of expected) {
    assert.ok(file, "every image is configured");
    const path = join("src-tauri", file);
    assert.ok(existsSync(path), `${path} exists (python scripts/gen-installer-art.py)`);
    assert.deepEqual(bmp(path), { width, height, bits: 24 }, path);
  }
  assert.ok(existsSync(join("src-tauri", windows.nsis.installerIcon)), "the installer has the app's icon");
});

test("installing takes few steps and both languages", () => {
  assert.equal(conf.bundle.licenseFile, undefined, "MIT asks for no acceptance, so there is no license page to click through");
  assert.deepEqual(windows.nsis.languages, ["English", "Russian"]);
  assert.equal(windows.nsis.displayLanguageSelector, true);
  assert.equal(windows.nsis.installMode, "both", "per user without admin rights, or for every user of a show PC");
});

test("the command line comes with the app: in the setup, the MSI and the Linux packages", () => {
  assert.equal(conf.build.beforeBundleCommand, "node scripts/cli-bundle.mjs", "signallab is built before the installers are");
  const hooks = readFileSync(join("src-tauri", windows.nsis.installerHooks), "utf8");
  for (const piece of ["NSIS_HOOK_POSTINSTALL", "NSIS_HOOK_PREUNINSTALL", "signallab.exe", "path.ps1", "/NOPATH", "/NOFIREWALL", "profile=private,domain"]) {
    assert.ok(hooks.includes(piece), `hooks.nsh: ${piece}`);
  }
  assert.ok(existsSync("src-tauri/installer/path.ps1"));
  assert.deepEqual(windows.wix.fragmentPaths, ["installer/cli.wxs"]);
  const fragment = readFileSync("src-tauri/installer/cli.wxs", "utf8");
  for (const id of windows.wix.componentRefs) assert.ok(fragment.includes(`Component Id="${id}"`), id);
  assert.ok(fragment.includes('Name="PATH"') && fragment.includes('Permanent="no"'), "the MSI puts the folder on PATH and takes it off again");
  for (const format of ["deb", "rpm"]) assert.equal(conf.bundle.linux[format].files["/usr/bin/signallab"], "../target/release/signallab", format);
});

test("PATH: the folder goes on once, comes off cleanly, and nothing else on it changes", { skip: process.platform !== "win32" && "Windows only" }, () => {
  const key = `Software\\SignalLabPathTest-${process.pid}`;
  const ps = (script) => spawnSync("powershell.exe", ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script], { encoding: "utf8" });
  const value = () => ps(`$k = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('${key}'); $k.GetValue('Path', '', 'DoNotExpandEnvironmentNames') + '|' + $k.GetValueKind('Path')`).stdout.trim();
  const path = (flag, folder) => ps(`& 'src-tauri\\installer\\path.ps1' ${flag} '${folder}' -Key '${key}'`);
  try {
    ps(`[Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('${key}').SetValue('Path', '%USERPROFILE%\\bin;C:\\Tools', 'ExpandString')`);
    assert.equal(path("-Add", "C:\\Program Files\\Signal Lab").status, 0);
    assert.equal(path("-Add", "C:\\Program Files\\Signal Lab\\").status, 0);
    assert.equal(value(), "%USERPROFILE%\\bin;C:\\Tools;C:\\Program Files\\Signal Lab|ExpandString", "once, with %USERPROFILE% kept as it was");
    assert.equal(path("-Remove", "c:\\program files\\signal lab").status, 0);
    assert.equal(value(), "%USERPROFILE%\\bin;C:\\Tools|ExpandString");
  } finally {
    ps(`[Microsoft.Win32.Registry]::CurrentUser.DeleteSubKeyTree('${key}', $false)`);
  }
});
