import test from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { placeTip, TIP_GAP, TIP_MARGIN } from "../src/lib/tooltip.ts";

const view = { width: 1200, height: 800 };
const tip = { width: 200, height: 40 };

test("a tooltip sits above its element, centred, and below only when above has no room", () => {
  const button = { left: 500, top: 300, width: 100, height: 30 };
  assert.deepEqual(placeTip(button, tip, view), { left: 450, top: 300 - TIP_GAP - 40, side: "above" });
  const toolbar = { left: 500, top: 10, width: 100, height: 30 };
  assert.deepEqual(placeTip(toolbar, tip, view), { left: 450, top: 40 + TIP_GAP, side: "below" });
});

test("a tooltip stays inside the window", () => {
  const atLeft = placeTip({ left: 0, top: 400, width: 20, height: 20 }, tip, view);
  assert.equal(atLeft.left, TIP_MARGIN);
  const atRight = placeTip({ left: 1190, top: 400, width: 10, height: 20 }, tip, view);
  assert.equal(atRight.left, view.width - TIP_MARGIN - tip.width);
  // Taller than the space on either side: the larger side, clamped to the window.
  const tall = placeTip({ left: 500, top: 380, width: 10, height: 20 }, { width: 200, height: 700 }, view);
  assert.deepEqual([tall.side, tall.top], ["below", view.height - TIP_MARGIN - 700]);
  assert.equal(placeTip({ left: 5, top: 400, width: 10, height: 10 }, { width: 2000, height: 40 }, view).left, TIP_MARGIN, "wider than the window");
});

/** Every .tsx file of the interface. */
function sources(dir = fileURLToPath(new URL("../src/", import.meta.url))) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? sources(path) : path.endsWith(".tsx") ? [path] : [];
  });
}

test("help is a tooltip: no native title attributes and no caption paragraphs in the interface", () => {
  const offenders = [];
  const files = sources();
  assert.ok(files.length > 20, "the interface was scanned");
  for (const file of files) {
    const text = readFileSync(file, "utf8");
    if (/\stitle=[{"]/.test(text)) offenders.push(`${file}: title= (use data-tip)`);
    for (const caption of ["experiment-hint", "field-hint", "palette-hint", "experiment-menu-hint", "experiment-documents-note", "hint info"]) {
      if (text.includes(`"${caption}"`) || text.includes(`"${caption} `)) offenders.push(`${file}: ${caption}`);
    }
    if (/<h1[^>]*>[^<]*<\/h1>\s*<p>/.test(text)) offenders.push(`${file}: a paragraph under the view title`);
  }
  assert.deepEqual(offenders, []);
});

/** The `{…}` expression of every data-tip attribute, braces balanced. */
function tipExpressions(text) {
  const found = [];
  for (let at = text.indexOf("data-tip={"); at >= 0; at = text.indexOf("data-tip={", at + 1)) {
    let depth = 0;
    for (let i = at + "data-tip=".length; i < text.length; i++) {
      if (text[i] === "{") depth++;
      else if (text[i] === "}" && --depth === 0) { found.push(text.slice(at, i + 1)); break; }
    }
  }
  return found;
}

test("every tooltip has a text of its own in every language", async () => {
  const { en } = await import("../src/lib/locales/en.ts");
  const { LOCALES } = await import("../src/lib/locales/index.ts");
  const keys = new Set();
  for (const file of sources()) {
    for (const expression of tipExpressions(readFileSync(file, "utf8"))) {
      for (const [, key] of expression.matchAll(/"([a-z]+\.[A-Za-z.]+)"/g)) if (key in en) keys.add(key);
      // A key built from a prefix, e.g. t(`bc.blurb.${mode}`): every key under it.
      for (const [, prefix] of expression.matchAll(/t\(`([a-z]+\.[A-Za-z.]+\.)\$\{/g)) Object.keys(en).filter((key) => key.startsWith(prefix)).forEach((key) => keys.add(key));
    }
  }
  // Tooltips whose key comes through a helper rather than the attribute itself.
  for (const key of Object.keys(en)) {
    if (/^exp\.description\.|^exp\.template.*Hint$|^exp\.(fork|join)Hint$|^exp\.missing(Values|Secrets)Hint$/.test(key)) keys.add(key);
  }
  assert.ok(keys.size > 60, `found ${keys.size} tooltip texts`);
  // A key combination reads the same in every language.
  const SAME_EVERYWHERE = new Set(["sig.fireHint"]);
  // A language in a script of its own writes its tooltips in it; one in Latin letters, not as English does.
  const SCRIPT = { ru: /[а-яё]/i, zh: /\p{Script=Han}/u, ja: /[\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Han}]/u, ko: /\p{Script=Hangul}/u, hi: /\p{Script=Devanagari}/u, ar: /\p{Script=Arabic}/u };
  // {name} is filled in; {{name}} is template syntax shown as it is.
  const placeholders = (text) => [...text.matchAll(/(?<!\{)\{(\w+)\}(?!\})/g)].map((found) => found[1]).sort().join();
  for (const { code, dict } of LOCALES) {
    if (code === "en") continue;
    // Units are notation: `ms` reads the same in most languages.
    const untranslated = [...keys].filter((key) => !SAME_EVERYWHERE.has(key) && !key.startsWith("unit.") && (!dict[key] || dict[key] === en[key] || (SCRIPT[code] && !SCRIPT[code].test(dict[key]))));
    assert.deepEqual(untranslated, [], `${code}: a tooltip is in the language`);
    assert.deepEqual([...keys].filter((key) => placeholders(en[key]) !== placeholders(dict[key])), [], `${code}: the same values as English`);
  }
});

test("no tooltip text is written into the markup in one language", () => {
  const ALLOWED = new Set(["Ctrl", "Shift", "Enter", "Delete", "Esc"]);
  const literal = [];
  for (const file of sources()) {
    const text = readFileSync(file, "utf8");
    if (/data-tip="/.test(text)) literal.push(`${file}: data-tip="…"`);
    for (const expression of tipExpressions(text)) {
      // Text outside t(…): string literals and the fixed parts of template literals.
      const rest = expression.replace(/t\((?:[^()]|\((?:[^()]|\([^()]*\))*\))*\)/g, "");
      const pieces = [...rest.matchAll(/"([^"]*)"|`([^`]*)`/g)].map((found) => (found[1] ?? found[2]).replace(/\$\{[^}]*\}/g, " "));
      // Words a person reads: capitalised or Cyrillic; lower-case identifiers ("list") are code.
      const words = pieces.join(" ").match(/[A-Za-zА-Яа-яЁё]{2,}/g) ?? [];
      if (words.some((word) => !ALLOWED.has(word) && /^[A-ZА-ЯЁ]|[а-яё]/.test(word))) literal.push(`${file}: ${expression}`);
    }
  }
  assert.deepEqual(literal, []);
});
