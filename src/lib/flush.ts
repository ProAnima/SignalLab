/**
 * Writes that wait for a debounce — the signal library, the emulators, the
 * experiment document — say here how to finish now. `flushAll` makes them
 * before the app goes away under them: an update installs and restarts it.
 */

type Flush = () => Promise<unknown> | void;

const flushes = new Set<Flush>();

/** Register a pending write's "now"; the returned function unregisters it. */
export function onFlush(flush: Flush): () => void {
  flushes.add(flush);
  return () => { flushes.delete(flush); };
}

/** Every pending write, made now; done when all are written or failed, or after `timeoutMs`. */
export async function flushAll(timeoutMs = 5000): Promise<void> {
  const all = Promise.allSettled([...flushes].map((flush) => Promise.resolve().then(flush)));
  await Promise.race([all, new Promise((resolve) => setTimeout(resolve, timeoutMs))]);
}
