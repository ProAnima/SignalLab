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

/**
 * The newest of `prior` and `added` (newest last) that fit both `maxCount`
 * items and `maxWeight` of `weight` — messages of 64 KB each would otherwise
 * hold hundreds of megabytes in a list of a few thousand. The newest is kept
 * whatever it weighs.
 */
export function keepNewest<T>(prior: T[], added: T[], maxCount: number, maxWeight: number, weight: (item: T) => number): T[] {
  const all = added.length ? prior.concat(added) : prior;
  let from = Math.max(0, all.length - maxCount);
  let held = 0;
  for (let i = all.length - 1; i >= from; i--) {
    held += weight(all[i]);
    if (held > maxWeight && i < all.length - 1) { from = i + 1; break; }
  }
  return from > 0 ? all.slice(from) : all;
}
