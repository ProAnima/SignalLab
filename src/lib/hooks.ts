import { useCallback, useEffect, useRef, useState } from "react";
import { on } from "./api";

/** Held events, bounded: a firehose must not become a memory leak. */
const PENDING_CAP = 4000;

/**
 * Subscribe to a job's telemetry, keyed by job id.
 *
 * The id only exists once the start command resolves, but the engine emits from
 * the moment it is spawned — and a loopback port scan can finish before React
 * has re-rendered with the id at all. So events that arrive before the id is
 * known are held and replayed once it lands, rather than dropped, which is what
 * made a finished scan show an empty table.
 */
export function useJobStream<T extends { job_id: number }>(
  event: string,
  jobId: number | null,
  handler: (payload: T) => void
) {
  // The handler is a new closure every render; the listener must not be.
  const cb = useRef(handler);
  cb.current = handler;
  const idRef = useRef<number | null>(jobId);
  const pending = useRef<T[]>([]);

  useEffect(() => {
    idRef.current = jobId;
    const held = pending.current;
    pending.current = [];
    if (jobId === null) return;
    for (const p of held) if (p.job_id === jobId) cb.current(p);
  }, [jobId]);

  useEffect(() => {
    const un = on<T>(event, (e) => {
      const id = idRef.current;
      if (id === null) {
        if (pending.current.length < PENDING_CAP) pending.current.push(e.payload);
        return;
      }
      if (e.payload.job_id === id) cb.current(e.payload);
    });
    return () => { un.then((f) => f()); };
  }, [event]);
}


/** A fixed-capacity rolling numeric series for live charts. */
export function useSeries(capacity = 240) {
  const [data, setData] = useState<number[]>([]);
  const push = useCallback(
    (v: number) => {
      setData((prev) => {
        const next = prev.length >= capacity ? prev.slice(prev.length - capacity + 1) : prev.slice();
        next.push(v);
        return next;
      });
    },
    [capacity]
  );
  const clear = useCallback(() => setData([]), []);
  return { data, push, clear };
}

/** A capped rolling list of items (newest last). */
export function useRollingList<T>(capacity = 300) {
  const [items, setItems] = useState<T[]>([]);
  const push = useCallback(
    (v: T) => {
      setItems((prev) => {
        const next = prev.length >= capacity ? prev.slice(prev.length - capacity + 1) : prev.slice();
        next.push(v);
        return next;
      });
    },
    [capacity]
  );
  const clear = useCallback(() => setItems([]), []);
  return { items, push, clear };
}
