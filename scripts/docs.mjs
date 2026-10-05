#!/usr/bin/env node
// The documentation (docs/, VitePress): pages written from the interface's
// dictionaries, the site built, its inline scripts moved into files.
//
//   node scripts/docs.mjs generate        the generated pages (reference/errors) in every language
//   node scripts/docs.mjs lint <pages…>   what is wrong with pages: front matter, ids, labels, links
//   node scripts/docs.mjs build           the site for the app and the server -> dist/docs (base /docs/)
//   node scripts/docs.mjs build --pages   the site for GitHub Pages -> artifacts/docs-pages (base /SignalLab/)
//   node scripts/docs.mjs dev             the site with live reload, http://localhost:5173/docs/
//
// The server sends `script-src 'self'`: a page may run only scripts it loads from
// a file, so the few VitePress writes into each page (the theme, the page map) are
// moved into files of their own after the build.

import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fail, isMain, must, root, seconds } from "./lib.mjs";

const DOCS = join(root, "docs");
const { LOCALES } = await import("../src/lib/locales/index.ts");

/** `{n, plural, one {# x} other {# xs}}` → `{n} xs`: what a message reads in general. */
export function general(text) {
  let out = "";
  for (let at = 0; at < text.length; ) {
    const choice = /^\{(\w+),\s*(plural|select|selectordinal),/.exec(text.slice(at));
    if (!choice) {
      out += text[at++];
      continue;
    }
    // The branches: `key {text}` pairs up to the closing brace; `other` is the general one.
    let depth = 0;
    let end = at;
    for (; end < text.length; end++) {
      if (text[end] === "{") depth++;
      else if (text[end] === "}" && --depth === 0) break;
    }
    const body = text.slice(at + choice[0].length, end);
    const branches = {};
    for (let i = 0; i < body.length; ) {
      const key = /^\s*(=?\w+)\s*\{/.exec(body.slice(i));
      if (!key) break;
      let level = 0;
      let start = i + key[0].length - 1;
      let j = start;
      for (; j < body.length; j++) {
        if (body[j] === "{") level++;
        else if (body[j] === "}" && --level === 0) break;
      }
      branches[key[1]] = body.slice(start + 1, j);
      i = j + 1;
    }
    out += general((branches.other ?? Object.values(branches)[0] ?? "").replace(/#/g, `{${choice[1]}}`));
    at = end + 1;
  }
  return out;
}

const cell = (text) => `<span v-pre>${general(text).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/\|/g, "&#124;").replace(/\n/g, " ")}</span>`;

/** reference/errors.md in every language: every `err.<code>` and `field.<key>` text of its dictionary. */
export async function generate() {
  const { TEXT } = await import("../docs/.vitepress/i18n.ts");
  for (const { code, dict } of LOCALES) {
    const words = TEXT[code].errors;
    const codes = Object.keys(dict).filter((key) => key.startsWith("err.")).map((key) => key.slice(4)).sort();
    const domains = [...new Set(codes.map((name) => name.split(".")[0]))];
    const lines = [
      "---",
      `title: ${JSON.stringify(words.title)}`,
      "# Written by scripts/docs.mjs from src/lib/locales — edit the dictionary, not this page.",
      "---",
      "",
      `# ${words.title}`,
      "",
      words.intro,
      "",
    ];
    for (const domain of domains) {
      lines.push(`## \`${domain}\` {#${domain}}`, "", `| ${words.code} | ${words.message} |`, "| --- | --- |");
      for (const name of codes.filter((item) => item.split(".")[0] === domain)) lines.push(`| \`${name}\` | ${cell(dict[`err.${name}`])} |`);
      lines.push("");
    }
    const fields = Object.keys(dict).filter((key) => key.startsWith("field.")).sort();
    lines.push(`## ${words.fields} {#fields}`, "", words.fieldsIntro, "", `| ${words.field} | ${words.name} |`, "| --- | --- |");
    for (const key of fields) lines.push(`| \`${key.slice(6)}\` | ${cell(dict[key])} |`);
    lines.push("");
    const folder = join(DOCS, ...(code === "en" ? [] : [code]), "reference");
    mkdirSync(folder, { recursive: true });
    writeFileSync(join(folder, "errors.md"), lines.join("\n"));
  }
}

/** A page's front matter as `key: value` pairs (titles may be quoted). */
export function frontMatter(text) {
  const block = /^---\r?\n([\s\S]*?)\r?\n---/.exec(text);
  const values = {};
  for (const line of block?.[1].split(/\r?\n/) ?? []) {
    const pair = /^([\w-]+):\s*(.*)$/.exec(line);
    if (pair) values[pair[1]] = pair[2].trim().replace(/^(["'])(.*)\1$/, "$2");
  }
  return values;
}

/** The page's text without front matter and fenced code (where `#` and `[x](y)` mean nothing). */
const prose = (text) => text.replace(/^---\r?\n[\s\S]*?\r?\n---/, "").replace(/^(`{3,}|~{3,})[\s\S]*?^\1/gm, "");

/** The explicit ids of a page's `##`–`####` headings, in order. */
export function headingIds(text) {
  return [...prose(text).matchAll(/^#{2,4} .*?\{#([\w.-]+)\}\s*$/gm)].map((match) => match[1]);
}

/**
 * What is wrong with documentation pages, before a build: front matter, an id
 * on every heading, every `[[ui:key]]` a label of the dictionary, every link to
 * a page and an id that exist (English ids — every language has the same).
 */
export function lint(paths) {
  const { dict } = LOCALES.find((locale) => locale.code === "en");
  const problems = [];
  for (const path of paths) {
    const file = path.startsWith(DOCS) ? path : join(root, path);
    const name = relative(root, file).replace(/\\/g, "/");
    const text = readFileSync(file, "utf8");
    const front = frontMatter(text);
    if (!front.title) problems.push(`${name}: no title`);
    if (front.layout !== "home" && !front.description && !name.includes("/develop/")) problems.push(`${name}: no description`);
    if (front.draft === "true") problems.push(`${name}: still a draft`);
    const body = prose(text);
    // The design notes and the roadmap are the record of their time, never translated:
    // their headings keep the ids VitePress gives them.
    const notes = /docs\/develop\/(design-[\w-]+|roadmap)\.md$/.test(name);
    for (const heading of body.matchAll(/^(#{2,4}) (.*)$/gm)) {
      if (!notes && !/\{#[\w.-]+\}\s*$/.test(heading[2])) problems.push(`${name}: heading without an id: ${heading[0]}`);
      if (heading[2].includes("[[ui:")) problems.push(`${name}: a label in a heading: ${heading[0]}`);
    }
    for (const match of body.replace(/`[^`\n]*`/g, "").matchAll(/\[\[ui:([A-Za-z0-9_.-]+)\]\]/g)) {
      const value = dict[match[1]];
      if (value === undefined) problems.push(`${name}: no interface text ${match[1]}`);
      else if (/[{}]/.test(value)) problems.push(`${name}: ${match[1]} has values to fill in, not a label`);
    }
    for (const link of body.replace(/`[^`\n]*`/g, "").matchAll(/\]\(([^)\s]+)\)/g)) {
      const target = link[1];
      if (/^(https?:|mailto:|#)/.test(target) && !target.startsWith("#")) continue;
      const [where, anchor] = target.split("#");
      const page = where ? join(dirname(file), where) : file;
      if (where && !where.endsWith(".md")) {
        problems.push(`${name}: link to ${target} — link pages by their .md file`);
        continue;
      }
      if (!existsSync(page)) {
        problems.push(`${name}: link to a missing page ${target}`);
        continue;
      }
      if (anchor) {
        // Ids are the same in every language; a translation's links are checked against English.
        const english = page.replace(new RegExp(`([\\\\/])docs[\\\\/](${LOCALES.map((locale) => locale.code).join("|")})[\\\\/]`), "$1docs$1");
        const ids = headingIds(readFileSync(existsSync(english) ? english : page, "utf8"));
        if (!ids.includes(anchor) && !GENERATED_IDS(page, anchor)) problems.push(`${name}: link to ${target} — no heading has that id`);
      }
    }
  }
  return problems;
}

/** The generated error page's ids: a domain of codes, or `fields`. */
const GENERATED_IDS = (page, anchor) => /reference[\\/]errors\.md$/.test(page) && (anchor === "fields" || /^[a-z_]+$/.test(anchor));

function* files(folder) {
  for (const name of readdirSync(folder)) {
    const path = join(folder, name);
    if (statSync(path).isDirectory()) yield* files(path);
    else yield path;
  }
}

/** Every inline script of every page into `assets/inline.<hash>.js`, in its place and order. */
export function externalize(out, base) {
  const written = new Set();
  let pages = 0;
  for (const file of files(out)) {
    if (!file.endsWith(".html")) continue;
    const html = readFileSync(file, "utf8");
    const moved = html.replace(/<script(?![^>]*\bsrc=)([^>]*)>([\s\S]*?)<\/script>/g, (whole, attributes, body) => {
      if (/type="(application\/(ld\+)?json|importmap)"/.test(attributes) || !body.trim()) return whole;
      const name = `inline.${createHash("sha256").update(body).digest("hex").slice(0, 16)}.js`;
      if (!written.has(name)) {
        writeFileSync(join(out, "assets", name), body);
        written.add(name);
      }
      return `<script${attributes} src="${base}assets/${name}"></script>`;
    });
    if (moved !== html) {
      writeFileSync(file, moved);
      pages++;
    }
    if (/<script(?![^>]*\bsrc=)[^>]*>\s*\S/.test(moved.replace(/<script[^>]*type="application\/(ld\+)?json"[^>]*>[\s\S]*?<\/script>/g, ""))) {
      fail(`${relative(root, file)} still has an inline script`);
    }
  }
  return { pages, scripts: written.size };
}

export async function build({ pages = false } = {}) {
  const at = Date.now();
  const base = pages ? "/SignalLab/" : "/docs/";
  const out = pages ? join(root, "artifacts", "docs-pages") : join(root, "dist", "docs");
  await generate();
  rmSync(out, { recursive: true, force: true });
  must("npx", ["vitepress", "build", "docs"], { env: { DOCS_BASE: base, DOCS_OUT: out } });
  const { pages: changed, scripts } = externalize(out, base);
  console.log(`✔ documentation in ${relative(root, out)} (base ${base}) · ${scripts} inline scripts moved out of ${changed} pages · ${seconds(at)}`);
}

if (isMain(import.meta.url)) {
  const [command, ...rest] = process.argv.slice(2);
  if (command === "generate") await generate();
  else if (command === "lint") {
    const problems = lint(rest);
    for (const problem of problems) console.error(`✖ ${problem}`);
    if (problems.length) process.exit(1);
    console.log(`✔ ${rest.length} pages`);
  } else if (command === "build") await build({ pages: rest.includes("--pages") });
  else if (command === "dev") {
    await generate();
    must("npx", ["vitepress", "dev", "docs"], { env: { DOCS_BASE: "/docs/" } });
  } else fail("usage: node scripts/docs.mjs generate | build [--pages] | dev");
}
