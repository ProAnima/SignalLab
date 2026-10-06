// The labels of .github/labels.json (scripts/labels.mjs puts them on the
// repository): each one valid for GitHub, and every label something else hands
// out — an issue form, Dependabot — among them, so none is made up on the fly.
import test from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { LABELS, plan } from "../scripts/labels.mjs";

const names = new Set(LABELS.map((label) => label.name.toLowerCase()));

test("every label has a name of its own, a colour and a description GitHub takes", () => {
  assert.equal(names.size, LABELS.length, "two labels with one name");
  for (const label of LABELS) {
    assert.ok(label.name.length > 0 && label.name.length <= 50, `${label.name}: a name of 1 to 50 characters`);
    assert.match(label.color, /^[0-9a-f]{6}$/, `${label.name}: the colour as six lowercase hex digits`);
    assert.ok(label.description.length > 0 && label.description.length <= 100, `${label.name}: a description of 1 to 100 characters`);
  }
});

test("the labels the issue forms and Dependabot give are listed", () => {
  const forms = ".github/ISSUE_TEMPLATE";
  for (const name of readdirSync(forms).filter((file) => file.endsWith(".yml") && file !== "config.yml")) {
    const given = /^labels:\s*\[(.*)\]/m.exec(readFileSync(`${forms}/${name}`, "utf8"))?.[1] ?? "";
    for (const label of given.split(",").map((item) => item.trim().replace(/^["']|["']$/g, "")).filter(Boolean)) {
      assert.ok(names.has(label.toLowerCase()), `${name} gives "${label}"`);
    }
  }
  // Dependabot labels its pull requests "dependencies" and the ecosystem's label.
  const ecosystems = { npm: "javascript", cargo: "rust", "github-actions": "github_actions", docker: "docker" };
  const config = readFileSync(".github/dependabot.yml", "utf8");
  for (const [, ecosystem] of config.matchAll(/package-ecosystem:\s*([\w-]+)/g)) {
    assert.ok(ecosystem in ecosystems, `a label for Dependabot's ${ecosystem} pull requests`);
    for (const label of ["dependencies", ecosystems[ecosystem]]) assert.ok(names.has(label), `Dependabot gives "${label}"`);
  }
});

test("a sync creates what is missing, updates what differs and leaves the rest", () => {
  const wanted = [
    { name: "bug", color: "d73a4a", description: "Broken" },
    { name: "area: mcp", color: "35c8e8", description: "MCP" },
    { name: "rust", color: "000000", description: "Rust" },
  ];
  const current = [
    { name: "Bug", color: "D73A4A", description: "Broken" },
    { name: "rust", color: "000000", description: "Rust" },
    { name: "kept", color: "ffffff", description: "Not in the file" },
  ];
  assert.deepEqual(
    plan(wanted, current).map(({ action, label }) => `${action} ${label.name}`),
    ["update bug", "create area: mcp"],
  );
});
