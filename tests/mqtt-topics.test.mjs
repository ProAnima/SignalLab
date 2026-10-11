import test from "node:test";
import assert from "node:assert/strict";
import { flattenTopics, ingestTopic, makeTopicRoot, MAX_TOPICS, sortedChildren, visibleRows } from "../src/lib/topics.ts";

const message = (topic, extra = {}) => ({ ts: 1, topic, payload: "1", bytes: 1, qos: 0, retain: false, dup: false, ...extra });

test("a batch's message counts what it stands for, and the tree keeps the latest", () => {
  const root = makeTopicRoot();
  ingestTopic(root, message("hall/fader1", { payload: "0.1" }));
  ingestTopic(root, message("hall/fader1", { payload: "0.9", count: 999 }));
  const [fader] = flattenTopics(root);
  assert.deepEqual([fader.path, fader.last.payload, fader.count], ["hall/fader1", "0.9", 1000]);
  assert.equal(root.topics, 1);
});

test("50 000 topics draw as rows only as far as the tree is open, sorted once", () => {
  const root = makeTopicRoot();
  for (let n = 0; n < 50_000; n++) ingestTopic(root, message(`site/${n % 50}/dev${n}`));
  assert.equal(root.topics, 50_000);
  assert.deepEqual(visibleRows(root, new Set()).map((row) => [row.node.path, row.depth, row.parent]), [["site", 0, true]]);
  const open = visibleRows(root, new Set(["site", "site/7"]));
  assert.equal(open.length, 1 + 50 + 1000, "the root, its 50 children, the 1000 under site/7");
  assert.deepEqual(open.slice(0, 3).map((row) => row.node.path), ["site", "site/0", "site/1"]);
  assert.equal(open[open.findIndex((row) => row.node.path === "site/7") + 1].depth, 2);
  const site = root.children.get("site");
  assert.equal(sortedChildren(site), sortedChildren(site), "kept until a child is added");
  ingestTopic(root, message("site/new/x"));
  assert.ok(sortedChildren(site).some((node) => node.name === "new"), "and sorted again after");
});

test("a full tree takes messages for what it holds and refuses new topics", () => {
  const root = makeTopicRoot();
  root.topics = MAX_TOPICS - 1;
  assert.equal(ingestTopic(root, message("a/b")), true);
  assert.equal(ingestTopic(root, message("a/c")), false, "past the limit");
  assert.equal(root.children.get("a").children.has("c"), false, "nothing of it is kept");
  assert.equal(ingestTopic(root, message("a/b", { payload: "2" })), true, "a topic it holds still takes its latest");
  assert.equal(flattenTopics(root)[0].last.payload, "2");
});
