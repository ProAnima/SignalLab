#!/usr/bin/env node
// The repository's labels as .github/labels.json lists them, used by
// .github/workflows/labels.yml whenever that file changes on main:
//
//   node scripts/labels.mjs sync      create the missing labels; give the others the listed colour and description
//   node scripts/labels.mjs check     the same comparison, changing nothing
//
// A label the file does not list is left alone — issues may carry it; delete it
// on the labels page. Names match without regard to case, as GitHub matches them.
//
// Needs GITHUB_TOKEN (or GH_TOKEN) with issues: write for sync; check reads the
// labels of a public repository without one. GITHUB_REPOSITORY defaults to
// ProAnima/SignalLab.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fail, isMain, root } from "./lib.mjs";

const repository = process.env.GITHUB_REPOSITORY || "ProAnima/SignalLab";
const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
const API = `https://api.github.com/repos/${repository}`;

/** `.github/labels.json`: `{name, color, description}` each, the colour as six hex digits. */
export const LABELS = JSON.parse(readFileSync(join(root, ".github", "labels.json"), "utf8"));

async function github(url, { method = "GET", body } = {}) {
  const response = await fetch(url, {
    method,
    headers: { ...(token && { Authorization: `Bearer ${token}` }), Accept: "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28" },
    body: body && JSON.stringify(body),
  });
  if (!response.ok) throw new Error(`${method} ${url} → ${response.status} ${await response.text()}`);
  return response.status === 204 ? null : response.json();
}

async function existing() {
  const found = [];
  for (let page = 1; page < 20; page++) {
    const labels = await github(`${API}/labels?per_page=100&page=${page}`);
    found.push(...labels);
    if (labels.length < 100) break;
  }
  return found;
}

/** What it takes to make the repository's labels the file's: `[{action, label, current?}]`. */
export function plan(wanted, current) {
  const byName = new Map(current.map((label) => [label.name.toLowerCase(), label]));
  return wanted.flatMap((label) => {
    const now = byName.get(label.name.toLowerCase());
    if (!now) return [{ action: "create", label }];
    const same = now.name === label.name && now.color.toLowerCase() === label.color && (now.description ?? "") === label.description;
    return same ? [] : [{ action: "update", label, current: now }];
  });
}

async function sync({ dry }) {
  if (!dry && !token) fail("GITHUB_TOKEN (or GH_TOKEN) is not set");
  const steps = plan(LABELS, await existing());
  for (const { action, label, current } of steps) {
    console.log(`${dry ? "would " : ""}${action} ${label.name}`);
    if (dry) continue;
    const body = { color: label.color, description: label.description };
    if (action === "create") await github(`${API}/labels`, { method: "POST", body: { name: label.name, ...body } });
    else await github(`${API}/labels/${encodeURIComponent(current.name)}`, { method: "PATCH", body: { new_name: label.name, ...body } });
  }
  console.log(`✔ ${LABELS.length} labels, ${steps.length} ${dry ? "to change" : "changed"}`);
}

if (isMain(import.meta.url)) {
  const command = process.argv[2];
  if (command === "sync") await sync({ dry: false });
  else if (command === "check") await sync({ dry: true });
  else fail("usage: node scripts/labels.mjs sync | check");
}
