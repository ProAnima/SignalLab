import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { format, formatNumber, pickLanguage, placeholders, plurals } from "../src/lib/translate.ts";
import { LOCALES, SOURCE } from "../src/lib/locales/index.ts";
import { en } from "../src/lib/locales/en.ts";
import { SEED_IDS, localizeSeed } from "../src/lib/library.ts";
import { fmtBytes, fmtNum, setFormatLanguage } from "../src/lib/format.ts";

const FILES = "{n, plural, one {# файл} few {# файла} many {# файлов} other {# файла}}";

test("placeholders: values as given, plurals by the language's rules", () => {
  assert.equal(format("to {target}:{port}", { target: "127.0.0.1", port: 9000 }, "ru"), "to 127.0.0.1:9000", "a port is not grouped");
  assert.equal(format("{n, plural, one {# signal} other {# signals}}", { n: 1 }, "en"), "1 signal");
  assert.equal(format("{n, plural, one {# signal} other {# signals}}", { n: 2 }, "en"), "2 signals");
  assert.equal(format("{n, plural, one {# signal} other {# signals}}", { n: 0 }, "en"), "0 signals");
  for (const [n, text] of [[1, "1 файл"], [2, "2 файла"], [5, "5 файлов"], [11, "11 файлов"], [21, "21 файл"], [22, "22 файла"], [1.5, "1,5 файла"]]) {
    assert.equal(format(FILES, { n }, "ru"), text, String(n));
  }
  assert.equal(format(FILES, { n: 12345 }, "ru").replace(/\s/g, " "), "12 345 файлов", "# is the number as the language writes it");
  assert.equal(format("{n, plural, =0 {nothing} one {# item} other {# items}}", { n: 0 }, "en"), "nothing", "an exact value first");
  assert.equal(format("{n, plural, one {# of {total}} other {# of {total}}}", { n: 3, total: 7 }, "en"), "3 of 7", "values inside a branch");
  assert.equal(format("{n, plural, one {# frame} other {# frames}}", { n: "1" }, "en"), "1 frame", "engine values arrive as strings");
  assert.equal(format("{n, plural, one {# frame} other {# frames}}", { n: "many" }, "en"), "many", "a value that is not a number shows as it is");
});

test("select: a branch by the value, other when none matches", () => {
  const text = "{topic}{retain, select, true { · retain} other {}}";
  assert.equal(format(text, { topic: "a/b", retain: "true" }, "en"), "a/b · retain");
  assert.equal(format(text, { topic: "a/b", retain: "false" }, "en"), "a/b");
  assert.equal(format("{n, plural, one {# {kind, select, udp {datagram} other {packet}}} other {# {kind, select, udp {datagrams} other {packets}}}}", { n: 2, kind: "udp" }, "en"), "2 datagrams", "nested in a plural");
});

test("anything else in braces is text", () => {
  assert.equal(format("use {{name}} here", { other: 1 }, "en"), "use {{name}} here");
  assert.equal(format("hello {who}", { other: 1 }, "en"), "hello {who}", "an unknown value stays visible");
  assert.equal(format("on {bind}{n, plural, one { + {g}} other { + {g}}}", { bind: "x", g: "y" }, "en"), "on x{n, plural, one { + {g}} other { + {g}}}", "…a plural without its value too, whole");
  assert.equal(format("{n, plural, one {# x}", { n: 1 }, "en"), "{n, plural, one {# x}", "a broken plural is left as it is");
  assert.equal(format("# and {a}", { a: "b" }, "en"), "# and b", "# means the number only inside a plural");
  assert.equal(format("no params {x}", undefined, "en"), "no params {x}");
});

test("the language a person prefers, by base language", () => {
  const codes = LOCALES.map((locale) => locale.code);
  assert.equal(pickLanguage(["de-DE", "ru-RU", "en"], codes, SOURCE), "ru");
  assert.equal(pickLanguage(["en-GB"], codes, SOURCE), "en");
  assert.equal(pickLanguage(["ja", "fr"], codes, SOURCE), "en");
  assert.equal(pickLanguage([], codes, SOURCE), "en");
});

test("numbers and sizes follow the language", () => {
  setFormatLanguage("ru", { b: "Б", kb: "КБ", mb: "МБ", gb: "ГБ" });
  assert.equal(fmtNum(1234567).replace(/\s/g, " "), "1 234 567");
  assert.equal(fmtBytes(1536), "1,5 КБ");
  assert.equal(fmtBytes(512), "512 Б");
  setFormatLanguage("en", { b: "B", kb: "KB", mb: "MB", gb: "GB" });
  assert.equal(fmtNum(1234567), "1,234,567");
  assert.equal(fmtNum(2.5, 1), "2.5");
  assert.equal(fmtBytes(3 * 1024 * 1024), "3.00 MB");
  assert.equal(formatNumber(0.125, "en"), "0.125");
});

test("every language has every text, asks for the same values and has the plural forms it needs", () => {
  const keys = Object.keys(en);
  for (const { code, dict } of LOCALES) {
    assert.deepEqual(Object.keys(dict).sort(), [...keys].sort(), `${code}: the same keys as ${SOURCE}`);
    const categories = new Intl.PluralRules(code).resolvedOptions().pluralCategories;
    for (const key of keys) {
      const text = dict[key];
      assert.ok(typeof text === "string" && text.trim() !== "", `${code} ${key}: empty`);
      assert.deepEqual(placeholders(text), placeholders(en[key]), `${code} ${key}: the same values as ${SOURCE}`);
      for (const plural of plurals(text)) {
        assert.ok(plural.branches.has("other"), `${code} ${key}: a ${plural.kind} needs "other"`);
        if (plural.kind === "select") continue;
        for (const category of categories) {
          assert.ok(plural.branches.has(category), `${code} ${key}: ${plural.name} has no "${category}" form`);
        }
        for (const selector of plural.branches.keys()) {
          assert.ok(selector.startsWith("=") || categories.includes(selector), `${code} ${key}: "${selector}" is not a plural form of ${code}`);
        }
      }
    }
  }
});

/** Every file of the interface (and the e2e tour, which reads texts by key). */
function sources(dir) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return name === "locales" ? [] : sources(path);
    return /\.(ts|tsx)$/.test(name) ? [path] : [];
  });
}

test("every text the code names by a literal key exists", () => {
  const prefixes = new Set(Object.keys(en).map((key) => key.split(".")[0]));
  const missing = [];
  for (const file of [...sources("src"), ...sources("tests/e2e")]) {
    const code = readFileSync(file, "utf8");
    for (const found of code.matchAll(/"([a-z]+(?:\.[A-Za-z0-9_-]+)+)"/g)) {
      const key = found[1];
      if (!prefixes.has(key.split(".")[0]) || key in en) continue;
      // A key that is only the start of others (`"exp.op"` + `.${op}`) is a prefix, not a text.
      if (Object.keys(en).some((known) => known.startsWith(`${key}.`))) continue;
      missing.push(`${file}: ${key}`);
    }
  }
  assert.deepEqual(missing, [], "texts used but not in en.ts");
});

test("the starter set: the same signals as the engine, a name and a note for each in every language", () => {
  const engine = readFileSync("engine/src/signals.rs", "utf8");
  const seed = engine.slice(engine.indexOf("pub fn seed()"), engine.indexOf("#[cfg(test)]"));
  const ids = [...seed.matchAll(/sig\(\s*"([^"]+)"/g)].map((found) => found[1]);
  assert.deepEqual([...SEED_IDS].sort(), ids.sort(), "SEED_IDS in src/lib/library.ts matches seed() in engine/src/signals.rs");
  const groups = new Set([...seed.matchAll(/sig\(\s*"[^"]+",\s*"([^"]*)"/g)].map((found) => found[1]));
  for (const { code, dict } of LOCALES) {
    for (const id of ids) assert.ok(dict[`seed.${id}.name`] && dict[`seed.${id}.note`], `${code}: seed.${id}`);
  }
  // Folders named like a protocol stay as they are; any other has a text.
  for (const group of groups) assert.ok(/^[A-Z]{2,}$/.test(group) || en[`seed.folder.${group}`], `seed.folder.${group}`);

  const text = (key) => ({ "seed.osc-fader.name": "Фейдер", "seed.folder.Raw": "Сырые" })[key] ?? null;
  const [fader, raw, mine] = localizeSeed([
    { id: "osc-fader", name: "Fader value", group: "OSC", note: "n", body: { transport: "osc", target: "127.0.0.1:9000", address: "/f", args: [] } },
    { id: "udp-raw", name: "Raw UDP bytes", group: "Raw", note: "n", body: { transport: "udp", target: "127.0.0.1:9000", payload: { kind: "text", text: "" } } },
    { id: "my-own", name: "Mine", group: "Raw", note: "n", body: { transport: "osc", target: "127.0.0.1:9000", address: "/m", args: [] } },
  ], text);
  assert.equal(fader.name, "Фейдер");
  assert.equal(fader.group, "OSC", "no text: the folder keeps its name");
  assert.equal(fader.note, "n", "no text: the note stays");
  assert.equal(raw.group, "Сырые");
  assert.equal(mine.name, "Mine", "a signal of the person's own is never touched");
  assert.equal(mine.group, "Raw");
});
