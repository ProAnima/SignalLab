#!/usr/bin/env node
// deploy/install.sh, run for real against this machine's Docker: install,
// sign in with the token it prints, run it again (an update keeps the token),
// uninstall (the data stays), install again (the same token), uninstall --purge.
//
//   node scripts/install-test.mjs [--image signallab:dev] [--port 14390]
//
// The script runs inside docker:cli (a Linux shell with the docker and compose
// clients) talking to the host's daemon, under a name of its own so nothing of
// a real installation is touched. Needs the image built first (npm run check:image).
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const arg = (name, fallback) => { const at = process.argv.indexOf(name); return at > 0 ? process.argv[at + 1] : fallback; };
const image = arg("--image", "signallab:dev");
const port = Number(arg("--port", "14390"));
const name = "signallab-install-test";
const CLI = "docker:27-cli";

const failures = [];
const check = (what, ok, detail = "") => { console.log(`${ok ? "✔" : "✖"} ${what}${ok || !detail ? "" : ` — ${detail}`}`); if (!ok) failures.push(what); };

/** The install script in a Linux shell on the host's Docker; its output, and its exit code. */
function install(...args) {
  const deploy = join(root, "deploy").replaceAll("\\", "/");
  const run = spawnSync("docker", [
    "run", "--rm", "--network", "host",
    "-v", "/var/run/docker.sock:/var/run/docker.sock",
    "-v", `${deploy}:/deploy:ro`,
    // The folder the compose file goes to outlives each run, as it does on a real machine.
    "-v", `${name}-files:/opt/${name}`,
    CLI, "sh", "/deploy/install.sh", "--image", image, "--name", name, "--port", String(port), "--yes", ...args,
  ], { encoding: "utf8", env: { ...process.env, MSYS_NO_PATHCONV: "1" } });
  return { out: `${run.stdout}${run.stderr}`, code: run.status };
}

/** An HTTP request from a shell on the host network, where the server listens. */
function http(path, token, method = "GET", body) {
  const headers = token ? ["--header", `Authorization: Bearer ${token}`] : [];
  const data = body ? ["--header", "Content-Type: application/json", "--post-data", body] : [];
  const run = spawnSync("docker", ["run", "--rm", "--network", "host", CLI, "wget", "-S", "-qO-", ...headers, ...data, `http://127.0.0.1:${port}${path}`],
    { encoding: "utf8", env: { ...process.env, MSYS_NO_PATHCONV: "1" } });
  const status = Number(/HTTP\/1\.1 (\d+)/.exec(run.stderr)?.[1] ?? 0);
  return { status, body: run.stdout };
}

const tokenIn = (out) => /Token:\s+([0-9a-f]{64})/.exec(out)?.[1] ?? null;
const volumes = () => spawnSync("docker", ["volume", "ls", "--format", "{{.Name}}"], { encoding: "utf8" }).stdout.split("\n").filter((volume) => volume.startsWith(`${name}_`));

try {
  const first = install();
  check("installs and says it is running", first.code === 0 && /is running/.test(first.out), first.out.slice(-1500));
  const token = tokenIn(first.out);
  check("prints the access token it made", !!token);
  check("the server answers its health check", http("/api/health").status === 200);
  check("…and refuses a command without the token", http("/api/invoke/app_info", null, "POST", "null").status === 401);
  const info = http("/api/invoke/app_info", token, "POST", "null");
  check("…and runs one with it", info.status === 200 && /"mode":"server"/.test(info.body), `${info.status} ${info.body}`);

  const again = install();
  check("running it again updates in place", again.code === 0 && /is running/.test(again.out), again.out.slice(-800));
  check("…and keeps the token", tokenIn(again.out) === token);

  const removed = install("--uninstall");
  check("--uninstall removes the server", removed.code === 0 && spawnSync("docker", ["inspect", name]).status !== 0, removed.out);
  check("…and keeps its data", volumes().length === 1, volumes().join(", "));
  const back = install();
  check("installing again brings the same token back", back.code === 0 && tokenIn(back.out) === token);

  const purged = install("--uninstall", "--purge");
  check("--uninstall --purge deletes the data too", purged.code === 0 && volumes().length === 0, purged.out);

  const wrong = install("--port", "http");
  check("a wrong option is refused before anything happens", wrong.code !== 0 && /--port is a number/.test(wrong.out));
} finally {
  // Whatever happened above, leave nothing of the test behind.
  spawnSync("docker", ["rm", "-f", name], { stdio: "ignore" });
  for (const volume of [...volumes(), `${name}-files`]) spawnSync("docker", ["volume", "rm", "-f", volume], { stdio: "ignore" });
}

console.log(failures.length ? `\n✖ ${failures.length} failed` : "\n✔ install.sh works end to end");
process.exit(failures.length ? 1 : 0);
