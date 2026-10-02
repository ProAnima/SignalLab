import { useEffect, useLayoutEffect, useRef, useState, type CSSProperties, type ReactNode } from "react";
import { StoreProvider, useStore, logText } from "./lib/store";
import { I18nProvider, LANGS, useI18n, type TKey } from "./lib/i18n";
import { Brand } from "./components/Brand";
import { Palette } from "./components/Palette";
import { TooltipLayer } from "./components/TooltipLayer";
import { Splitter } from "./components/Splitter";
import { FirewallBanner } from "./components/FirewallBanner";
import { ExperimentView, type ExperimentHandle } from "./views/ExperimentView";
import { SignalsView } from "./views/SignalsView";
import { MqttView } from "./views/MqttView";
import { OscView } from "./views/OscView";
import { BroadcastView } from "./views/BroadcastView";
import { InspectView } from "./views/InspectView";
import { HttpView } from "./views/HttpView";
import { NetsimView } from "./views/NetsimView";
import { StormView } from "./views/StormView";
import { ScanView } from "./views/ScanView";
import { fmtNum, fmtTime } from "./lib/format";
import { usePersistentState } from "./lib/hooks";
import { isDesktop, serverNeedsSignIn, signOut } from "./lib/platform";
import { api, on, EV, type CaptureStats, type ExperimentNode, type InspectBatch, type JobInfo, type Signal, type SignalBody } from "./lib/api";
import { en } from "./lib/locales/en";

type ViewKey =
  | "experiment" | "signals" | "osc" | "mqtt" | "broadcast" | "http" | "netsim" | "storm" | "scan";
/** The bottom panel shows one of these; the Inspector watches every screen, so it lives here, not in the sidebar. */
type DockTab = "console" | "inspector";

/** `short` names the item in the compact rail, where the full label has no room. */
const NAV: { key: ViewKey; glyph: string; label: TKey; short?: TKey; kinds: string[] }[] = [
  { key: "experiment", glyph: "◇", label: "nav.experiment", short: "nav.short.experiment", kinds: ["experiment"] },
  { key: "signals", glyph: "❖", label: "nav.signals", kinds: [] },
  { key: "osc", glyph: "∿", label: "nav.osc", kinds: ["osc-monitor", "osc-gen"] },
  { key: "mqtt", glyph: "◈", label: "nav.mqtt", kinds: ["mqtt"] },
  { key: "broadcast", glyph: "⊛", label: "nav.broadcast", short: "nav.short.broadcast", kinds: ["beacon", "discovery"] },
  { key: "http", glyph: "⇄", label: "nav.http", kinds: ["http-burst"] },
  { key: "netsim", glyph: "⚡", label: "nav.netsim", short: "nav.short.netsim", kinds: ["netsim"] },
  { key: "storm", glyph: "☰", label: "nav.storm", kinds: ["storm"] },
  { key: "scan", glyph: "⊹", label: "nav.scan", kinds: ["scan"] },
];

function LanguageSwitch() {
  const { lang, setLang, t } = useI18n();
  return (
    <div className="lang-switch" role="group" aria-label={t("app.language")}>
      {LANGS.map((l) => (
        <button
          key={l.code}
          className={lang === l.code ? "on" : ""}
          data-tip={l.label}
          aria-pressed={lang === l.code}
          onClick={() => setLang(l.code)}
        >
          {l.short}
        </button>
      ))}
    </div>
  );
}

const isView = (value: unknown) => NAV.some((item) => item.key === value);
/** The console's height when open, until its handle moves it (double-click restores it). */
const CONSOLE_HEIGHT = 188;
/** The header and room for the screen above an open console. */
const ABOVE_CONSOLE = 54 + 200;
/** What a maximised bottom panel leaves of the screen above it. */
const ABOVE_MAXIMISED = 54 + 120;
/** The Inspector needs room for its bars, a few rows and a frame's detail. */
const INSPECTOR_HEIGHT = 320;

/** Whether capture is on and how much it has taken, for the Inspector's tab, from any screen. */
function useCaptureStats(): CaptureStats | null {
  const [stats, setStats] = useState<CaptureStats | null>(null);
  useEffect(() => {
    api.inspectStats().then(setStats).catch(() => {});
    const off = on<InspectBatch>(EV.inspectBatch, (event) => setStats(event.payload.stats));
    return () => { off.then((stop) => stop()); };
  }, []);
  return stats;
}

function Shell() {
  const { host, info, connection, jobs, stopJob, stopAll, log, clearLog, pushLog } = useStore();
  // In a browser: whether this server asks for a token, so Sign out makes sense.
  const [signedIn, setSignedIn] = useState(false);
  useEffect(() => { serverNeedsSignIn().then(setSignedIn); }, []);
  const { t } = useI18n();
  // A job as the engine describes it (`job.<kind>` with its values); the English label for a kind without a text.
  const jobLabel = (job: JobInfo) => `job.${job.kind}` in en ? t(`job.${job.kind}`, job.params) : job.label;
  // Reopen on the screen last used: someone who only ever sends OSC lands on OSC.
  const [view, setView] = usePersistentState<ViewKey>("signal-lab.view", "experiment", isView);
  const [compactNav, setCompactNav] = usePersistentState("signal-lab.navCompact", true);
  const [consoleHeight, setConsoleHeight] = usePersistentState("signal-lab.layout.console", CONSOLE_HEIGHT,
    (value) => typeof value === "number" && Number.isFinite(value) && value > 0);
  const [focusMode, setFocusMode] = useState(false);
  const experiment = useRef<ExperimentHandle>(null);
  // A screen is mounted the first time it is opened and then kept, hidden, for
  // the session: what was typed, a response, a running monitor's list and its
  // job all survive switching away and back.
  const [visited, setVisited] = useState<ReadonlySet<ViewKey>>(() => new Set([view]));
  const mainRef = useRef<HTMLElement>(null);
  const scrollOf = useRef<Partial<Record<ViewKey, number>>>({});
  const shown = useRef(view);
  const go = (next: ViewKey) => {
    // The screens share one scroll container; each comes back where it was left.
    if (mainRef.current) scrollOf.current[shown.current] = mainRef.current.scrollTop;
    setVisited((prior) => prior.has(next) ? prior : new Set(prior).add(next));
    setView(next); setFocusMode(false);
  };
  useLayoutEffect(() => {
    shown.current = view;
    const main = mainRef.current;
    if (!main) return;
    const top = scrollOf.current[view] ?? 0;
    main.scrollTop = top;
    // A screen mounting for the first time may still grow in its first frame
    // (and the browser may re-anchor the scroll); settle it once more then.
    const frame = requestAnimationFrame(() => { if (shown.current === view) main.scrollTop = top; });
    return () => cancelAnimationFrame(frame);
  }, [view]);
  const page = (key: ViewKey, content: ReactNode) => visited.has(key) && <div className="view-host" hidden={view !== key}>{content}</div>;
  /** A request that works in a direct instrument becomes the next step of the experiment. */
  const toExperiment = (body: SignalBody) => {
    if (experiment.current?.add(body)) go("experiment");
    else pushLog("warn", "experiment", "log.experimentBusy");
  };
  /** Wait for this: a wait built from a received message becomes the next step. */
  const waitInExperiment = (node: ExperimentNode) => {
    if (experiment.current?.addNode(node)) go("experiment");
    else pushLog("warn", "experiment", "log.experimentBusy");
  };
  // A frame the timeline points at, for the Inspector to select — in the panel, so the canvas stays put.
  const [revealFrame, setRevealFrame] = useState<{ seq: number; at: number } | null>(null);
  const showFrame = (seq: number) => {
    setRevealFrame({ seq, at: Date.now() });
    showDock("inspector");
  };
  // A stored signal opened in the screen of its transport, and a signal shown in the library.
  const [opened, setOpened] = useState<{ signal: Signal; at: number } | null>(null);
  const openSignal = (signal: Signal) => {
    const screen = ({ http: "http", osc: "osc", mqtt: "mqtt" } as const)[signal.body.transport as "http" | "osc" | "mqtt"];
    if (!screen) return;
    setOpened({ signal, at: Date.now() });
    go(screen);
  };
  const openedFor = (transport: string) => opened?.signal.body.transport === transport ? opened : null;
  const [revealSignal, setRevealSignal] = useState<{ id: string; at: number } | null>(null);
  const showSignal = (id: string) => {
    setRevealSignal({ id, at: Date.now() });
    go("signals");
  };
  const lastLine = log[log.length - 1];
  const logEndRef = useRef<HTMLDivElement>(null);
  const [autoscroll, setAutoscroll] = useState(true);
  // Open or not, and how tall, as it was left.
  const [consoleOpen, setConsoleOpen] = usePersistentState("signal-lab.console.open", false);
  const [dockTab, setDockTab] = usePersistentState<DockTab>("signal-lab.dock.tab", "console", (value) => value === "console" || value === "inspector");
  // Mounted once first shown, then kept: its list and selection survive closing the panel.
  const [inspectorShown, setInspectorShown] = useState(consoleOpen && dockTab === "inspector");
  const capture = useCaptureStats();
  // The height to go back to after the panel was made as tall as the window.
  const [restoreHeight, setRestoreHeight] = useState<number | null>(null);
  const showDock = (tab: DockTab) => {
    setDockTab(tab); setConsoleOpen(true);
    if (tab !== "inspector") return;
    setInspectorShown(true);
    // A console-sized panel leaves the Inspector no rows: it opens at least this tall.
    if (consoleHeight < INSPECTOR_HEIGHT) setConsoleHeight(Math.max(consoleHeight, Math.min(INSPECTOR_HEIGHT, window.innerHeight - ABOVE_CONSOLE)));
  };
  const toggleMaximised = () => {
    if (restoreHeight !== null) { setConsoleHeight(restoreHeight); setRestoreHeight(null); return; }
    setRestoreHeight(consoleHeight);
    setConsoleHeight(Math.max(96, window.innerHeight - ABOVE_MAXIMISED));
    setConsoleOpen(true);
  };

  useEffect(() => {
    if (!autoscroll || !consoleOpen) return;
    // Scroll the log's own container, not the page — scrollIntoView on an
    // ancestor-scrolling element makes the whole layout jump.
    const end = logEndRef.current;
    const list = end?.parentElement;
    if (list) list.scrollTop = list.scrollHeight;
  }, [log, autoscroll, consoleOpen]);

  return (
    <div className={"app" + (consoleOpen ? "" : " console-collapsed") + (compactNav ? " sidebar-compact" : "") + (view === "experiment" ? " experiment-active" : "") + (focusMode && view === "experiment" ? " experiment-focus" : "")}
      style={consoleOpen ? { "--console-h": `${consoleHeight}px` } as CSSProperties : undefined}>
      <aside className="sidebar">
        <Brand name={t("app.name")} by={t("app.by")} />
        {NAV.map((n) => {
          const running = jobs.filter((j) => n.kinds.includes(j.kind)).length;
          return (
            <button
              key={n.key}
              className={"nav-item" + (view === n.key ? " active" : "")}
              aria-current={view === n.key ? "page" : undefined}
              data-tip={compactNav ? t(n.label) : undefined}
              aria-label={t(n.label)}
              onClick={() => go(n.key)}
            >
              <span className="glyph" aria-hidden="true">{n.glyph}</span>
              <span className="label">{t(n.label)}</span>
              <span className="short" aria-hidden="true">{t(n.short ?? n.label)}</span>
              {running > 0 && (
                <span className="badge" data-tip={t("app.activeJobs", { n: running })}>{running}</span>
              )}
            </button>
          );
        })}
        <div className="fill" />
        <div className="foot">
          <span>{t("app.version", { version: __APP_VERSION__ })}</span>
          <span>{t("app.activeJobs", { n: jobs.length })}</span>
        </div>
      </aside>

      <header className="header">
        <button className="ghost sm nav-toggle" aria-label={compactNav ? t("app.expandNav") : t("app.collapseNav")} data-tip={compactNav ? t("app.expandNav") : t("app.collapseNav")} onClick={() => setCompactNav((value) => !value)}>☰</button>
        <div className="title">Signal <b>Lab</b></div>
        <div className="spacer" />
        {host && (
          <div className="host-chip" data-tip={!isDesktop ? t("app.serverHint", { dir: info?.data_dir ?? "" }) : undefined}>
            <span className={"live-dot" + (connection === "lost" ? " lost" : "")} />
            {!isDesktop && <span className="server-badge">{t("app.server")}</span>}
            {t("app.host")} <b>{host.hostname}</b> · <b>{host.local_ip}</b>
          </div>
        )}
        <LanguageSwitch />
        {signedIn && <button className="ghost sm" onClick={() => void signOut()}>{t("app.signOut")}</button>}
        <button className="danger sm" data-tip={t("app.stopAllHint")} onClick={stopAll} disabled={jobs.length === 0}>
          {t("app.stopAll")}
        </button>
      </header>

      {/* In a browser the engine is on the server; say when it cannot be reached. */}
      {connection === "lost" && <div className="connection-banner" role="status">{t("app.connectionLost")}</div>}
      <FirewallBanner />
      <main className="main" ref={mainRef}>
        <div className="experiment-host" hidden={view !== "experiment"}>
          <ExperimentView ref={experiment} active={view === "experiment"} focusMode={focusMode} setFocusMode={setFocusMode} onShowFrame={showFrame} />
        </div>
        {page("signals", <SignalsView onOpen={openSignal} reveal={revealSignal} />)}
        {page("osc", <OscView onToExperiment={toExperiment} onWaitFor={waitInExperiment} load={openedFor("osc")} onShowSignal={showSignal} />)}
        {page("mqtt", <MqttView onWaitFor={waitInExperiment} load={openedFor("mqtt")} onShowSignal={showSignal} />)}
        {page("broadcast", <BroadcastView />)}
        {page("http", <HttpView onToExperiment={toExperiment} load={openedFor("http")} onShowSignal={showSignal} />)}
        {page("netsim", <NetsimView />)}
        {page("storm", <StormView />)}
        {page("scan", <ScanView />)}
      </main>

      <section className="console">
        {consoleOpen && <Splitter orientation="horizontal" label="layout.console" size={consoleHeight} min={96} initial={CONSOLE_HEIGHT}
          max={() => Math.max(96, window.innerHeight - ABOVE_CONSOLE)} onSize={setConsoleHeight} />}
        <div className="console-head">
          <button
            className="ghost sm console-toggle"
            onClick={() => setConsoleOpen(!consoleOpen)}
            aria-expanded={consoleOpen}
            data-tip={consoleOpen ? t("console.collapse") : t("console.expand")}
          >
            {consoleOpen ? "▾" : "▸"}
          </button>
          <div className="dock-tabs" role="tablist" aria-label={t("dock.tabs")}>
            <button role="tab" className={`dock-tab ${dockTab === "console" ? "on" : ""}`} aria-selected={dockTab === "console"} aria-controls="dock-console"
              data-tip={t("dock.consoleHint")} onClick={() => showDock("console")}>{t("console.title")}</button>
            <button role="tab" className={`dock-tab ${dockTab === "inspector" ? "on" : ""}`} aria-selected={dockTab === "inspector"} aria-controls="dock-inspector"
              data-tip={capture?.enabled ? t("dock.inspectorLive", { n: capture.total }) : t("dock.inspectorHint")} onClick={() => showDock("inspector")}>
              <span className={`rec-dot ${capture?.enabled ? "live" : ""}`} aria-hidden="true" />
              {t("dock.inspector")}
              {!!capture?.total && <span className="dock-count">{fmtNum(capture.total)}</span>}
            </button>
          </div>
          <div className="jobs-strip">
            {jobs.map((j) => (
              <div className="job-pill" key={j.id} data-tip={jobLabel(j)}>
                <span className="pulse" />
                <span>#{j.id} {jobLabel(j)}</span>
                <button className="danger" onClick={() => stopJob(j.id)}>
                  {t("common.stop").toLowerCase()}
                </button>
              </div>
            ))}
            {jobs.length === 0 && (consoleOpen || !lastLine) && (
              <span style={{ color: "var(--text-faint)", fontFamily: "var(--mono)", fontSize: 11 }}>
                {t("console.empty")}
              </span>
            )}
          </div>
          {/* Collapsed, the console still says what the last action did. */}
          {!consoleOpen && lastLine ? (
            <button className={"console-last " + lastLine.level} data-tip={t("console.expand")} onClick={() => setConsoleOpen(true)}>
              <span className="t">{fmtTime(lastLine.ts).slice(0, 8)}</span>
              <span className="tag">{lastLine.tag}</span>
              <span className="msg">{logText(lastLine, t)}</span>
            </button>
          ) : <div className="spacer" />}
          {dockTab === "console" && <>
            <label className="checkbox" style={{ fontSize: 11 }} data-tip={t("console.autoscrollHint")}>
              <input
                type="checkbox"
                checked={autoscroll}
                onChange={(e) => setAutoscroll(e.target.checked)}
              />
              {t("console.autoscroll")}
            </label>
            <button className="ghost sm" onClick={clearLog}>{t("common.clear")}</button>
          </>}
          <button className="ghost sm dock-maximise" aria-pressed={restoreHeight !== null} onClick={toggleMaximised}
            aria-label={restoreHeight !== null ? t("dock.restore") : t("dock.maximise")} data-tip={restoreHeight !== null ? t("dock.restore") : t("dock.maximise")}>
            {restoreHeight !== null ? "⤡" : "⤢"}</button>
        </div>
        {inspectorShown && <div id="dock-inspector" role="tabpanel" className="dock-body" hidden={!consoleOpen || dockTab !== "inspector"}>
          <InspectView reveal={revealFrame} />
        </div>}
        <div id="dock-console" role="tabpanel" className="log-list" hidden={dockTab !== "console"}>
          {log.map((l) => (
            <div className={"log-line " + l.level} key={l.id}>
              <span className="t">{fmtTime(l.ts)}</span>
              <span className="tag">{l.tag}</span>
              <span className="msg">{logText(l, t)}</span>
            </div>
          ))}
          <div ref={logEndRef} />
        </div>
      </section>

      <Palette />
      <TooltipLayer />
    </div>
  );
}

export default function App() {
  return (
    <I18nProvider>
      <StoreProvider>
        <Shell />
      </StoreProvider>
    </I18nProvider>
  );
}
