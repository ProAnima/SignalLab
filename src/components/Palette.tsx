import { useEffect, useMemo, useRef, useState } from "react";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { signalSummary, signalTarget, transportKey } from "../lib/signals";

/**
 * Ctrl+K anywhere: type a couple of letters, Enter, sent. The library view is
 * for building a signal; this is for the twentieth time you need it, mid-task,
 * without leaving the module you are watching.
 */
export function Palette() {
  const { library, fire } = useStore();
  const t = useT();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [cursor, setCursor] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((was) => !was);
        setQuery("");
        setCursor(0);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  useEffect(() => {
    if (open) inputRef.current?.focus();
  }, [open]);

  const matches = useMemo(() => {
    const q = query.trim().toLowerCase();
    const all = library;
    if (!q) return all.slice(0, 12);
    return all
      .filter((s) =>
        [s.name, s.group, signalTarget(s), signalSummary(s)].join(" ").toLowerCase().includes(q)
      )
      .slice(0, 12);
  }, [library, query]);

  if (!open) return null;

  const at = Math.min(cursor, Math.max(matches.length - 1, 0));

  const send = (index: number) => {
    const signal = matches[index];
    if (!signal) return;
    setOpen(false);
    void fire(signal);
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") { setOpen(false); return; }
    if (e.key === "ArrowDown") { e.preventDefault(); setCursor(Math.min(at + 1, matches.length - 1)); return; }
    if (e.key === "ArrowUp") { e.preventDefault(); setCursor(Math.max(at - 1, 0)); return; }
    if (e.key === "Enter") { e.preventDefault(); send(at); }
  };

  return (
    // Clicking the backdrop closes it; the dialog stops the click from reaching it.
    <div className="palette-backdrop" onClick={() => setOpen(false)}>
      <div
        className="palette"
        role="dialog"
        aria-modal="true"
        aria-label={t("sig.paletteTitle")}
        onClick={(e) => e.stopPropagation()}
      >
        <input
          ref={inputRef}
          value={query}
          placeholder={t("sig.paletteTitle")}
          onChange={(e) => { setQuery(e.target.value); setCursor(0); }}
          onKeyDown={onKeyDown}
        />
        <div className="palette-list">
          {matches.map((s, i) => (
            <button
              key={s.id}
              className={"palette-row" + (i === at ? " active" : "")}
              onMouseEnter={() => setCursor(i)}
              onClick={() => send(i)}
            >
              <span className="palette-name">{s.name}</span>
              <span className="sig-badge">{t(transportKey(s.body.transport))}</span>
              <span className="palette-target">{signalTarget(s)}</span>
            </button>
          ))}
          {matches.length === 0 && <div className="empty-state">{t("sig.paletteEmpty")}</div>}
        </div>
        <p className="palette-hint">{t("sig.paletteHint")}</p>
      </div>
    </div>
  );
}
