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
}

export function makeTopicRoot(): TopicNode {
  return { name: "", path: "", children: new Map(), last: null, count: 0, firstTs: 0 };
}

export function ingestTopic(root: TopicNode, m: MqttMessage): void {
  let node = root;
  for (const part of m.topic.split("/")) {
    const path = node.path ? `${node.path}/${part}` : part;
    let child = node.children.get(part);
    if (!child) {
      child = { name: part, path, children: new Map(), last: null, count: 0, firstTs: 0 };
      node.children.set(part, child);
    }
    node = child;
  }
  node.last = m;
  node.count += 1;
  if (node.firstTs === 0) node.firstTs = m.ts;
}

/** Every node that has ever carried a message — for search, and for counting. */
export function flattenTopics(node: TopicNode, into: TopicNode[] = []): TopicNode[] {
  if (node.last) into.push(node);
  for (const child of node.children.values()) flattenTopics(child, into);
  return into;
}

export function sortedChildren(node: TopicNode): TopicNode[] {
  return [...node.children.values()].sort((a, b) => a.name.localeCompare(b.name));
}
