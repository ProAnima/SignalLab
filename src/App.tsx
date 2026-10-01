import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { StoreProvider, useStore, logText } from "./lib/store";
import { I18nProvider, LANGS, useI18n, type TKey } from "./lib/i18n";
import { Brand } from "./components/Brand";
import { Palette } from "./components/Palette";
import { TooltipLayer } from "./components/TooltipLayer";
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
import { fmtTime } from "./lib/format";
import { usePersistentState } from "./lib/hooks";
import { isDesktop, serverNeedsSignIn, signOut } from "./lib/platform";
import type { SignalBody } from "./lib/api";

type ViewKey =
  | "experiment" | "signals" | "osc" | "mqtt" | "broadcast" | "inspect" | "http" | "netsim" | "storm" | "scan";

/** `short` names the item in the compact rail, where the full label has no room. */
const NAV: { key: ViewKey; glyph: string; label: TKey; short?: TKey; kinds: string[] }[] = [
  { key: "experiment", glyph: "◇", label: "nav.experiment", short: "nav.short.experiment", kinds: ["experiment"] },
  { key: "signals", glyph: "❖", label: "nav.signals", kinds: [] },
  { key: "osc", glyph: "∿", label: "nav.osc", kinds: ["osc-monitor", "osc-gen"] },
  { key: "mqtt", glyph: "◈", label: "nav.mqtt", kinds: ["mqtt"] },
  { key: "broadcast", glyph: "⊛", label: "nav.broadcast", short: "nav.short.broadcast", kinds: ["beacon", "discovery"] },
  { key: "inspect", glyph: "◫", label: "nav.inspect", short: "nav.short.inspect", kinds: [] },
  { key: "http", glyph: "⇄", label: "nav.http", kinds: ["http-burst"] },
  { key: "netsim", glyph: "⚡", label: "nav.netsim", short: "nav.short.netsim", kinds: ["netsim"] },
  { key: "storm", glyph: "☰", label: "nav.storm", kinds: ["storm"] },
  { key: "scan", glyph: "⊹", label: "nav.scan", kinds: ["scan"] },
];

function LanguageSwitch() {
  const { lang, setLang } = useI18n();
  return (
    <div className="lang-switch" role="group" aria-label="Language">
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

function Shell() {
  const { host, info, connection, jobs, stopJob, stopAll, log, clearLog, pushLog } = useStore();
  // In a browser: whether this server asks for a token, so Sign out makes sense.
  const [signedIn, setSignedIn] = useState(false);
  useEffect(() => { serverNeedsSignIn().then(setSignedIn); }, []);
  const { t } = useI18n();
  // Reopen on the screen last used: someone who only ever sends OSC lands on OSC.
  const [view, setView] = usePersistentState<ViewKey>("signal-lab.view", "experiment", isView);
  const [compactNav, setCompactNav] = usePersistentState("signal-lab.navCompact", true);
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
  const lastLine = log[log.length - 1];
  const logEndRef = useRef<HTMLDivElement>(null);
  const [autoscroll, setAutoscroll] = useState(true);
  const [consoleOpen, setConsoleOpen] = useState(false);

  useEffect(() => {
    if (!autoscroll || !consoleOpen) return;
    // Scroll the log's own container, not the page — scrollIntoView on an
    // ancestor-scrolling element makes the whole layout jump.
    const end = logEndRef.current;
    const list = end?.parentElement;
    if (list) list.scrollTop = list.scrollHeight;
  }, [log, autoscroll, consoleOpen]);

  return (
    <div className={"app" + (consoleOpen ? "" : " console-collapsed") + (compactNav ? " sidebar-compact" : "") + (view === "experiment" ? " experiment-active" : "") + (focusMode && view === "experiment" ? " experiment-focus" : "")}>
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
        <button className="danger sm" onClick={stopAll} disabled={jobs.length === 0}>
          {t("app.stopAll")}
        </button>
      </header>

      {/* In a browser the engine is on the server; say when it cannot be reached. */}
      {connection === "lost" && <div className="connection-banner" role="status">{t("app.connectionLost")}</div>}
      <main className="main" ref={mainRef}>
        <div className="experiment-host" hidden={view !== "experiment"}>
          <ExperimentView ref={experiment} active={view === "experiment"} focusMode={focusMode} setFocusMode={setFocusMode} />
        </div>
        {page("signals", <SignalsView />)}
        {page("osc", <OscView onToExperiment={toExperiment} />)}
        {page("mqtt", <MqttView />)}
        {page("broadcast", <BroadcastView />)}
        {page("inspect", <InspectView />)}
        {page("http", <HttpView onToExperiment={toExperiment} />)}
        {page("netsim", <NetsimView />)}
        {page("storm", <StormView />)}
        {page("scan", <ScanView />)}
      </main>

      <section className="console">
        <div className="console-head">
          <button
            className="ghost sm console-toggle"
            onClick={() => setConsoleOpen(!consoleOpen)}
            aria-expanded={consoleOpen}
            data-tip={consoleOpen ? t("console.collapse") : t("console.expand")}
          >
            {consoleOpen ? "▾" : "▸"}
          </button>
          <p className="section-label">{t("console.title")}</p>
          <div className="jobs-strip">
            {jobs.map((j) => (
              <div className="job-pill" key={j.id} data-tip={j.label}>
                <span className="pulse" />
                <span>#{j.id} {j.label}</span>
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
          <label className="checkbox" style={{ fontSize: 11 }}>
            <input
              type="checkbox"
              checked={autoscroll}
              onChange={(e) => setAutoscroll(e.target.checked)}
            />
            {t("console.autoscroll")}
          </label>
          <button className="ghost sm" onClick={clearLog}>{t("common.clear")}</button>
        </div>
        <div className="log-list">
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
