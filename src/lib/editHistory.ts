/** Immutable document history. A drag or field edit shares one group until committed. */
export interface EditHistory<T> {
  past: T[];
  present: T;
  future: T[];
  group: string | null;
}

export type HistoryAction<T> =
  | { type: "reset"; document: T }
  | { type: "edit"; update: (document: T) => T; group?: string | null }
  | { type: "commit" | "undo" | "redo" };

export const HISTORY_LIMIT = 100;

export function newHistory<T>(document: T): EditHistory<T> {
  return { past: [], present: document, future: [], group: null };
}

export function historyReducer<T>(state: EditHistory<T>, action: HistoryAction<T>): EditHistory<T> {
  switch (action.type) {
    case "reset": return newHistory(action.document);
    case "commit": return state.group === null ? state : { ...state, group: null };
    case "edit": {
      const present = action.update(state.present);
      if (present === state.present) return state;
      const group = action.group ?? null;
      const merged = group !== null && group === state.group;
      return { present, group, future: [], past: merged ? state.past : [...state.past, state.present].slice(-HISTORY_LIMIT) };
    }
    case "undo": {
      if (!state.past.length) return state;
      return { present: state.past[state.past.length - 1], past: state.past.slice(0, -1),
        future: [state.present, ...state.future], group: null };
    }
    case "redo": {
      if (!state.future.length) return state;
      return { present: state.future[0], past: [...state.past, state.present].slice(-HISTORY_LIMIT),
        future: state.future.slice(1), group: null };
    }
  }
}
