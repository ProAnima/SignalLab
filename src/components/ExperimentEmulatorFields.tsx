import { useEffect, useId, useRef, useState } from "react";
import type { Emulator } from "../lib/api";
import { useEmulators } from "../lib/emulatorStore";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { makeEmulatorId, protocolLabel, ruleCount } from "../lib/emulators";
import { EmulatorEditor } from "./EmulatorEditor";

/**
 * An Emulator node: what it plays in one line, its rules in a dialog wide
 * enough for them, and a way to and from the library — the same emulator the
 * Emulators screen runs on its own.
 */
export function ExperimentEmulatorFields({ emulator, onChange }: { emulator: Emulator; onChange: (next: Emulator) => void }) {
  const t = useT();
  const id = useId();
  const { pushLog } = useStore();
  const { emulators, setEmulators } = useEmulators();
  const [editing, setEditing] = useState(false);

  const fromLibrary = (wanted: string) => {
    const stored = emulators.find((item) => item.id === wanted);
    if (stored) onChange(structuredClone(stored.emulator));
  };
  const toLibrary = () => {
    const name = emulator.name.trim() || t(`emu.newName.${emulator.protocol}`);
    setEmulators([...emulators, { id: makeEmulatorId(name, emulators), note: "", emulator: structuredClone({ ...emulator, name }) }]);
    pushLog("ok", "emulators", "log.emulatorToLibrary", { name });
  };

  return <div className="experiment-emulator">
    <p className="experiment-emulator-summary">
      <b>{emulator.name}</b>
      <span>{t("emu.summary", { protocol: protocolLabel(emulator.protocol), bind: emulator.bind, n: ruleCount(emulator) })}</span>
    </p>
    <div className="btn-row tight">
      <button className="ghost sm" data-primary onClick={() => setEditing(true)}>{t("emu.edit")}</button>
      <button className="ghost sm" data-tip={t("emu.toLibraryHint")} onClick={toLibrary}>{t("emu.toLibrary")}</button>
    </div>
    {emulators.length > 0 && <label data-tip={t("emu.fromLibraryHint")}>{t("emu.fromLibrary")}
      <select id={`${id}library`} value="" onChange={(event) => { if (event.target.value) fromLibrary(event.target.value); }}>
        <option value="">—</option>
        {emulators.map((stored) => <option key={stored.id} value={stored.id}>{stored.emulator.name} · {protocolLabel(stored.emulator.protocol)} {stored.emulator.bind}</option>)}
      </select>
    </label>}
    {editing && <EmulatorDialog emulator={emulator} onChange={onChange} onClose={() => setEditing(false)} />}
  </div>;
}

function EmulatorDialog({ emulator, onChange, onClose }: { emulator: Emulator; onChange: (next: Emulator) => void; onClose: () => void }) {
  const t = useT();
  const id = useId();
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const element = dialog.current!;
    element.showModal();
    return () => element.close();
  }, []);
  return <dialog ref={dialog} className="save-signal-dialog emu-dialog" aria-labelledby={`${id}title`}
    onCancel={(event) => { event.preventDefault(); onClose(); }}>
    <div className="emu-dialog-body">
      <h2 id={`${id}title`} data-tip={t("exp.emulatorHint")}>{t("emu.editTitle")}</h2>
      <EmulatorEditor emulator={emulator} onChange={onChange} />
      <footer className="btn-row">
        <span className="spacer" />
        <button className="primary" onClick={onClose}>{t("emu.done")}</button>
      </footer>
    </div>
  </dialog>;
}
