import { useCallback, useEffect, useReducer, useRef, useState } from "react";
import { api, type Experiment, type ProfileIssue } from "./api";
import { historyReducer, newHistory } from "./editHistory";
import type { Failure } from "./errors";

/**
 * Owns the editable document, its history and persistence lifecycle. Failures
 * are kept as they came from the engine and described where they are shown.
 */
export function useExperimentDocument() {
  const [history, dispatch] = useReducer(historyReducer<Experiment | null>, newHistory<Experiment | null>(null));
  const document = history.present;
  const [error, setError] = useState<Failure | null>(null);
  const [saveState, setSaveState] = useState<"saved" | "saving" | "error">("saved");
  const [validationError, setValidationError] = useState<Failure | null>(null);
  const [profileIssues, setProfileIssues] = useState<ProfileIssue[]>([]);
  // Validation also depends on the secret store, which is not part of the document.
  const [validationTick, setValidationTick] = useState(0);
  const revalidate = useCallback(() => setValidationTick((tick) => tick + 1), []);
  const queue = useRef<Promise<void>>(Promise.resolve());

  const save = useCallback((snapshot: Experiment): Promise<void> => {
    // A later snapshot must never be overwritten by an earlier slow write.
    const pending = queue.current.catch(() => {}).then(async () => { await api.experimentSave(snapshot); });
    queue.current = pending;
    return pending;
  }, []);

  useEffect(() => {
    let current = true;
    api.experimentLoad().then((loaded) => {
      if (current) dispatch({ type: "reset", document: loaded });
    }).catch((reason) => { if (current) setError(reason ?? "?"); });
    return () => { current = false; };
  }, []);

  useEffect(() => {
    if (!document) return;
    let current = true;
    setSaveState("saving");
    const timer = window.setTimeout(() => {
      save(document).then(() => { if (current) { setSaveState("saved"); setError(null); } })
        .catch((reason) => { if (current) { setSaveState("error"); setError(reason ?? "?"); } });
    }, 700);
    return () => { current = false; window.clearTimeout(timer); };
  }, [document, save]);

  useEffect(() => {
    if (!document) return;
    let current = true;
    const timer = window.setTimeout(() => {
      api.experimentValidate(document).then((issues) => { if (current) { setValidationError(null); setProfileIssues(issues ?? []); } })
        .catch((reason) => { if (current) { setValidationError(reason ?? "?"); setProfileIssues([]); } });
    }, 250);
    return () => { current = false; window.clearTimeout(timer); };
  }, [document, validationTick]);

  const replace = (next: Experiment) => {
    setError(null);
    if (!document) dispatch({ type: "reset", document: next });
    else dispatch({ type: "edit", update: () => next });
  };

  return { document, history, dispatch, save, replace, saveState, validationError, profileIssues, revalidate, error };
}
