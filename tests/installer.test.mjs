import test from "node:test";
import assert from "node:assert/strict";
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
