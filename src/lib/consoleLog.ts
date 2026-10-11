/**
 * The console's lines as they are batched (pure). A storm of lines (hundreds
 * of jobs ending at once, a failure per request) is drawn at most every
 * `LOG_FLUSH_MS`, not once a line, and holds at most `LOG_CAPACITY` lines
 * however many arrive between two draws.
 */

/** Lines the console keeps; the oldest give way. */
export const LOG_CAPACITY = 500;
/** At most this often the console is drawn again while lines keep coming. */
export const LOG_FLUSH_MS = 100;

/** `prev` and then `lines`, the newest `capacity` of them. */
export function appendLines<T>(prev: readonly T[], lines: readonly T[], capacity = LOG_CAPACITY): T[] {
  if (lines.length === 0) return prev as T[];
  if (lines.length >= capacity) return lines.slice(lines.length - capacity);
  const keep = Math.max(0, prev.length + lines.length - capacity);
  return prev.slice(keep).concat(lines);
}

/**
 * Collects items and hands them to `flush` together, once per `delay`
 * milliseconds at most; what waits is capped at `capacity` (the newest kept).
 */
export function batcher<T>(flush: (items: T[]) => void, delay = LOG_FLUSH_MS, capacity = LOG_CAPACITY) {
  let pending: T[] = [];
  let timer: ReturnType<typeof setTimeout> | null = null;
  const run = () => {
    timer = null;
    const items = pending;
    pending = [];
    if (items.length) flush(items);
  };
  return {
    push(item: T) {
      pending.push(item);
      if (pending.length > capacity * 2) pending = pending.slice(pending.length - capacity);
      if (timer === null) timer = setTimeout(run, delay);
    },
    /** Hand over what waits now, and forget the timer. */
    flushNow() {
      if (timer !== null) clearTimeout(timer);
      run();
    },
    /** Drop what waits (the console was cleared). */
    clear() {
      pending = [];
    },
  };
}
