import { useEffect, useMemo, useState } from "react";
import type { ExperimentNode, Signal } from "../lib/api";
import { ADDABLE_NODES, NODE_CATALOG, NODE_GROUPS } from "../lib/experimentCatalog";
import type { NodeType } from "../lib/experimentGraph";
import { anchorLabel, GROUP_GLYPH, nodeLabel } from "../lib/experimentText";
import type { Menu } from "../lib/useExperimentCanvas";
import { nodeFromSignal, signalTarget, transportKey } from "../lib/signals";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";

/** What the add menu offers: a kind of node, or a library signal as a ready action. */
export type MenuItem = { kind: "node"; type: NodeType } | { kind: "signal"; signal: Signal };

/**
 * The add menu: type to filter, arrows and Enter to pick. Nodes by group,
 * then the library's signals that can become an action.
 */
export function ExperimentAddMenu({ menu, nodes, onAdd, onClose }: {
  menu: Menu;
  /** The document's nodes, to name the output the new node attaches to. */
  nodes: ExperimentNode[];
  onAdd: (item: MenuItem) => void;
  onClose: () => void;
}) {
  const t = useT();
  const { library } = useStore();
  const [search, setSearch] = useState("");
  const [menuIndex, setMenuIndex] = useState(0);

  // The search field is focused by autoFocus as it mounts, so typing right after A is never lost.
  useEffect(() => { setSearch(""); setMenuIndex(0); }, [menu]);

  const menuItems = useMemo((): MenuItem[] => {
    const query = search.trim().toLowerCase();
    // Every word must appear somewhere: "check val" finds "Check value".
    const words = query.split(/\s+/).filter(Boolean);
    const matches = (text: string) => { const lower = text.toLowerCase(); return words.every((word) => lower.includes(word)); };
    const nodes: MenuItem[] = NODE_GROUPS.flatMap((group) => ADDABLE_NODES
      .filter((type) => NODE_CATALOG[type].group === group && matches(`${t(NODE_CATALOG[type].title)} ${t(NODE_CATALOG[type].description)} ${type}`))
      .map((type): MenuItem => ({ kind: "node", type })));
    const signals: MenuItem[] = library
      .filter((signal) => nodeFromSignal(signal.body, 0, 0) && matches(`${signal.name} ${signal.group} ${signal.body.transport} ${signalTarget(signal)}`))
      .map((signal): MenuItem => ({ kind: "signal", signal }));
    return [...nodes, ...signals];
  }, [search, library, t]);

  let menuCursor = 0;
  const menuButton = (item: MenuItem, group: string, name: string, description: string, glyph: string) => {
    const index = menuCursor++;
    const id = item.kind === "node" ? `catalog-${item.type}` : `catalog-signal-${item.signal.id}`;
    return <button key={id} id={id} className={menuIndex === index ? "highlighted" : ""} data-group={group} data-tip={description} onClick={() => onAdd(item)} onMouseEnter={() => setMenuIndex(index)}>
      <span className="experiment-kind">{glyph}</span><span><b>{name}</b></span></button>;
  };

  return <div className="experiment-menu-backdrop" onPointerDown={onClose}><div className="experiment-add-menu" role="dialog" aria-label={t("exp.addNode")} style={{ left: Math.max(8, menu.screenX), top: Math.max(8, menu.screenY) }} onPointerDown={(event) => event.stopPropagation()}>
    {menu.anchor && <p className="experiment-menu-context">{t(menu.branch ? "exp.addingBranch" : "exp.addingAfter", { node: anchorLabel(nodes, menu.anchor, t) })}</p>}
    <input autoFocus value={search} onChange={(event) => { setSearch(event.target.value); setMenuIndex(0); }} placeholder={t("exp.searchNodes")} aria-label={t("exp.searchNodes")} data-tip={t("exp.menuHint")} aria-controls="experiment-catalog" onKeyDown={(event) => {
      if (event.key === "Escape") { event.preventDefault(); onClose(); return; }
      if (event.key === "Enter" && menuItems[menuIndex]) { event.preventDefault(); onAdd(menuItems[menuIndex]); }
      if (["ArrowDown", "ArrowUp"].includes(event.key) && menuItems.length) { event.preventDefault(); const next = (menuIndex + (event.key === "ArrowDown" ? 1 : -1) + menuItems.length) % menuItems.length; setMenuIndex(next); const item = menuItems[next]; document.getElementById(item.kind === "node" ? `catalog-${item.type}` : `catalog-signal-${item.signal.id}`)?.scrollIntoView({ block: "nearest" }); } }} />
    <div id="experiment-catalog" className="experiment-catalog-results">{NODE_GROUPS.map(group => {
      const items = menuItems.filter((item) => item.kind === "node" && NODE_CATALOG[item.type].group === group);
      return items.length ? <section key={group}><h3>{t(`exp.group.${group}`)}</h3>{items.map((item) => item.kind === "node" && menuButton(item, group, nodeLabel(item.type, t), t(NODE_CATALOG[item.type].description), GROUP_GLYPH[group]))}</section> : null;
    })}
      {menuItems.some((item) => item.kind === "signal") && <section><h3>{t("exp.group.signals")}</h3>{menuItems.map((item) => item.kind === "signal" && menuButton(item, "action", item.signal.name, `${t(transportKey(item.signal.body.transport))} · ${signalTarget(item.signal)}`, "❖"))}</section>}
      {!menuItems.length && <p className="experiment-empty">{t("exp.noMatches")}</p>}</div>
  </div></div>;
}
