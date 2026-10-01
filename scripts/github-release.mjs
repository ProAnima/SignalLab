#!/usr/bin/env node
// The GitHub side of a release, used by .github/workflows/release.yml.
//
//   node scripts/github-release.mjs draft v1.2.3       create (or refresh) the draft release, notes from CHANGELOG.md
//   node scripts/github-release.mjs checksums v1.2.3   SHA256SUMS.txt of every asset, uploaded next to them
//
// Needs GITHUB_TOKEN (or GH_TOKEN) with contents: write. GITHUB_REPOSITORY
// defaults to ProAnima/SignalLab. A published release is never modified:
// publishing is a person's decision, made on the releases page.

import { createHash } from "node:crypto";
import { appendFileSync } from "node:fs";
import { releaseNotes } from "./changelog.mjs";
import { fail, isMain } from "./lib.mjs";

const repository = process.env.GITHUB_REPOSITORY || "ProAnima/SignalLab";
const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
const API = `https://api.github.com/repos/${repository}`;
const UPLOADS = `https://uploads.github.com/repos/${repository}`;
const SUMS = "SHA256SUMS.txt";

async function github(url, { method = "GET", body, headers = {}, raw = false } = {}) {
  const response = await fetch(url, {
    method,
    headers: { Authorization: `Bearer ${token}`, Accept: "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28", ...headers },
    body,
  });
  if (!response.ok) throw new Error(`${method} ${url} → ${response.status} ${await response.text()}`);
  return raw ? response : response.status === 204 ? null : response.json();
}

/** The release for `tag`, drafts included (the tag lookup endpoint skips drafts). */
async function findRelease(tag) {
  for (let page = 1; page < 20; page++) {
    const releases = await github(`${API}/releases?per_page=100&page=${page}`);
    const found = releases.find((release) => release.tag_name === tag);
    if (found || releases.length < 100) return found ?? null;
  }
  return null;
}

function output(name, value) {
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `${name}=${value}\n`);
}

function summary(text) {
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${text}\n`);
}

async function draft(tag) {
  const version = tag.replace(/^v/, "");
  const body = releaseNotes(version);
  const existing = await findRelease(tag);
  let release;
  if (existing && !existing.draft) {
    fail(`${tag} is already published (${existing.html_url}); a published release is not modified`);
  } else if (existing) {
    release = await github(`${API}/releases/${existing.id}`, { method: "PATCH", body: JSON.stringify({ body, name: `Signal Lab ${version}` }) });
    console.log(`refreshed the draft for ${tag}: ${release.html_url}`);
  } else {
    release = await github(`${API}/releases`, {
      method: "POST",
      body: JSON.stringify({ tag_name: tag, name: `Signal Lab ${version}`, body, draft: true, prerelease: version.includes("-") }),
    });
    console.log(`created a draft for ${tag}: ${release.html_url}`);
  }
  output("id", release.id);
  summary(`Draft release for **${tag}**: ${release.html_url}`);
}

async function sha256(asset) {
  // The asset endpoint redirects to storage; fetch drops the token across that redirect.
  const response = await github(`${API}/releases/assets/${asset.id}`, { headers: { Accept: "application/octet-stream" }, raw: true });
  const hash = createHash("sha256");
  for await (const chunk of response.body) hash.update(chunk);
  return hash.digest("hex");
}

async function checksums(tag) {
  const release = await findRelease(tag);
  if (!release) fail(`no release for ${tag}`);
  if (!release.draft) fail(`${tag} is already published; its assets are not changed`);
  const assets = (await github(`${API}/releases/${release.id}/assets?per_page=100`)).filter((asset) => asset.name !== SUMS);
  if (!assets.length) fail(`the release for ${tag} has no assets to sum`);
  const lines = [];
  for (const asset of assets.sort((a, b) => a.name.localeCompare(b.name))) {
    const sum = await sha256(asset);
    lines.push(`${sum}  ${asset.name}`);
    console.log(`${sum}  ${asset.name} (${(asset.size / 1048576).toFixed(1)} MiB)`);
  }
  const old = (await github(`${API}/releases/${release.id}/assets?per_page=100`)).find((asset) => asset.name === SUMS);
  if (old) await github(`${API}/releases/assets/${old.id}`, { method: "DELETE" });
  await github(`${UPLOADS}/releases/${release.id}/assets?name=${SUMS}`, {
    method: "POST",
    headers: { "Content-Type": "text/plain" },
    body: lines.join("\n") + "\n",
  });
  summary(`\`${SUMS}\`\n\n\`\`\`\n${lines.join("\n")}\n\`\`\`\n\nReview and publish: ${release.html_url}`);
}

if (isMain(import.meta.url)) {
  const [command, tag] = process.argv.slice(2);
  if (!token) fail("GITHUB_TOKEN (or GH_TOKEN) is required");
  if (!tag || !/^v\d/.test(tag) || !["draft", "checksums"].includes(command)) fail("usage: node scripts/github-release.mjs draft|checksums vX.Y.Z");
  (command === "draft" ? draft(tag) : checksums(tag)).catch((error) => fail(error.message ?? String(error)));
}
