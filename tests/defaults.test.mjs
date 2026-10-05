// Every address the interface or a bundled template fills in by itself is
// on this machine: a default target never points at a host we don't own
// (see "Responsible use" in CLAUDE.md). The engine's signal seed has its own
// test (engine/src/signals.rs, seed_targets_stay_on_loopback).

import { test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(fileURLToPath(import.meta.url), "..", "..");

function files(dir, extensions) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return files(path, extensions);
    return extensions.some((extension) => name.endsWith(extension)) ? [path] : [];
  });
}

/**
 * Not addresses anything is sent to: an XML namespace, a placeholder that
 * shows the format, and the project's pages, which About and the documentation
 * button open in the browser (the repository, and the published documentation
 * that a build without the documentation inside falls back to).
 */
const NOT_TARGETS = new Set(["www.w3.org", "...", "github.com", "proanima.github.io"]);
const LOOPBACK = new Set(["127.0.0.1", "localhost", "[::1]"]);

test("default URLs in the interface and the templates are loopback", () => {
  const sources = [...files(join(root, "src"), [".ts", ".tsx"]), ...files(join(root, "experiments"), [".json"])];
  const outside = [];
  for (const file of sources) {
    const text = readFileSync(file, "utf8");
    for (const match of text.matchAll(/\bhttps?:\/\/([^/"'`\s:)]+)/g)) {
      const host = match[1];
      // Event channel names (`http://burst-progress`) are not URLs.
      if (/^[a-z]+(-[a-z]+)+$/.test(host) && !host.includes(".")) continue;
      if (!LOOPBACK.has(host) && !NOT_TARGETS.has(host)) outside.push(`${relative(root, file)}: ${match[0]}`);
    }
  }
  assert.deepEqual(outside, [], "a default points off this machine");
});

test("sources are plain text that git can diff", () => {
  // A NUL byte makes git treat the whole file as binary and hide its changes from review.
  const sources = [...files(join(root, "src"), [".ts", ".tsx", ".css"]), ...files(join(root, "tests"), [".mjs", ".ts", ".tsx"]), ...files(join(root, "scripts"), [".mjs"])];
  const binary = sources.filter((file) => readFileSync(file).includes(0)).map((file) => relative(root, file));
  assert.deepEqual(binary, []);
});
