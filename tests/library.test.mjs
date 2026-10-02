import test from "node:test";
import assert from "node:assert/strict";
import {
  addFolder, allFolders, ancestors, freeFolderName, libraryTree, moveFolder, moveSignal, normalizeFolder, removeFolder, renameFolder,
  canonicalBody, signalPlace,
} from "../src/lib/library.ts";

const signal = (id, group) => ({ id, name: id, group, note: "", body: { transport: "osc", target: "127.0.0.1:9000", address: "/x", args: [] } });

test("folders are paths; a group from before folders is a top-level folder", () => {
  assert.equal(normalizeFolder("  API / Auth/ "), "API/Auth");
  assert.equal(normalizeFolder("//"), "");
  const signals = [signal("a", "OSC"), signal("b", "API/Auth"), signal("c", ""), signal("d", "API/Auth")];
  assert.deepEqual(allFolders(signals, ["Empty/Inner"]), ["API", "API/Auth", "Empty", "Empty/Inner", "OSC"]);
  const tree = libraryTree(signals, ["Empty/Inner"]);
  assert.deepEqual(tree.folders.map((node) => node.name), ["API", "Empty", "OSC"], "folders by name");
  assert.deepEqual(tree.signals.map((s) => s.id), ["c"], "the root holds what has no folder");
  assert.equal(tree.total, 4);
  const api = tree.folders[0];
  assert.equal(api.total, 2, "a folder counts what is inside its folders too");
  assert.deepEqual(api.folders[0].signals.map((s) => s.id), ["b", "d"], "signals keep file order");
  assert.equal(tree.folders[1].folders[0].path, "Empty/Inner");
  const numbered = libraryTree([signal("x", "Scene 10"), signal("y", "Scene 2")], []);
  assert.deepEqual(numbered.folders.map((node) => node.name), ["Scene 2", "Scene 10"], "numbers in order, not as text");
  assert.deepEqual(ancestors("A/B/C"), ["A", "A/B", "A/B/C"]);
});

test("new, renamed, moved and removed folders keep every signal", () => {
  let state = { signals: [signal("a", "API"), signal("b", "API/Auth"), signal("c", "")], folders: [] };
  assert.equal(freeFolderName(state.signals, state.folders, "", "API"), "API 2");
  state = addFolder(state, "Venue/Stage");
  assert.deepEqual(state.folders, ["API", "API/Auth", "Venue", "Venue/Stage"], "every folder is listed, the empty ones too");

  const renamed = renameFolder(state, "API", "Service");
  assert.deepEqual(renamed.signals.map((s) => s.group), ["Service", "Service/Auth", ""], "everything inside follows");
  assert.equal(renameFolder(state, "API", "Venue"), null, "a rename never merges into a sibling");
  assert.equal(renameFolder(state, "API", "a/b"), null);
  assert.equal(renameFolder(state, "API", "  "), null);

  const moved = moveFolder(state, "API/Auth", "Venue");
  assert.deepEqual(moved.signals.map((s) => s.group), ["API", "Venue/Auth", ""]);
  assert.deepEqual(moved.folders, ["API", "Venue", "Venue/Auth", "Venue/Stage"]);
  assert.equal(moveFolder(state, "API", "API/Auth"), null, "not into itself");

  const removed = removeFolder(state, "API");
  assert.deepEqual(removed.signals.map((s) => s.group), ["", "Auth", ""], "removing a folder moves what is in it up a level");
  const emptyGone = removeFolder(state, "Venue/Stage");
  assert.deepEqual(emptyGone.folders, ["API", "API/Auth", "Venue"], "its parent stays, now empty itself");

  const filed = moveSignal(state, "c", "Venue/Stage");
  assert.equal(filed.signals.find((s) => s.id === "c").group, "Venue/Stage");
  const emptied = moveSignal(filed, "b", "");
  assert.ok(emptied.folders.includes("API/Auth"), "a folder its last signal left stays");
});

test("a sender knows it holds what it was saved as, whatever order the engine sends fields in", () => {
  const typed = { transport: "http", request: { method: "GET", url: "http://127.0.0.1:8080/", headers: [["Accept", "application/json"]], body: null, timeout_ms: 10000 } };
  const fromEngine = { request: { body: null, headers: [["Accept", "application/json"]], method: "GET", timeout_ms: 10000, url: "http://127.0.0.1:8080/" }, transport: "http" };
  assert.equal(canonicalBody(typed), canonicalBody(fromEngine));
  assert.notEqual(canonicalBody(typed), canonicalBody({ ...fromEngine, request: { ...fromEngine.request, url: "http://127.0.0.1:8081/" } }));
  // OSC floats are stored at 32 bits: 0.1 comes back as the same float.
  const osc = (value) => ({ transport: "osc", target: "127.0.0.1:9000", address: "/x", args: [{ type: "float", value }] });
  assert.equal(canonicalBody(osc(0.1)), canonicalBody(osc(Math.fround(0.1))));
  assert.notEqual(canonicalBody(osc(0.1)), canonicalBody(osc(0.2)));
  assert.equal(signalPlace(signal("Login", " API / Auth ")), "API / Auth / Login");
  assert.equal(signalPlace(signal("Ping", "")), "Ping");
});

