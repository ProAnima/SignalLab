// Helpers shared by the build and release scripts. No dependencies.

import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

/** The repository root. */
export const root = join(dirname(fileURLToPath(import.meta.url)), "..");

/** True when the module at `url` is the script node was started with. */
export function isMain(url) {
  if (!process.argv[1]) return false;
  const [a, b] = [resolve(process.argv[1]), fileURLToPath(url)];
  return process.platform === "win32" ? a.toLowerCase() === b.toLowerCase() : a === b;
}

/**
 * Start a program and wait for it. npm and npx are .cmd shims on Windows,
 * which only a shell can start, and a shell takes one command line (an
 * argument list together with `shell` is deprecated), so the line is quoted
 * here. `capture: true` returns stdout instead of printing it.
 */
export function run(command, args, { cwd = root, capture = false, env } = {}) {
  const options = { cwd, env: env ? { ...process.env, ...env } : process.env, encoding: "utf8", stdio: capture ? ["ignore", "pipe", "pipe"] : "inherit" };
  const result = process.platform === "win32"
    ? spawnSync([command, ...args].map(quote).join(" "), { ...options, shell: true })
    : spawnSync(command, args, options);
  return { ok: !result.error && result.status === 0, status: result.status, stdout: result.stdout ?? "", stderr: result.stderr ?? "", error: result.error };
}

function quote(arg) {
  return /[\s"&|<>^]/.test(arg) ? `"${arg.replace(/"/g, '\\"')}"` : arg;
}

/** Like `run`, but a failure ends the script with the reason. */
export function must(command, args, options) {
  const result = run(command, args, options);
  if (!result.ok) fail(`${command} ${args.join(" ")} failed${result.error ? `: ${result.error.message}` : ` (exit ${result.status})`}${result.stderr ? `\n${result.stderr.trim()}` : ""}`);
  return result.stdout.trimEnd();
}

export function fail(message) {
  console.error(`\n✖ ${message}`);
  process.exit(1);
}

export const seconds = (since) => `${((Date.now() - since) / 1000).toFixed(1)} s`;
