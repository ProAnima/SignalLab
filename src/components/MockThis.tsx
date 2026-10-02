import { useEffect, useId, useRef, useState } from "react";
import type { HttpResponse, StoredEmulator } from "../lib/api";
import { useStore } from "../lib/store";
import { useEmulators } from "../lib/emulatorStore";
import { useT } from "../lib/i18n";
import { blankEmulator, freeBind, makeEmulatorId, routeFromResponse } from "../lib/emulators";

/**
 * "Mock this": a response that worked becomes a route of an HTTP emulator
 * that answers exactly so — creation starts from something that worked.
 * `onShow` opens the emulator it went into.
 */
export function MockThis({ method, url, response, onShow }: { method: string; url: string; response: HttpResponse; onShow?: (id: string) => void }) {
  const t = useT();
  const [asking, setAsking] = useState(false);
  return <>
    <button className="ghost sm" data-tip={t("http.mockThisHint")} onClick={() => setAsking(true)}>⧉ {t("http.mockThis")}</button>
    {asking && <MockDialog method={method} url={url} response={response} onClose={() => setAsking(false)}
      onDone={(id) => { setAsking(false); onShow?.(id); }} />}
  </>;
}

function MockDialog({ method, url, response, onClose, onDone }: {
  method: string; url: string; response: HttpResponse; onClose: () => void; onDone: (id: string) => void;
}) {
  const t = useT();
  const { pushLog } = useStore();
  const { emulators, setEmulators } = useEmulators();
  const apis = emulators.filter((stored) => stored.emulator.protocol === "http");
  const [target, setTarget] = useState(apis[0]?.id ?? "new");
  const dialog = useRef<HTMLDialogElement>(null);
  const id = useId();
  const route = routeFromResponse(method, url, response);

  useEffect(() => {
    const element = dialog.current!;
    element.showModal();
    return () => element.close();
  }, []);

  const add = () => {
    let into: StoredEmulator;
    if (target === "new") {
      const name = t("emu.newName.http");
      const emulator = { ...blankEmulator("http", name, freeBind("http", emulators)), routes: [route] };
      into = { id: makeEmulatorId(name, emulators), note: "", emulator };
      setEmulators([...emulators, into]);
    } else {
      const found = emulators.find((stored) => stored.id === target);
      if (!found || found.emulator.protocol !== "http") return;
      // First, so it answers before a broader route of the same path does.
      into = { ...found, emulator: { ...found.emulator, routes: [route, ...found.emulator.routes] } };
      setEmulators(emulators.map((stored) => stored.id === target ? into : stored));
    }
    pushLog("ok", "emulators", "log.mockAdded", { method: route.method, path: route.path, name: into.emulator.name });
    onDone(into.id);
  };

  return <dialog ref={dialog} className="save-signal-dialog" aria-labelledby={`${id}title`}
    onCancel={(event) => { event.preventDefault(); onClose(); }} onClick={(event) => { if (event.target === event.currentTarget) onClose(); }}>
    <form method="dialog" onSubmit={(event) => { event.preventDefault(); add(); }}>
      <h2 id={`${id}title`}>{t("http.mockTitle")}</h2>
      <p className="emu-mock-route"><code>{route.method} {route.path} → {route.responses[0].status}</code></p>
      <div className="field">
        <label htmlFor={`${id}target`}>{t("http.mockInto")}</label>
        <select id={`${id}target`} value={target} onChange={(event) => setTarget(event.target.value)}>
          {apis.map((stored) => <option key={stored.id} value={stored.id}>{stored.emulator.name} · {stored.emulator.bind}</option>)}
          <option value="new">{t("http.mockNew")}</option>
        </select>
      </div>
      <footer className="btn-row">
        <button type="button" className="ghost" onClick={onClose}>{t("http.mockCancel")}</button>
        <button type="submit" className="primary">{t("http.mockAdd")}</button>
      </footer>
    </form>
  </dialog>;
}
