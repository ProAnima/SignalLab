/** WebSocket helpers for the screen and the nodes (pure). */

/** "lab.v1, lab.v0" ⇄ ["lab.v1", "lab.v0"]: what a field shows and what a document keeps. */
export const protocolsText = (protocols: string[]) => protocols.join(", ");
export const protocolsOf = (text: string) => text.split(",").map((protocol) => protocol.trim()).filter(Boolean);

/** A message's text as formatted JSON, when it is a JSON object or array. */
export function asJson(text: string): string | null {
  const trimmed = text.trim();
  if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) return null;
  try { return JSON.stringify(JSON.parse(trimmed), null, 2); } catch { return null; }
}
