import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { LOCALES } from "../src/lib/locales/index.ts";
import {
  EMULATOR_SEED_IDS, PRESETS, blankEmulator, emulatorUrl, freeBind, localizeEmulatorSeed, makeEmulatorId, mockPath, presetResponse, routeFromResponse, ruleCount,
} from "../src/lib/emulators.ts";
import { createNode, validPortsFor, requiredPortsFor } from "../src/lib/experimentGraph.ts";
import { canRetry, canRepeat, replyFields, writtenVariable } from "../src/lib/experimentData.ts";
import { describeError } from "../src/lib/errors.ts";
import { en } from "../src/lib/locales/en.ts";

const stored = (id, protocol, bind) => ({ id, note: "", emulator: blankEmulator(protocol, id, bind) });

test("the starter set: the same emulators as the engine, a name and a note for each in every language", () => {
  const engine = readFileSync("engine/src/emulator_files.rs", "utf8");
  const seed = engine.slice(engine.indexOf("pub fn seed()"), engine.indexOf("fn io_error"));
  const ids = [...seed.matchAll(/stored\(\s*"([^"]+)"/g)].map((found) => found[1]);
  assert.deepEqual([...EMULATOR_SEED_IDS].sort(), ids.sort(), "EMULATOR_SEED_IDS in src/lib/emulators.ts matches seed() in engine/src/emulator_files.rs");
  for (const { code, dict } of LOCALES) {
    for (const id of ids) assert.ok(dict[`seed.emu.${id}.name`] && dict[`seed.emu.${id}.note`], `${code}: seed.emu.${id}`);
  }
  // The engine writes the notes in English until the interface renames them; they say what en.ts says.
  const notes = [...seed.matchAll(/stored\(\s*"([^"]+)",\s*"((?:[^"\\]|\\.)*)"/g)].map((found) => [found[1], found[2]]);
  assert.equal(notes.length, ids.length, "a note for every starter emulator");
  for (const [id, note] of notes) assert.equal(note, en[`seed.emu.${id}.note`], `the engine's note of ${id} is the interface's English one`);
  const text = (key) => ({ "seed.emu.demo-api.name": "Демо API", "seed.emu.demo-api.note": "заметка" })[key] ?? null;
  const [demo, mine] = localizeEmulatorSeed([stored("demo-api", "http", "127.0.0.1:8080"), stored("mine", "http", "127.0.0.1:8081")], text);
  assert.equal(demo.emulator.name, "Демо API");
  assert.equal(demo.note, "заметка");
  assert.equal(mine.emulator.name, "mine", "an emulator of the person's own is never touched");
});

test("a new emulator works as it stands, on a free loopback port", () => {
  for (const protocol of ["http", "osc", "udp", "tcp", "mqtt"]) {
    const emulator = blankEmulator(protocol, "x", "127.0.0.1:1");
    assert.equal(emulator.protocol, protocol);
    assert.equal(ruleCount(emulator), 1, `${protocol} has one rule to start from`);
  }
  const taken = [stored("a", "http", "127.0.0.1:18080"), stored("b", "tcp", "127.0.0.1:18081"), stored("c", "udp", "127.0.0.1:18082")];
  assert.equal(freeBind("http", taken), "127.0.0.1:18082", "HTTP and TCP share the TCP ports; UDP's do not count");
  assert.equal(freeBind("osc", [stored("d", "udp", "127.0.0.1:9100")]), "127.0.0.1:9101");
  assert.equal(freeBind("mqtt", taken), "127.0.0.1:1883", "a broker starts at MQTT's own port");
  assert.equal(freeBind("tcp", [stored("m", "mqtt", "127.0.0.1:7200")]), "127.0.0.1:7201", "a broker listens on TCP too");
  assert.equal(makeEmulatorId("Demo API", [stored("demo-api", "http", "127.0.0.1:1")]), "demo-api-2");
  assert.equal(makeEmulatorId("Новый API", []), "api", "only what an id can hold");
  assert.equal(emulatorUrl(blankEmulator("http", "x", "0.0.0.0:8080")), "http://127.0.0.1:8080");
  assert.equal(emulatorUrl(blankEmulator("http", "x", "0.0.0.0:8080"), undefined, "lab-pc"), "http://lab-pc:8080", "on a server, by the server's name");
  assert.equal(emulatorUrl(blankEmulator("http", "x", "127.0.0.1:8080"), undefined, "lab-pc"), "http://127.0.0.1:8080", "loopback stays loopback");
  assert.equal(emulatorUrl(blankEmulator("osc", "x", "127.0.0.1:9100")), null);
  assert.equal(emulatorUrl(blankEmulator("mqtt", "x", "127.0.0.1:1883")), null, "a broker has no URL to open");
  for (const preset of PRESETS) {
    const response = presetResponse(preset);
    assert.ok(response.status >= 100 && response.status <= 599, preset);
  }
  assert.equal(presetResponse("timeout").fault, "timeout");
  const malformed = presetResponse("malformed");
  assert.ok(malformed.fault === "malformed" && JSON.parse(malformed.body), "a malformed answer is cut from a body that parses");
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
  // From an experiment's Send now, the URL is a template: its base goes, whole-template segments name themselves.
  assert.equal(mockPath("{{params.api}}/orders/{{vars.order-id}}/items"), "/orders/:order_id/items");
  assert.equal(mockPath("http://{{host}}:8080/a/{{id}}?x={{y}}"), "/a/:id");
  assert.equal(mockPath("{{api}}/files/report-{{date}}.pdf/x"), "/files/*", "a segment only partly a template takes the rest");
  assert.equal(mockPath("{{api}}"), "/");
  assert.equal(mockPath("http://127.0.0.1:8080/a%20b?q=1"), "/a%20b");
  assert.equal(routeFromResponse("get", "{{api}}/users/{{id}}", response).path, "/users/:id");
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

test("a problem in an emulator names its rule, retained message and response", () => {
  const t = (key, params) => {
    const text = en[key] ?? key;
    return params ? text.replace(/\{(\w+)\}/g, (match, name) => name in params ? String(params[name]) : match) : text;
  };
  const route = describeError({ code: "emulator.name_unknown", params: { name: "vars", rule: "2", response: "3" }, node: "mock", field: { key: "body" } }, t, () => "Emulator");
  assert.equal(route.where, "Emulator · Rule 2 · Response 3 · Body");
  assert.equal(describeError({ code: "node.topic_wildcard", params: { retained: "1" }, field: { key: "topic" } }, t).where, "Retained 1 · Topic");
  assert.equal(describeError({ code: "transport.refused", params: { target: "x" } }, t).where, "", "other failures are not about rules");
});
