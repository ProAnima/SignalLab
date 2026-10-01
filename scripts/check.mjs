#!/usr/bin/env node
// Every check a change must pass, in one place: `npm run check` on your
// machine and the CI workflow run exactly this, so "green here" means
// "green there". Stops at the first failure and says which step it was.
//
//   node scripts/check.mjs            all steps
//   node scripts/check.mjs --list     print the steps

import { join } from "node:path";
import { isMain, root, run, seconds } from "./lib.mjs";

const engine = join(root, "src-tauri");

/** `--locked`: lock files are part of a change; the checks must not rewrite them. */
export const STEPS = [
  { name: "versions agree", cwd: root, command: "node", args: ["scripts/version.mjs", "check"] },
  { name: "UI unit tests", cwd: root, command: "npm", args: ["test"] },
  { name: "UI type check and build", cwd: root, command: "npm", args: ["run", "build"] },
  { name: "engine lints (clippy, warnings are errors)", cwd: engine, command: "cargo", args: ["clippy", "--all-targets", "--locked", "--", "-D", "warnings"] },
  { name: "engine tests", cwd: engine, command: "cargo", args: ["test", "--locked"] },
];

/** Runs every step; returns false at the first failure. */
export function check() {
  const started = Date.now();
  for (const [index, step] of STEPS.entries()) {
    const at = Date.now();
    console.log(`\n▶ [${index + 1}/${STEPS.length}] ${step.name}`);
    const result = run(step.command, step.args, { cwd: step.cwd });
    if (!result.ok) {
      console.error(`\n✖ ${step.name} failed${result.error ? `: ${result.error.message}` : ` (exit ${result.status})`} after ${seconds(at)}`);
      return false;
    }
    console.log(`✔ ${step.name} · ${seconds(at)}`);
  }
  console.log(`\n✔ all ${STEPS.length} checks passed · ${seconds(started)}`);
  return true;
}

if (isMain(import.meta.url)) {
  if (process.argv.includes("--list")) {
    for (const step of STEPS) console.log(`${step.name}: ${step.command} ${step.args.join(" ")}`);
  } else if (!check()) {
    process.exit(1);
  }
}
