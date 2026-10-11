// What the console and the Inspector do with a storm: lines and frames drawn
// together, bounded, and what was not drawn said (src/lib/consoleLog.ts,
// src/lib/capture.ts), and the library at ten thousand signals (src/lib/library.ts).
import test from "node:test";
import assert from "node:assert/strict";
import { appendLines, batcher, LOG_CAPACITY, LOG_FLUSH_MS } from "../src/lib/consoleLog.ts";
import { appendFrames } from "../src/lib/capture.ts";
import { libraryRows, libraryTree } from "../src/lib/library.ts";

test("a storm of console lines is one draw, and keeps the newest", async () => {
  const draws = [];
  const lines = batcher((items) => draws.push(items));
  // Three hundred jobs stopped at once: a line each, and once one per draw of the whole app.
  for (let i = 0; i < 1200; i++) lines.push(i);
  assert.equal(draws.length, 0, "nothing is drawn per line");
  await new Promise((done) => setTimeout(done, LOG_FLUSH_MS + 50));
  assert.equal(draws.length, 1, "one draw for the lot");
  const shown = appendLines([], draws[0]);
  assert.equal(shown.length, LOG_CAPACITY);
  assert.equal(shown[shown.length - 1], 1199, "the newest line is there");
  assert.ok(draws[0].length <= LOG_CAPACITY * 2, "what waits between two draws is bounded too");
});

test("console lines: the newest capacity, the old ones giving way", () => {
  assert.deepEqual(appendLines([1, 2, 3], [4, 5], 4), [2, 3, 4, 5]);
  assert.deepEqual(appendLines([1, 2], [3, 4, 5, 6, 7], 4), [4, 5, 6, 7]);
  const same = [1];
  assert.equal(appendLines(same, [], 4), same, "nothing new, the same list (no draw)");
  const lines = batcher(() => assert.fail("cleared"), 5);
  lines.push(1);
  lines.clear();
  lines.flushNow();
});

const frame = (seq) => ({ seq, ts: 0, proto: "udp", dir: "rx", source: "t", job_id: null, local: "", remote: "", bytes: 1, summary: "", detail: null, hex: null, verdict: null, kept: 1 });
const frames = (from, to) => Array.from({ length: to - from + 1 }, (_, i) => frame(from + i));

test("the Inspector says how many frames it never showed, paused or skipped", () => {
  let rows = appendFrames([], frames(1, 3), 0, 10);
  assert.deepEqual(rows.map((r) => [r.seq, r.gap]), [[1, undefined], [2, undefined], [3, undefined]]);
  // The engine shipped 250 of a busy tick's 400: it says 150 were skipped.
  rows = appendFrames(rows, frames(154, 156), 150, 10);
  assert.equal(rows[3].gap, 150);
  // Paused: batches 157…200 were dropped by the view; the engine skipped none of them.
  rows = appendFrames(rows, frames(201, 202), 0, 10);
  assert.equal(rows[6].gap, 44, "the frames that came while paused are not shown, and it says so");
  // The snapshot and the first batch overlap: nothing twice, no gap.
  const again = appendFrames(rows, frames(200, 203), 0, 10);
  assert.deepEqual(again.slice(-2).map((r) => [r.seq, r.gap]), [[202, undefined], [203, undefined]]);
  assert.equal(appendFrames(again, frames(1, 203), 0, 10), again, "nothing new, the same list");
  assert.equal(appendFrames(again, frames(204, 300), 0, 10).length, 10, "at most its capacity");
});

const signal = (i, group) => ({ id: `s${i}`, name: `Signal ${i}`, group, note: "", body: { transport: "osc", target: "127.0.0.1:9000", address: `/x/${i}`, args: [] } });

test("ten thousand signals: the rows a scrolled tree draws, in order, closed folders skipped", () => {
  const signals = [];
  for (let i = 0; i < 10_000; i++) signals.push(signal(i, i % 4 === 0 ? "" : `Rack ${i % 10}/Bank ${i % 7}/Deep ${i % 3}`));
  const started = performance.now();
  const tree = libraryTree(signals, []);
  const all = libraryRows(tree, () => true);
  const took = performance.now() - started;
  assert.equal(all.filter((row) => row.kind === "signal").length, 10_000, "every signal a row when all is open");
  assert.ok(took < 1000, `tree and rows of 10 000 in ${took.toFixed(0)} ms`);
  // Rows run as the tree reads: a folder, then its folders, then its signals.
  assert.deepEqual(all.slice(0, 3).map((row) => [row.kind, row.depth, row.kind === "folder" ? row.node.path : row.signal.id]),
    [["folder", 0, "Rack 0"], ["folder", 1, "Rack 0/Bank 0"], ["folder", 2, "Rack 0/Bank 0/Deep 0"]]);
  const closed = libraryRows(tree, (path) => !path.startsWith("Rack"));
  assert.deepEqual(closed.filter((row) => row.kind === "folder").map((row) => row.node.path), tree.folders.map((node) => node.path));
  assert.equal(closed.filter((row) => row.kind === "signal").length, 2500, "the top level's signals, closed folders' not");
});
