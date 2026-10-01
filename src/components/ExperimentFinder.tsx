import { useEffect, useRef, useState } from "react";
import type { ExperimentNode } from "../lib/api";
import { useT } from "../lib/i18n";

interface Props {
  nodes: ExperimentNode[];
  label: (type: ExperimentNode["type"]) => string;
  summary: (node: ExperimentNode) => string;
  onSelect: (node: ExperimentNode) => void;
  onClose: () => void;
}

export function ExperimentFinder({ nodes, label, summary, onSelect, onClose }: Props) {
  const t = useT();
  const [query, setQuery] = useState("");
  const [cursor, setCursor] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const selectionMade = useRef(false);
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  const matches = nodes.filter((node) => { const text = `${label(node.type)} ${summary(node)} ${node.id}`.toLowerCase(); return words.every((word) => text.includes(word)); });
  const at = Math.min(cursor, Math.max(0, matches.length - 1));

  useEffect(() => {
    const previous = document.activeElement;
    input.current?.focus();
    return () => { if (!selectionMade.current && previous instanceof HTMLElement && previous.isConnected) previous.focus({ preventScroll: true }); };
  }, []);

  useEffect(() => { list.current?.children[at]?.scrollIntoView({ block: "nearest" }); }, [at]);
  const choose = (node: ExperimentNode) => { selectionMade.current = true; onSelect(node); };

  return <div className="palette-backdrop" onPointerDown={onClose}>
    <div className="palette experiment-finder" role="dialog" aria-modal="true" aria-label={t("exp.findNode")}
      onPointerDown={(event) => event.stopPropagation()} onKeyDown={(event) => {
        event.stopPropagation();
        if (event.key === "Escape") { event.preventDefault(); onClose(); }
        if (event.key === "Tab") { event.preventDefault(); input.current?.focus(); }
        if (event.key === "ArrowDown") { event.preventDefault(); setCursor(Math.min(at + 1, matches.length - 1)); }
        if (event.key === "ArrowUp") { event.preventDefault(); setCursor(Math.max(at - 1, 0)); }
        if (event.key === "Enter" && matches[at]) { event.preventDefault(); choose(matches[at]); }
      }}>
      <input ref={input} role="combobox" aria-expanded="true" aria-controls="experiment-node-list"
        aria-activedescendant={matches[at] ? `node-option-${matches[at].id}` : undefined}
        aria-label={t("exp.findNode")} placeholder={t("exp.findNode")} value={query}
        onChange={(event) => { setQuery(event.target.value); setCursor(0); }} />
      <div className="palette-list" ref={list} role="listbox" id="experiment-node-list" aria-label={t("exp.findNode")}>
        {matches.map((node, index) => <button key={node.id} id={`node-option-${node.id}`} role="option" aria-selected={index === at} tabIndex={-1}
          className={`palette-row ${index === at ? "active" : ""}`} onClick={() => choose(node)} onMouseEnter={() => setCursor(index)}>
          <span className="experiment-finder-index">{nodes.indexOf(node) + 1}</span><span className="palette-name">{label(node.type)}</span>
          <span className="palette-target">{summary(node)}</span>
        </button>)}
        {!matches.length && <div className="empty-state">{t("exp.noMatchingNodes")}</div>}
      </div>
      <p className="palette-hint">{t("exp.findHint")}</p>
    </div>
  </div>;
}
