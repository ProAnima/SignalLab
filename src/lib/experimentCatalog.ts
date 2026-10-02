import type { NodeType } from "./experimentGraph";
import type { TKey } from "./i18n";

export const NODE_CATALOG = {
  start: { title: "exp.node.start", description: "exp.description.start", group: "flow" },
  end: { title: "exp.node.end", description: "exp.description.end", group: "flow" },
  fork: { title: "exp.node.fork", description: "exp.description.fork", group: "flow" },
  join: { title: "exp.node.join", description: "exp.description.join", group: "flow" },
  http: { title: "exp.node.http", description: "exp.description.http", group: "action" },
  tcp: { title: "exp.node.tcp", description: "exp.description.tcp", group: "action" },
  osc: { title: "exp.node.osc", description: "exp.description.osc", group: "action" },
  udp: { title: "exp.node.udp", description: "exp.description.udp", group: "action" },
  mqtt: { title: "exp.node.mqtt", description: "exp.description.mqtt", group: "action" },
  log: { title: "exp.node.log", description: "exp.description.log", group: "action" },
  wait_osc: { title: "exp.node.wait_osc", description: "exp.description.wait_osc", group: "observe" },
  wait_udp: { title: "exp.node.wait_udp", description: "exp.description.wait_udp", group: "observe" },
  wait_mqtt: { title: "exp.node.wait_mqtt", description: "exp.description.wait_mqtt", group: "observe" },
  wait_http: { title: "exp.node.wait_http", description: "exp.description.wait_http", group: "observe" },
  emulator: { title: "exp.node.emulator", description: "exp.description.emulator", group: "emulate" },
  assert_status: { title: "exp.node.assert_status", description: "exp.description.assert_status", group: "check" },
  assert_body: { title: "exp.node.assert_body", description: "exp.description.assert_body", group: "check" },
  assert_header: { title: "exp.node.assert_header", description: "exp.description.assert_header", group: "check" },
  assert_latency: { title: "exp.node.assert_latency", description: "exp.description.assert_latency", group: "check" },
  delay: { title: "exp.node.delay", description: "exp.description.delay", group: "flow" },
  branch_status: { title: "exp.node.branch_status", description: "exp.description.branch_status", group: "flow" },
  extract: { title: "exp.node.extract", description: "exp.description.extract", group: "data" },
  assert_value: { title: "exp.node.assert_value", description: "exp.description.assert_value", group: "check" },
  branch_value: { title: "exp.node.branch_value", description: "exp.description.branch_value", group: "flow" },
  loop: { title: "exp.node.loop", description: "exp.description.loop", group: "flow" },
} satisfies Record<NodeType, { title: TKey; description: TKey; group: NodeGroup }>;

export type NodeGroup = "action" | "observe" | "emulate" | "data" | "check" | "flow";
/** Order of the add menu: send, then wait for the answer, play the other side, then work with it. */
export const NODE_GROUPS: NodeGroup[] = ["action", "observe", "emulate", "data", "check", "flow"];
export const ADDABLE_NODES = (Object.keys(NODE_CATALOG) as NodeType[]).filter(type => type !== "start" && type !== "end");
