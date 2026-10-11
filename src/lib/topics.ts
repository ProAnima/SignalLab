/**
 * The topic index behind the MQTT tree.
 *
 * It is a mutable index rather than React state on purpose: a `#` scan of a real
 * broker is thousands of nodes, and rebuilding an immutable tree per batch would
 * spend the whole frame budget copying. Callers bump a version counter to tell
 * React that something inside changed.
 *
 * It lives in the store rather than in the view because it is the state of the
 * broker, not of a screen — walking away from the tab must not forget what the
 * broker holds, and a retained value only arrives once, on subscribe.
 */
import type { MqttMessage } from "./api";

export interface TopicNode {
  name: string;
  /** Full topic path, which is also the node's identity in the UI. */
  path: string;
  children: Map<string, TopicNode>;
  /** The most recent message on exactly this topic, if any ever arrived. */
  last: MqttMessage | null;
  count: number;
  firstTs: number;
  /** `children` in order, kept until a child is added (`sortedChildren`). */
  sorted?: TopicNode[] | null;
  /** On the root: topics that carry a message, at most `MAX_TOPICS`. */
  topics?: number;
}

/**
 * Topics the tree holds at most. Each keeps its latest message (4 KB of it at
 * most), so this bounds the screen's memory however many topics a broker has;
 * messages on topics past it are counted as not shown.
 */
export const MAX_TOPICS = 100_000;

export function makeTopicRoot(): TopicNode {
  return { name: "", path: "", children: new Map(), last: null, count: 0, firstTs: 0, sorted: null, topics: 0 };
}

/** Files `m` under its topic; false when the tree is full and it is a topic it does not hold. */
export function ingestTopic(root: TopicNode, m: MqttMessage): boolean {
  const parts = m.topic.split("/");
  if ((root.topics ?? 0) >= MAX_TOPICS) {
    // Full: only a topic already held takes the message.
    let node: TopicNode | undefined = root;
    for (const part of parts) node = node?.children.get(part);
    if (!node?.last) return false;
  }
  let node = root;
  for (const part of parts) {
    let child = node.children.get(part);
    if (!child) {
      const path = node.path ? `${node.path}/${part}` : part;
      child = { name: part, path, children: new Map(), last: null, count: 0, firstTs: 0, sorted: null };
      node.children.set(part, child);
      node.sorted = null;
    }
    node = child;
  }
  if (!node.last) root.topics = (root.topics ?? 0) + 1;
  node.last = m;
  node.count += m.count ?? 1;
  if (node.firstTs === 0) node.firstTs = m.ts;
  return true;
}

/** Every node that has ever carried a message — for search, and for counting. */
export function flattenTopics(node: TopicNode, into: TopicNode[] = []): TopicNode[] {
  if (node.last) into.push(node);
  for (const child of node.children.values()) flattenTopics(child, into);
  return into;
}

const collator = new Intl.Collator();

/** A node's children by name; sorted once, again only after a child is added. */
export function sortedChildren(node: TopicNode): TopicNode[] {
  node.sorted ??= [...node.children.values()].sort((a, b) => collator.compare(a.name, b.name));
  return node.sorted;
}

/** One line of the tree as drawn: a node, how deep, whether it has children. */
export interface TopicRow { node: TopicNode; depth: number; parent: boolean }

/** The lines the tree shows: the roots, and the children of every node in `expanded`, depth first. */
export function visibleRows(root: TopicNode, expanded: ReadonlySet<string>): TopicRow[] {
  const rows: TopicRow[] = [];
  const walk = (node: TopicNode, depth: number) => {
    for (const child of sortedChildren(node)) {
      const parent = child.children.size > 0;
      rows.push({ node: child, depth, parent });
      if (parent && expanded.has(child.path)) walk(child, depth + 1);
    }
  };
  walk(root, 0);
  return rows;
}
