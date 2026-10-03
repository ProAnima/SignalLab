import { useEffect, useMemo, useState } from "react";
import type { HttpRequest, Signal, SignalBody } from "../lib/api";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { useFieldIds } from "../lib/hooks";
import { describeError } from "../lib/errors";
import { fmtTime } from "../lib/format";
import { OscArgsEditor, fromOscArg, toOscArg, type ArgRow } from "../components/OscArgs";
import { HttpAuthFields } from "../components/HttpAuthFields";
import { SignalTree } from "../components/SignalTree";
import { addFolder, allFolders, freeFolderName, joinFolder, normalizeFolder } from "../lib/library";
import {
  TRANSPORTS, blankBody, makeId, signalSummary, signalTarget, transportKey,
  type Transport,
} from "../lib/signals";

const METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"];

/** The screens a stored signal can be opened in, to edit and send it there. */
export type OpenableTransport = "http" | "osc" | "mqtt";
const OPENS_IN: Record<string, "nav.http" | "nav.osc" | "nav.mqtt" | undefined> = { http: "nav.http", osc: "nav.osc", mqtt: "nav.mqtt" };

/**
 * `onOpen`: load a signal into the screen of its transport. `reveal`: a signal
 * to select and show (from a sender's chip); `at` makes a repeat a new request.
 */
export function SignalsView({ onOpen, reveal }: { onOpen?: (signal: Signal) => void; reveal?: { id: string; at: number } | null } = {}) {
  const { library, folders, setLibraryState, libraryPath, libraryError, saveState, setLibrary, reloadLibrary, fire, lastFired } =
    useStore();
  const t = useT();
  const fid = useFieldIds();

  const [selectedId, setSelectedId] = useState<string | null>(null);
  // Where "New signal" and "New folder" put things: the folder last picked.
  const [folder, setFolder] = useState("");
  const [query, setQuery] = useState("");
  const [confirmDelete, setConfirmDelete] = useState(false);
  // Argument rows are edited as text, so a half-typed float stays half-typed
  // instead of being rounded back at the user on every keystroke.
  const [args, setArgs] = useState<ArgRow[]>([]);

  const selected = library.find((s) => s.id === selectedId) ?? null;

  // Reload the argument rows only when the selection itself changes.
  useEffect(() => {
    setConfirmDelete(false);
    if (!selected) { setArgs([]); return; }
    setArgs(selected.body.transport === "osc" ? selected.body.args.map(fromOscArg) : []);
  }, [selectedId]); // eslint-disable-line react-hooks/exhaustive-deps

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return library;
    return library.filter((s) =>
      [s.name, s.group, s.note, signalTarget(s), signalSummary(s)]
        .join(" ")
        .toLowerCase()
        .includes(q)
    );
  }, [library, query]);

  const everyFolder = useMemo(() => allFolders(library, folders), [library, folders]);

  useEffect(() => {
    if (!reveal) return;
    const signal = library.find((s) => s.id === reveal.id);
    if (!signal) return;
    setQuery(""); setSelectedId(signal.id); setFolder(normalizeFolder(signal.group));
  }, [reveal]); // eslint-disable-line react-hooks/exhaustive-deps

  const select = (id: string) => {
    setSelectedId(id);
    const signal = library.find((s) => s.id === id);
    if (signal) setFolder(normalizeFolder(signal.group));
  };

  const newFolder = () => {
    const name = freeFolderName(library, folders, folder, t("sig.newFolderName"));
    const path = joinFolder(folder, name);
    setLibraryState(addFolder({ signals: library, folders }, path));
    setFolder(path);
    // Straight into renaming it, as a file manager does.
    requestAnimationFrame(() => {
      const row = document.querySelector<HTMLButtonElement>(`[data-folder="${CSS.escape(path)}"]`);
      row?.dispatchEvent(new KeyboardEvent("keydown", { key: "F2", bubbles: true }));
    });
  };

  const replace = (next: Signal) => setLibrary(library.map((s) => (s.id === next.id ? next : s)));

  const patch = (fields: Partial<Signal>) => {
    if (!selected) return;
    replace({ ...selected, ...fields });
  };

  const patchBody = (body: SignalBody) => patch({ body });

  /** Argument rows are the source of truth while editing; the model follows. */
  const setArgRows = (rows: ArgRow[]) => {
    setArgs(rows);
    if (selected?.body.transport === "osc") {
      patchBody({ ...selected.body, args: rows.map(toOscArg) });
    }
  };

  const changeTransport = (transport: Transport) => {
    if (!selected || selected.body.transport === transport) return;
    const body = blankBody(transport);
    // Carry the target only between OSC and raw UDP, where host:port means the
    // same thing. A broker or a URL is a different address, and pre-filling it
    // with the old one just looks configured when it isn't.
    const from = selected.body;
    if ((body.transport === "osc" || body.transport === "udp")
      && (from.transport === "osc" || from.transport === "udp")) {
      body.target = from.target;
    }
    setArgs(body.transport === "osc" ? [] : args);
    patchBody(body);
  };

  const create = () => {
    const name = t("sig.newName");
    const signal: Signal = {
      id: makeId(name, library),
      name,
      group: folder,
      note: "",
      body: blankBody("osc"),
    };
    setLibrary([...library, signal]);
    setSelectedId(signal.id);
  };

  const duplicate = () => {
    if (!selected) return;
    const name = `${selected.name} ·`;
    const copy: Signal = { ...selected, id: makeId(name, library), name };
    const at = library.findIndex((s) => s.id === selected.id);
    const next = library.slice();
    next.splice(at + 1, 0, copy);
    setLibrary(next);
    setSelectedId(copy.id);
  };

  const remove = () => {
    if (!selected) return;
    // Its folder stays, as a file manager keeps a folder whose last file went.
    setLibraryState(addFolder({ signals: library.filter((s) => s.id !== selected.id), folders }, selected.group));
    setSelectedId(null);
  };

  // The one shortcut worth having here: edit a field, fire without reaching for
  // the mouse.
  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey) && selected) {
      e.preventDefault();
      void fire(selected);
    }
  };

  const body = selected?.body;

  return (
    <div onKeyDown={onKeyDown}>
      <div className="view-head">
        <h1 data-tip={`${t("sig.blurb")}\nCtrl+K ${t("sig.paletteOpen")}`}>{t("sig.title")}</h1>
      </div>

      <div className="cols side">
        {/* ---- explorer ---- */}
        <div className="panel" style={{ display: "flex", flexDirection: "column", minHeight: 420 }}>
          <p className="section-label">{t("sig.library")}</p>
          <div className="row" style={{ marginBottom: 8 }}>
            <input
              value={query}
              aria-label={t("sig.search")}
              placeholder={t("sig.search")}
              onChange={(e) => setQuery(e.target.value)}
            />
            <button className="ghost sm" style={{ flex: "0 0 auto" }} onClick={create} data-tip={t("sig.newHint")}>
              {t("sig.new")}
            </button>
          </div>

          <SignalTree signals={filtered} searching={query.trim() !== ""} selectedId={selectedId} onSelect={select} onFire={(s) => void fire(s)}
            folder={folder} onFolder={setFolder} reveal={reveal?.id ?? null} onNewFolder={newFolder} />
          {filtered.length === 0 && (library.length > 0 || !everyFolder.length) && (
            <div className="empty-state">
              {libraryError ? describeError(libraryError, t).text : library.length === 0 ? t("sig.empty") : t("sig.emptyFiltered")}
            </div>
          )}

          <div className="sig-foot">
            <span data-tip={libraryPath}>{t("sig.count", { n: library.length })}</span>
            <span className="spacer" />
            {saveState === "saving" && <span>{t("sig.saving")}</span>}
            {saveState === "saved" && <span className="ok">{t("sig.saved")}</span>}
            {/* Always offered: the file is meant to be edited by hand and swapped
                between machines, and the app has no way to notice that itself. */}
            <button className="ghost sm" onClick={reloadLibrary} data-tip={libraryPath}>
              {t("sig.reload")}
            </button>
          </div>
        </div>

        {/* ---- editor ---- */}
        <div className="panel">
          {!selected && <div className="empty-state">{t("sig.pick")}</div>}
          {selected && body && (
            <>
              <div className="row">
                <div className="field">
                  <label htmlFor="sig-name">{t("sig.name")}</label>
                  <input
                    id="sig-name"
                    value={selected.name}
                    onChange={(e) => patch({ name: e.target.value })}
                  />
                </div>
                <div className="field">
                  <label htmlFor="sig-group" data-tip={t("sig.folderHint")}>{t("sig.group")}</label>
                  <input
                    id="sig-group"
                    list="sig-groups"
                    value={selected.group}
                    placeholder={t("sig.topLevel")}
                    onChange={(e) => patch({ group: e.target.value })}
                    onBlur={() => { const clean = normalizeFolder(selected.group); if (clean !== selected.group) patch({ group: clean }); setFolder(clean); }}
                  />
                  <datalist id="sig-groups">
                    {everyFolder.map((g) => <option key={g} value={g} />)}
                  </datalist>
                </div>
                <div className="field" style={{ flex: "0 0 132px" }}>
                  <label htmlFor="sig-transport">{t("sig.transport")}</label>
                  <select
                    id="sig-transport"
                    value={body.transport}
                    onChange={(e) => changeTransport(e.target.value as Transport)}
                  >
                    {TRANSPORTS.map((tr) => (
                      <option key={tr} value={tr}>{t(transportKey(tr))}</option>
                    ))}
                  </select>
                </div>
              </div>

              {body.transport === "osc" && (
                <>
                  <div className="row">
                    <div className="field">
                      <label htmlFor={fid("osc-target")} data-tip={t("common.targetHint")}>{t("common.target")}</label>
                      <input id={fid("osc-target")}
                        value={body.target}
                        onChange={(e) => patchBody({ ...body, target: e.target.value })}
                      />
                    </div>
                    <div className="field">
                      <label htmlFor={fid("osc-address")} data-tip={t("common.addressHint")}>{t("common.address")}</label>
                      <input id={fid("osc-address")}
                        value={body.address}
                        onChange={(e) => patchBody({ ...body, address: e.target.value })}
                      />
                    </div>
                  </div>
                  <div className="field">
                    <label data-tip={t("common.argumentsHint")}>{t("common.arguments")}</label>
                    <OscArgsEditor args={args} onChange={setArgRows} onSubmit={() => void fire(selected)} />
                  </div>
                </>
              )}

              {body.transport === "udp" && (
                <>
                  <div className="row">
                    <div className="field">
                      <label htmlFor={fid("udp-target")} data-tip={t("common.targetHint")}>{t("common.target")}</label>
                      <input id={fid("udp-target")}
                        value={body.target}
                        onChange={(e) => patchBody({ ...body, target: e.target.value })}
                      />
                    </div>
                    <div className="field" style={{ flex: "0 0 132px" }}>
                      <label htmlFor="sig-kind">{t("sig.payloadKind")}</label>
                      <select
                        id="sig-kind"
                        value={body.payload.kind}
                        onChange={(e) =>
                          patchBody({
                            ...body,
                            payload: e.target.value === "hex" ? { kind: "hex", hex: "" } : { kind: "text", text: "" },
                          })
                        }
                      >
                        <option value="text">{t("sig.payloadText")}</option>
                        <option value="hex">{t("sig.payloadHex")}</option>
                      </select>
                    </div>
                  </div>
                  <div className="field">
                    <label htmlFor="sig-payload">{t("sig.payload")}</label>
                    <textarea
                      id="sig-payload"
                      value={body.payload.kind === "hex" ? body.payload.hex : body.payload.text}
                      onChange={(e) =>
                        patchBody({
                          ...body,
                          payload: body.payload.kind === "hex"
                            ? { kind: "hex", hex: e.target.value }
                            : { kind: "text", text: e.target.value },
                        })
                      }
                    />
                  </div>
                </>
              )}

              {body.transport === "mqtt" && (
                <>
                  <div className="row">
                    <div className="field">
                      <label htmlFor={fid("mqtt-broker")}>{t("mq.broker")}</label>
                      <input id={fid("mqtt-broker")}
                        value={body.broker}
                        onChange={(e) => patchBody({ ...body, broker: e.target.value })}
                      />
                    </div>
                    <div className="field">
                      <label htmlFor={fid("mqtt-topic")}>{t("mq.topic")}</label>
                      <input id={fid("mqtt-topic")}
                        value={body.topic}
                        onChange={(e) => patchBody({ ...body, topic: e.target.value })}
                      />
                    </div>
                    <div className="field" style={{ flex: "0 0 88px" }}>
                      <label htmlFor="sig-qos" data-tip={t("mq.qosHint")}>{t("mq.qos")}</label>
                      <select
                        id="sig-qos"
                        value={body.qos}
                        onChange={(e) => patchBody({ ...body, qos: +e.target.value })}
                      >
                        {[0, 1, 2].map((q) => <option key={q} value={q}>{q}</option>)}
                      </select>
                    </div>
                    <div className="field check" style={{ flex: "0 0 108px" }}>
                      <label className="checkbox" data-tip={t("mq.emptyClears")}>
                        <input
                          type="checkbox"
                          checked={body.retain}
                          onChange={(e) => patchBody({ ...body, retain: e.target.checked })}
                        />
                        {t("mq.retain")}
                      </label>
                    </div>
                  </div>
                  <div className="field">
                    <label htmlFor="sig-mqtt-payload">{t("sig.payload")}</label>
                    <textarea
                      id="sig-mqtt-payload"
                      value={body.payload}
                      onChange={(e) => patchBody({ ...body, payload: e.target.value })}
                    />
                  </div>
                </>
              )}

              {body.transport === "http" && (
                <HttpEditor
                  request={body.request}
                  onChange={(request) => patchBody({ ...body, request })}
                />
              )}

              <div className="field">
                <label htmlFor="sig-note" data-tip={t("sig.notePlaceholder")}>{t("sig.note")}</label>
                <textarea
                  id="sig-note"
                  value={selected.note}
                  onChange={(e) => patch({ note: e.target.value })}
                />
              </div>

              <div className="btn-row">
                <button className="primary" onClick={() => void fire(selected)} data-tip={t("sig.fireHint")}>
                  {t("sig.fire")}
                </button>
                {onOpen && OPENS_IN[selected.body.transport] && (
                  <button className="ghost" onClick={() => onOpen(selected)} data-tip={t("sig.openInHint", { screen: t(OPENS_IN[selected.body.transport]!) })}>
                    {t("sig.openIn", { screen: t(OPENS_IN[selected.body.transport]!) })}
                  </button>
                )}
                <button className="ghost" onClick={duplicate}>{t("sig.duplicate")}</button>
                {/* Two steps, because the file is written the moment you click. */}
                <button
                  className="danger"
                  onClick={() => (confirmDelete ? remove() : setConfirmDelete(true))}
                >
                  {confirmDelete ? t("sig.confirmDelete") : t("sig.delete")}
                </button>
                <span className="spacer" />
                {lastFired[selected.id] && (
                  <span style={{ color: "var(--text-faint)", fontFamily: "var(--mono)", fontSize: 11 }}>
                    {t("sig.lastFired", { at: fmtTime(lastFired[selected.id]) })}
                  </span>
                )}
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function HttpEditor({
  request,
  onChange,
}: {
  request: HttpRequest;
  onChange: (next: HttpRequest) => void;
}) {
  const t = useT();
  const patch = (fields: Partial<HttpRequest>) => onChange({ ...request, ...fields });
  const setHeader = (i: number, pair: [string, string]) =>
    patch({ headers: request.headers.map((h, j) => (j === i ? pair : h)) });

  return (
    <>
      <div className="row">
        <div className="field" style={{ flex: "0 0 108px" }}>
          <label htmlFor="sig-method">{t("sig.method")}</label>
          <select id="sig-method" value={request.method} onChange={(e) => patch({ method: e.target.value })}>
            {METHODS.map((m) => <option key={m} value={m}>{m}</option>)}
          </select>
        </div>
        <div className="field">
          <label htmlFor="sig-url">{t("sig.url")}</label>
          <input id="sig-url" value={request.url} onChange={(e) => patch({ url: e.target.value })} />
        </div>
        <div className="field" style={{ flex: "0 0 118px" }}>
          <label htmlFor="sig-timeout">{t("sig.timeout")}</label>
          <input
            id="sig-timeout"
            type="number"
            value={request.timeout_ms}
            onChange={(e) => patch({ timeout_ms: +e.target.value || 0 })}
          />
        </div>
      </div>
      <div className="field">
        <label>{t("sig.headers")}</label>
        {request.headers.map((h, i) => (
          <div className="row tight" key={i} style={{ marginBottom: 6 }}>
            <input aria-label={t("http.headerName")} placeholder={t("http.headerName")} value={h[0]} onChange={(e) => setHeader(i, [e.target.value, h[1]])} />
            <input aria-label={t("http.headerValue")} placeholder={t("http.headerValue")} value={h[1]} onChange={(e) => setHeader(i, [h[0], e.target.value])} />
            <button
              className="ghost sm"
              style={{ flex: "0 0 auto" }}
              aria-label={t("exp.removeHeader")}
              data-tip={t("exp.removeHeader")}
              onClick={() => patch({ headers: request.headers.filter((_, j) => j !== i) })}
            >
              ✕
            </button>
          </div>
        ))}
        <button className="ghost sm" onClick={() => patch({ headers: [...request.headers, ["", ""]] })}>
          {t("sig.addHeader")}
        </button>
      </div>
      {/* The credentials the signal was saved with: seen and changed here too, gone with "None". */}
      <HttpAuthFields auth={request.auth ?? { scheme: "none" }} onChange={(auth) => {
        const { auth: _previous, ...rest } = request;
        onChange(auth.scheme === "none" ? rest : { ...rest, auth });
      }} />
      <div className="field">
        <label htmlFor="sig-body">{t("sig.body")}</label>
        <textarea
          id="sig-body"
          value={request.body ?? ""}
          onChange={(e) => patch({ body: e.target.value || null })}
        />
      </div>
    </>
  );
}
