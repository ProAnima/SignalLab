import { useEffect, useMemo, useState } from "react";
import { api, type EmulatorProtocol, type StoredEmulator } from "../lib/api";
import { useStore } from "../lib/store";
import { useEmulators } from "../lib/emulatorStore";
import { useT, type TKey } from "../lib/i18n";
import { useFieldIds } from "../lib/hooks";
import { describeError, type Failure } from "../lib/errors";
import { fmtNum, fmtTime } from "../lib/format";
import { EmulatorEditor } from "../components/EmulatorEditor";
import { ErrorMessage } from "../components/ErrorMessage";
import { copyText, isDesktop } from "../lib/platform";
import { blankEmulator, emulatorUrl, freeBind, makeEmulatorId, PROTOCOLS, protocolLabel } from "../lib/emulators";

/** How long after the last keystroke an emulator is checked again. */
const CHECK_AFTER_MS = 300;

/**
 * Signal Lab as the other side: the library of emulators on the left, the
 * chosen one's rules on the right, and what it receives while it runs.
 * `reveal`: an emulator to select (Mock this made or changed it).
 */
export function EmulatorsView({ reveal, onShowFrame }: { reveal?: { id: string; at: number } | null; onShowFrame?: (seq: number) => void }) {
  const t = useT();
  const fid = useFieldIds();
  const { stopJob, pushLog, pushError } = useStore();
  const { emulators, setEmulators, path, error: libraryError, saveState, reload, running, live, start, startedWith } = useEmulators();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [problem, setProblem] = useState<Failure | null>(null);
  const [startError, setStartError] = useState<Failure | null>(null);
  const [starting, setStarting] = useState(false);
  const [copied, setCopied] = useState<boolean | null>(null);

  const selected = emulators.find((stored) => stored.id === selectedId) ?? null;
  const job = selected ? running.get(selected.id) ?? null : null;
  const activity = job ? live[job.id] : undefined;

  // The first one, until someone picks another.
  useEffect(() => {
    if (selectedId === null && emulators.length) setSelectedId(emulators[0].id);
  }, [emulators, selectedId]);
  useEffect(() => {
    if (reveal && emulators.some((stored) => stored.id === reveal.id)) setSelectedId(reveal.id);
  }, [reveal]); // eslint-disable-line react-hooks/exhaustive-deps
  useEffect(() => { setConfirmDelete(false); setStartError(null); setCopied(null); }, [selectedId]);

  // What Start would be refused for, while it is being written.
  const edited = selected?.emulator;
  useEffect(() => {
    if (!edited) { setProblem(null); return; }
    const timer = window.setTimeout(() => {
      api.emulatorCheck(edited).then(() => setProblem(null), (error) => setProblem(error));
    }, CHECK_AFTER_MS);
    return () => window.clearTimeout(timer);
  }, [edited]);

  const replace = (next: StoredEmulator) => setEmulators(emulators.map((stored) => stored.id === next.id ? next : stored));

  const create = (protocol: EmulatorProtocol) => {
    const name = t(`emu.newName.${protocol}` as TKey);
    const stored: StoredEmulator = { id: makeEmulatorId(name, emulators), note: "", emulator: blankEmulator(protocol, name, freeBind(protocol, emulators)) };
    setEmulators([...emulators, stored]);
    setSelectedId(stored.id);
  };

  const duplicate = () => {
    if (!selected) return;
    const name = `${selected.emulator.name} ·`;
    const copy: StoredEmulator = { ...selected, id: makeEmulatorId(name, emulators), emulator: { ...selected.emulator, name, bind: freeBind(selected.emulator.protocol, emulators) } };
    const at = emulators.findIndex((stored) => stored.id === selected.id);
    const next = emulators.slice();
    next.splice(at + 1, 0, copy);
    setEmulators(next);
    setSelectedId(copy.id);
  };

  const remove = () => {
    if (!selected) return;
    if (job) stopJob(job.id);
    setEmulators(emulators.filter((stored) => stored.id !== selected.id));
    setSelectedId(null);
  };

  const run = async () => {
    if (!selected) return;
    setStartError(null);
    setStarting(true);
    try {
      if (job) stopJob(job.id);
      await start(selected);
    } catch (error) {
      setStartError(error);
    } finally {
      setStarting(false);
    }
  };

  const changed = useMemo(() => {
    if (!job || !selected) return false;
    const was = startedWith(job.id);
    return !!was && JSON.stringify(was) !== JSON.stringify(selected.emulator);
  }, [job, selected, startedWith]);

  // In a browser the engine is on the server: one listening everywhere is reached by the server's name.
  const url = selected ? emulatorUrl(selected.emulator, job?.params?.local, isDesktop ? undefined : window.location.hostname) : null;
  const copyUrl = () => {
    if (!url) return;
    void copyText(url).then(setCopied);
  };

  const exchanges = activity ? activity.exchanges.slice().reverse() : [];

  return (
    <div>
      <div className="view-head">
        <h1 data-tip={t("emu.blurb")}>{t("emu.title")}</h1>
      </div>

      <div className="cols side">
        {/* ---- library ---- */}
        <div className="panel" style={{ display: "flex", flexDirection: "column", minHeight: 420 }}>
          <p className="section-label">{t("emu.library")}</p>
          <div className="emu-new" role="group" aria-label={t("emu.new")}>
            {PROTOCOLS.map((protocol) => <button key={protocol} className="ghost sm" data-tip={t(`emu.newHint.${protocol}` as TKey)} onClick={() => create(protocol)}>
              ＋ {t(`emu.new.${protocol}` as TKey)}
            </button>)}
          </div>
          <div className="sig-list">
            {emulators.map((stored) => {
              const runningJob = running.get(stored.id);
              const total = runningJob ? live[runningJob.id]?.counts.total ?? 0 : null;
              return <button key={stored.id} className={`sig-item emu-item ${stored.id === selectedId ? "active" : ""}`} aria-current={stored.id === selectedId ? "true" : undefined}
                onClick={() => setSelectedId(stored.id)}>
                <span className="sig-item-name">{runningJob && <span className="pulse emu-pulse" aria-hidden="true" />}{stored.emulator.name}</span>
                <span className="sig-item-meta">
                  <span className="sig-badge">{protocolLabel(stored.emulator.protocol)}</span>
                  <span>{stored.emulator.bind}</span>
                  {total !== null && <span className="emu-item-total" data-tip={t("emu.total")}>{fmtNum(total)}</span>}
                </span>
              </button>;
            })}
          </div>
          {emulators.length === 0 && <div className="empty-state">{libraryError ? describeError(libraryError, t).text : t("emu.empty")}</div>}
          <div className="sig-foot">
            <span data-tip={path}>{t("emu.count", { n: emulators.length })}</span>
            <span className="spacer" />
            {saveState === "saving" && <span>{t("emu.saving")}</span>}
            {saveState === "saved" && <span className="ok">{t("emu.saved")}</span>}
            <button className="ghost sm" onClick={reload} data-tip={path}>{t("emu.reload")}</button>
          </div>
        </div>

        {/* ---- the chosen one ---- */}
        <div className="emu-main">
          <div className="panel">
            {!selected && <div className="empty-state">{t("emu.pick")}</div>}
            {selected && <>
              <div className="btn-row emu-controls">
                <button className={job ? "danger" : "primary"} disabled={starting} data-tip={job ? undefined : t("emu.startHint")}
                  onClick={() => (job ? stopJob(job.id) : void run())}>
                  {job ? t("emu.stop") : t("emu.start")}
                </button>
                {job && changed && <button className="ghost" disabled={starting} data-tip={t("emu.restartHint")} onClick={() => void run()}>{t("emu.restart")}</button>}
                {/* Pull the plug, or put it back: the client under test meets an outage on cue. */}
                {job && <button className="ghost" data-tip={activity?.forced ? undefined : t("emu.takeDownHint")} onClick={() => {
                  const down = !activity?.forced;
                  api.emulatorDown(job.id, down).then(
                    () => pushLog(down ? "warn" : "ok", "emulators", down ? "log.emulatorTakenDown" : "log.emulatorBroughtUp", { name: selected.emulator.name }),
                    (error) => pushError("emulators", error),
                  );
                }}>{activity?.forced ? t("emu.bringUp") : t("emu.takeDown")}</button>}
                {url && <button className="ghost" data-tip={copied === true ? t("emu.copied") : copied === false ? t("emu.copyFailed", { url }) : t("emu.copyUrlHint", { url })} onClick={copyUrl}>{t("emu.copyUrl")}</button>}
                <button className="ghost" onClick={duplicate}>{t("emu.duplicate")}</button>
                {/* Two steps, because the file is written the moment you click. */}
                <button className="danger" onClick={() => (confirmDelete ? remove() : setConfirmDelete(true))}>
                  {confirmDelete ? t("emu.confirmDelete") : t("emu.delete")}
                </button>
                <span className="spacer" />
                <span className={`emu-state ${job ? (activity?.forced ? "down" : "on") : ""}`} role="status">
                  {!job ? t("emu.notRunning") : t(activity?.forced ? "emu.isDown" : "emu.runningOn", { local: job.params?.local ?? selected.emulator.bind })}
                </span>
              </div>
              {startError !== null && <ErrorMessage error={startError} />}
              {problem !== null && startError === null && <ErrorMessage className="emu-problem" error={problem} />}
              {/* Hits belong to the rules it was started with: after an edit they would sit on the wrong rule. */}
              <EmulatorEditor emulator={selected.emulator} onChange={(emulator) => replace({ ...selected, emulator })} hits={changed ? undefined : activity?.counts.hits} />
              <div className="field" style={{ marginTop: 14 }}>
                <label htmlFor={fid("note")} data-tip={t("emu.noteHint")}>{t("emu.note")}</label>
                <textarea id={fid("note")} value={selected.note} onChange={(event) => replace({ ...selected, note: event.target.value })} />
              </div>
            </>}
          </div>

          {selected && <div className="panel emu-live">
            <p className="section-label">{t("emu.live")}</p>
            <div className="metrics">
              <div className="metric"><div className="k">{t("emu.total")}</div><div className="v accent">{fmtNum(activity?.counts.total ?? 0)}</div></div>
              <div className="metric"><div className="k">{t("emu.unmatched")}</div><div className="v amber">{fmtNum(activity?.counts.unmatched ?? 0)}</div></div>
              <div className="metric"><div className="k">{t("emu.failed")}</div><div className="v red">{fmtNum(activity?.counts.failed ?? 0)}</div></div>
              {(selected.emulator.outage || !!activity?.counts.down) && <div className="metric">
                <div className="k">{t("emu.down")}</div><div className="v">{fmtNum(activity?.counts.down ?? 0)}</div>
              </div>}
              {!!activity?.counts.missed && <div className="metric" data-tip={t("emu.missedHint")}>
                <div className="k">{t("emu.missed")}</div><div className="v amber">{fmtNum(activity.counts.missed)}</div>
              </div>}
            </div>
            <p className="section-label" style={{ marginTop: 16 }}>{t("emu.received")}</p>
            <div className="scroll-y emu-exchanges">
              <table className="grid">
                <thead>
                  <tr>
                    <th style={{ width: 96 }}>{t("emu.col.time")}</th>
                    <th style={{ width: 140 }}>{t("emu.col.from")}</th>
                    <th>{t("emu.col.request")}</th>
                    <th style={{ width: 56 }}>{t("emu.col.rule")}</th>
                    <th>{t("emu.col.reply")}</th>
                    <th style={{ width: 56 }}>{t("emu.col.ms")}</th>
                    {onShowFrame && <th style={{ width: 40 }} aria-label={t("emu.inspectFrame")} />}
                  </tr>
                </thead>
                <tbody>
                  {exchanges.map((exchange) => <tr key={exchange.seq} className={exchange.error ? "emu-failed" : exchange.down ? "emu-down" : exchange.rule === undefined ? "emu-unmatched" : ""}>
                    <td style={{ color: "var(--text-faint)" }}>{fmtTime(exchange.ts)}</td>
                    <td>{exchange.from}</td>
                    <td className="emu-request">{exchange.request}</td>
                    <td>{exchange.rule !== undefined ? `#${exchange.rule}` : "—"}</td>
                    <td>{exchange.down && <span className="emu-was-down">{t("emu.wasDown")}</span>}{exchange.error ? describeError(exchange.error, t).text
                      : exchange.fault === "timeout" ? t("emu.held") : exchange.fault === "reset" ? t("emu.closed") : exchange.reply || (exchange.down ? "" : "—")}</td>
                    <td>{fmtNum(exchange.ms)}</td>
                    {onShowFrame && <td>{exchange.frame !== undefined && <button className="ghost xs" aria-label={t("emu.inspectFrame")} data-tip={t("emu.inspectFrame")}
                      onClick={() => onShowFrame(exchange.frame!)}>⌕</button>}</td>}
                  </tr>)}
                  {exchanges.length === 0 && <tr><td colSpan={onShowFrame ? 7 : 6} className="empty-state">{job ? t("emu.nothingYet") : t("emu.notRunning")}</td></tr>}
                </tbody>
              </table>
            </div>
            {!!activity?.dropped && <p className="emu-dropped" role="status">{t("emu.notShown", { n: activity.dropped })}</p>}
          </div>}
        </div>
      </div>
    </div>
  );
}
