import type { Frame } from "./api";

/** A captured frame, plus how many frames before it the view never got. */
export type CaptureRow = Frame & { gap?: number };

/**
 * The Inspector's list after a batch (pure): frames it already holds are not
 * added twice (the snapshot and the first batch overlap), at most `capacity`
 * are kept, and the first new frame says how many came between it and the
 * last one held. That count is read from the frames' numbers, which run
 * without holes: frames the engine could not ship (`skipped`) and frames that
 * arrived while the view was paused are both "not shown" — never a list that
 * looks continuous when it is not.
 */
export function appendFrames(prev: CaptureRow[], frames: Frame[], skipped: number, capacity: number): CaptureRow[] {
  const last = prev.length ? prev[prev.length - 1].seq : 0;
  const fresh = frames.filter((f) => f.seq > last);
  if (fresh.length === 0) return prev;
  const between = last > 0 ? fresh[0].seq - last - 1 : skipped;
  const gap = Math.max(between, skipped);
  const incoming: CaptureRow[] = gap > 0 ? fresh.map((f, i) => (i === 0 ? { ...f, gap } : f)) : fresh;
  const next = prev.concat(incoming);
  return next.length > capacity ? next.slice(next.length - capacity) : next;
}
