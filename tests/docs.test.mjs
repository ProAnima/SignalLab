// The documentation (docs/) against the code it describes: every language has
// every page and the same headings, pages name the interface by its keys, and
// the reference pages list every API command, event, kind of node, command of
// the command line and option of the server — none missing, none invented.
import test from "node:test";
import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { frontMatter, headingIds, lint } from "../scripts/docs.mjs";

const { SECTIONS, DEVELOP, GENERATED, SCREEN_PAGES } = await import("../docs/.vitepress/structure.ts");
const { TEXT } = await import("../docs/.vitepress/i18n.ts");
const { LOCALES } = await import("../src/lib/locales/index.ts");

const CODES = LOCALES.map((locale) => locale.code);
const PAGES = SECTIONS.flatMap((section) => section.pages.map((page) => `${section.id}/${page}`));
const WRITTEN = PAGES.filter((page) => !GENERATED.includes(page));
const file = (code, page) => join("docs", ...(code === "en" ? [] : [code]), `${page}.md`);
const read = (path) => readFileSync(path, "utf8");
const slash = (path) => path.replace(/\\/g, "/");

/** Every Markdown page under `folder`, as `section/page`, leaving out the folders named. */
function pagesIn(folder, skip = []) {
  const found = [];
  const walk = (dir) => {
    for (const name of readdirSync(dir)) {
      const path = join(dir, name);
      if (statSync(path).isDirectory()) {
        if (!name.startsWith(".") && !skip.includes(slash(relative(folder, path)))) walk(path);
      } else if (name.endsWith(".md")) found.push(slash(relative(folder, path)).replace(/\.md$/, ""));
    }
  };
  walk(folder);
  return found.sort();
}

const camelToSnake = (name) => name.replace(/[A-Z]/g, (letter, at) => (at ? "_" : "") + letter.toLowerCase());
const camelToKebab = (name) => camelToSnake(name).replace(/_/g, "-");

test("every language has every page, and only those; contributors' pages stay English", () => {
  const expected = ["index", ...PAGES].sort();
  for (const code of CODES) {
    const root = code === "en" ? "docs" : join("docs", code);
    const skip = code === "en" ? [...CODES.filter((other) => other !== "en"), "develop"] : [];
    const pages = pagesIn(root, skip);
    assert.deepEqual(pages, expected, `${code}: the pages of structure.ts`);
    assert.equal(frontMatter(read(file(code, "index"))).layout, "home", `${code}: a start page`);
    assert.ok(!existsSync(join(root, "develop")) || code === "en", `${code}: develop/ is English only`);
  }
  assert.deepEqual(pagesIn(join("docs", "develop")), [...DEVELOP.pages].sort());
});

test("pages are whole: front matter, an id on every heading, labels that exist, links that lead somewhere", () => {
  const all = [...CODES.flatMap((code) => ["index", ...WRITTEN].map((page) => file(code, page))), ...DEVELOP.pages.map((page) => file("en", `develop/${page}`))];
  assert.deepEqual(lint(all), []);
});

test("examples name the current release as [[version]], never the number written out", () => {
  const { version } = JSON.parse(read("package.json"));
  const written = new RegExp(`(?<![\\d.])${version.replace(/\./g, "\\.")}(?![.]?\\d)`);
  for (const code of CODES) {
    for (const page of ["index", ...WRITTEN]) {
      const lines = read(file(code, page)).split("\n");
      const at = lines.findIndex((line) => written.test(line));
      assert.equal(at, -1, `${code}/${page}.md:${at + 1} writes ${version}; write [[version]]`);
    }
  }
});

test("a page has the same heading ids in every language, so links and F1 work everywhere", () => {
  for (const page of WRITTEN) {
    const english = headingIds(read(file("en", page))).sort();
    for (const code of CODES.filter((code) => code !== "en")) {
      assert.deepEqual(headingIds(read(file(code, page))).sort(), english, `${code}/${page}`);
    }
  }
});

test("a translation keeps every interface label and every code block of its English page", () => {
  const labels = (text) => [...text.matchAll(/\[\[ui:([\w.-]+)\]\]/g)].map((match) => match[1]).sort();
  const blocks = (text) => (text.match(/^```[\s\S]*?^```/gm) ?? []).length;
  for (const page of ["index", ...WRITTEN]) {
    const english = read(file("en", page));
    for (const code of CODES.filter((code) => code !== "en")) {
      const translated = read(file(code, page));
      assert.deepEqual(labels(translated), labels(english), `${code}/${page}: the same [[ui:…]] labels`);
      assert.equal(blocks(translated), blocks(english), `${code}/${page}: the same code blocks`);
    }
  }
});

test("the site says everything it says in every language", () => {
  const shape = (value) => (typeof value === "object" ? Object.fromEntries(Object.entries(value).map(([key, inner]) => [key, shape(inner)])) : typeof value);
  for (const code of CODES) {
    assert.ok(TEXT[code], `${code}: texts of the site`);
    assert.deepEqual(shape(TEXT[code]), shape(TEXT.en), `${code}: the same texts as English`);
    const empty = JSON.stringify(TEXT[code]).match(/""/g);
    assert.equal(empty, null, `${code}: no empty text`);
  }
});

test("F1 has a page for every screen", () => {
  const screens = [...read("src/App.tsx").matchAll(/\{ key: "(\w+)"/g)].map((match) => match[1]).sort();
  assert.deepEqual(Object.keys(SCREEN_PAGES).sort(), screens);
  for (const page of Object.values(SCREEN_PAGES)) assert.ok(PAGES.includes(page), `${page} is a page of the documentation`);
});

test("the API reference names every command of the engine, and no other", () => {
  const commands = [...read("engine/src/service.rs").matchAll(/^\s+"([a-z_]+)" =>/gm)].map((match) => match[1]).sort();
  assert.ok(commands.length > 40, "the command table was read");
  const ids = headingIds(read(file("en", "api/commands"))).filter((id) => /^[a-z]+(_[a-z]+)+$|^[a-z]+$/.test(id) && commands.includes(id));
  assert.deepEqual([...new Set(ids)].sort(), commands);
  const documented = headingIds(read(file("en", "api/commands"))).filter((id) => /_/.test(id));
  assert.deepEqual(documented.filter((id) => !commands.includes(id)), [], "no command that is not the engine's");
});

test("the events page names every channel the interface listens to", () => {
  const block = /export const EV = \{([\s\S]*?)\} as const/.exec(read("src/lib/api.ts"))[1];
  const channels = [...block.matchAll(/"([a-z-]+:\/\/[a-z-]+)"/g)].map((match) => `event-${match[1].replace(/:\/\//, "-")}`).sort();
  const ids = headingIds(read(file("en", "api/events"))).filter((id) => id.startsWith("event-")).sort();
  assert.deepEqual(ids, channels);
});

test("the node reference has every kind of node", () => {
  const body = /pub enum NodeKind \{([\s\S]*?)\n\}/.exec(read("engine/src/experiment.rs"))[1];
  const kinds = [...body.matchAll(/^ {4}([A-Z]\w*)/gm)].map((match) => `node-${camelToSnake(match[1])}`).sort();
  assert.ok(kinds.length >= 30);
  const ids = headingIds(read(file("en", "experiments/nodes"))).filter((id) => id.startsWith("node-")).sort();
  assert.deepEqual(ids, kinds);
});

test("the command line's page has every command of signallab", () => {
  const body = /enum Command \{([\s\S]*?)\n\}/.exec(read("cli/src/main.rs"))[1];
  const commands = [...body.matchAll(/^ {4}([A-Z]\w*)/gm)].map((match) => `cli-${camelToKebab(match[1])}`).sort();
  const ids = headingIds(read(file("en", "automation/cli"))).filter((id) => id.startsWith("cli-") && !id.includes("--"));
  for (const command of commands) assert.ok(ids.includes(command), `${command} is documented`);
});

test("the server's page has every option and environment variable", () => {
  const config = read("server/src/config.rs");
  const page = read(file("en", "server/index"));
  const variables = [...config.matchAll(/env = "(SIGNALLAB_[A-Z_]+)"/g)].map((match) => match[1]);
  assert.ok(variables.length >= 8);
  for (const variable of variables) assert.ok(page.includes(variable), `${variable} on server/index.md`);
  const flags = [...config.matchAll(/#\[arg\(long(?: = "([a-z-]+)")?[^\]]*\)\]\s*(?:\/\/\/[^\n]*\n\s*)*(?:pub )?(\w+):/g)]
    .map((match) => `--${match[1] ?? match[2].replace(/_/g, "-")}`);
  for (const flag of flags) assert.ok(page.includes(flag), `${flag} on server/index.md`);
});
