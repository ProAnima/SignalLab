import { useCallback, useState } from "react";

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
