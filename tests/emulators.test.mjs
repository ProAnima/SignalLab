import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { LOCALES } from "../src/lib/locales/index.ts";
import {
  EMULATOR_SEED_IDS, PRESETS, blankEmulator, emulatorUrl, freeBind, localizeEmulatorSeed, makeEmulatorId, presetResponse, routeFromResponse, ruleCount,
} from "../src/lib/emulators.ts";
import { createNode, validPortsFor, requiredPortsFor } from "../src/lib/experimentGraph.ts";
import { canRetry, canRepeat, replyFields, writtenVariable } from "../src/lib/experimentData.ts";

const stored = (id, protocol, bind) => ({ id, note: "", emulator: blankEmulator(protocol, id, bind) });

test("the starter set: the same emulators as the engine, a name and a note for each in every language", () => {
  const engine = readFileSync("engine/src/emulator_files.rs", "utf8");
  const seed = engine.slice(engine.indexOf("pub fn seed()"), engine.indexOf("fn io_error"));
  const ids = [...seed.matchAll(/stored\(\s*"([^"]+)"/g)].map((found) => found[1]);
  assert.deepEqual([...EMULATOR_SEED_IDS].sort(), ids.sort(), "EMULATOR_SEED_IDS in src/lib/emulators.ts matches seed() in engine/src/emulator_files.rs");
  for (const { code, dict } of LOCALES) {
    for (const id of ids) assert.ok(dict[`seed.emu.${id}.name`] && dict[`seed.emu.${id}.note`], `${code}: seed.emu.${id}`);
  }
  const text = (key) => ({ "seed.emu.demo-api.name": "Демо API", "seed.emu.demo-api.note": "заметка" })[key] ?? null;
  const [demo, mine] = localizeEmulatorSeed([stored("demo-api", "http", "127.0.0.1:8080"), stored("mine", "http", "127.0.0.1:8081")], text);
  assert.equal(demo.emulator.name, "Демо API");
  assert.equal(demo.note, "заметка");
  assert.equal(mine.emulator.name, "mine", "an emulator of the person's own is never touched");
});

test("a new emulator works as it stands, on a free loopback port", () => {
  for (const protocol of ["http", "osc", "udp", "tcp"]) {
    const emulator = blankEmulator(protocol, "x", "127.0.0.1:1");
    assert.equal(emulator.protocol, protocol);
    assert.equal(ruleCount(emulator), 1, `${protocol} has one rule to start from`);
  }
  const taken = [stored("a", "http", "127.0.0.1:18080"), stored("b", "tcp", "127.0.0.1:18081"), stored("c", "udp", "127.0.0.1:18082")];
  assert.equal(freeBind("http", taken), "127.0.0.1:18082", "HTTP and TCP share the TCP ports; UDP's do not count");
  assert.equal(freeBind("osc", [stored("d", "udp", "127.0.0.1:9100")]), "127.0.0.1:9101");
  assert.equal(makeEmulatorId("Demo API", [stored("demo-api", "http", "127.0.0.1:1")]), "demo-api-2");
  assert.equal(makeEmulatorId("Новый API", []), "api", "only what an id can hold");
  assert.equal(emulatorUrl(blankEmulator("http", "x", "0.0.0.0:8080")), "http://127.0.0.1:8080");
  assert.equal(emulatorUrl(blankEmulator("http", "x", "0.0.0.0:8080"), undefined, "lab-pc"), "http://lab-pc:8080", "on a server, by the server's name");
  assert.equal(emulatorUrl(blankEmulator("http", "x", "127.0.0.1:8080"), undefined, "lab-pc"), "http://127.0.0.1:8080", "loopback stays loopback");
  assert.equal(emulatorUrl(blankEmulator("osc", "x", "127.0.0.1:9100")), null);
  for (const preset of PRESETS) {
    const response = presetResponse(preset);
    assert.ok(response.status >= 100 && response.status <= 599, preset);
  }
  assert.equal(presetResponse("timeout").fault, "timeout");
});

test("Mock this: the route answers like the response, its text taken literally", () => {
  const response = {
    ok: true, status: 201, status_text: "Created", latency_ms: 3, body_bytes: 20, truncated: false, error: null,
    headers: [["Content-Type", "application/json"], ["Date", "Thu, 01 Oct 2026"], ["Content-Length", "20"], ["X-Trace", "{{id}}"]],
    body: "{\"token\":\"{{x}}\"}",
  };
  const route = routeFromResponse("post", "http://127.0.0.1:8080/api/login?next=1", response);
  assert.equal(route.method, "POST");
  assert.equal(route.path, "/api/login");
  assert.deepEqual(route.responses[0].headers, [["Content-Type", "application/json"], ["X-Trace", "\\{{id}}"]], "what the server writes itself is left out");
  assert.equal(route.responses[0].body, "{\"token\":\"\\{{x}}\"}", "{{ in a recorded body is not a template");
  assert.equal(route.responses[0].status, 201);
});

test("the editor's nodes: an emulator and an HTTP wait that sees what it answers", () => {
  const emulator = createNode("emulator", 0, 0);
  const wait = createNode("wait_http", 0, 0);
  assert.equal(emulator.emulator.bind, wait.bind, "a new pair shares an address");
  assert.deepEqual(validPortsFor("wait_http"), ["matched", "timeout"]);
  assert.deepEqual(requiredPortsFor("wait_http"), ["matched"]);
  assert.deepEqual(validPortsFor("emulator"), ["next"]);
  assert.deepEqual(writtenVariable(wait), { name: "request", port: "matched" });
  assert.ok(replyFields(wait).includes("json") && replyFields(wait).includes("params"));
  assert.ok(canRetry(wait) && !canRepeat(wait) && !canRetry(emulator) && !canRepeat(emulator));
});
