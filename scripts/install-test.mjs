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
import { mkdirSync, writeFileSync } from "node:fs";
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

// A stand-in for ufw: "active", and every other call written down — a real one
// would change the firewall of the machine Docker runs on.
const fakes = join(root, "target", "install-test-bin");
mkdirSync(fakes, { recursive: true });
writeFileSync(join(fakes, "ufw"), `#!/bin/sh\nif [ "$1" = status ]; then echo "Status: active"; exit 0; fi\necho "$*" >> /opt/${name}/ufw.log\n`, { mode: 0o755 });

/** The install script in a Linux shell on the host's Docker; its output, and its exit code. */
function install(...args) {
  const deploy = join(root, "deploy").replaceAll("\\", "/");
  const ufw = args[0] === "--with-ufw" ? args.shift() : null;
  const run = spawnSync("docker", [
    "run", "--rm", "--network", "host",
    "-v", "/var/run/docker.sock:/var/run/docker.sock",
    "-v", `${deploy}:/deploy:ro`,
    // The folder the compose file goes to outlives each run, as it does on a real machine.
    "-v", `${name}-files:/opt/${name}`,
    ...(ufw ? ["-v", `${fakes.replaceAll("\\", "/")}:/fake:ro`, "-e", "PATH=/fake:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"] : []),
    CLI, "sh", "/deploy/install.sh", "--image", image, "--name", name, "--port", String(port), "--yes", ...args,
  ], { encoding: "utf8", env: { ...process.env, MSYS_NO_PATHCONV: "1" } });
  return { out: `${run.stdout}${run.stderr}`, code: run.status };
}

/** What the stand-in ufw was asked. */
const ufwLog = () => spawnSync("docker", ["run", "--rm", "-v", `${name}-files:/opt/${name}`, CLI, "sh", "-c", `cat /opt/${name}/ufw.log 2>/dev/null`], { encoding: "utf8", env: { ...process.env, MSYS_NO_PATHCONV: "1" } }).stdout;

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

  // With ufw on: the server's port and the UDP ports asked for are opened, and remembered.
  const walled = install("--with-ufw", "--open-udp", "9000,9100:9110");
  const opened = ufwLog();
  check("with ufw on, it opens the server's port and the UDP ports asked for", walled.code === 0
    && opened.includes(`allow ${port}/tcp comment Signal Lab`) && opened.includes("allow 9000/udp") && opened.includes("allow 9100:9110/udp"), `${opened}\n${walled.out.slice(-600)}`);

  const removed = install("--with-ufw", "--uninstall");
  check("--uninstall removes the server", removed.code === 0 && spawnSync("docker", ["inspect", name]).status !== 0, removed.out);
  const closed = ufwLog();
  check("…and closes in ufw what the install opened", closed.includes(`delete allow ${port}/tcp`) && closed.includes("delete allow 9100:9110/udp"), closed);
  check("…and keeps its data", volumes().length === 1, volumes().join(", "));
  const back = install();
  check("installing again brings the same token back", back.code === 0 && tokenIn(back.out) === token);

  const purged = install("--uninstall", "--purge");
  check("--uninstall --purge deletes the data too", purged.code === 0 && volumes().length === 0, purged.out);

  const wrong = install("--port", "http");
  check("a wrong option is refused before anything happens", wrong.code !== 0 && /--port is a number/.test(wrong.out));
  const ports = install("--open-udp", "9000;rm");
  check("…and a wrong list of ports too", ports.code !== 0 && /--open-udp is ports/.test(ports.out));
} finally {
  // Whatever happened above, leave nothing of the test behind.
  spawnSync("docker", ["rm", "-f", name], { stdio: "ignore" });
  for (const volume of [...volumes(), `${name}-files`]) spawnSync("docker", ["volume", "rm", "-f", volume], { stdio: "ignore" });
}

console.log(failures.length ? `\n✖ ${failures.length} failed` : "\n✔ install.sh works end to end");
process.exit(failures.length ? 1 : 0);
