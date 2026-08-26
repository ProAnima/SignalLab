import { useEffect, useRef, useState } from "react";
import { StoreProvider, useStore } from "./lib/store";
import { I18nProvider, LANGS, useI18n, type TKey } from "./lib/i18n";
import { Brand } from "./components/Brand";
import { OscView } from "./views/OscView";
import { BroadcastView } from "./views/BroadcastView";
import { InspectView } from "./views/InspectView";
import { HttpView } from "./views/HttpView";
import { NetsimView } from "./views/NetsimView";
import { StormView } from "./views/StormView";
import { ScanView } from "./views/ScanView";
import { fmtTime } from "./lib/format";

const APP_VERSION = "0.2";

type ViewKey = "osc" | "broadcast" | "inspect" | "http" | "netsim" | "storm" | "scan";

const NAV: { key: ViewKey; glyph: string; label: TKey; kinds: string[] }[] = [
  { key: "osc", glyph: "∿", label: "nav.osc", kinds: ["osc-monitor", "osc-gen"] },
  { key: "broadcast", glyph: "⊛", label: "nav.broadcast", kinds: ["beacon", "discovery"] },
  { key: "inspect", glyph: "◫", label: "nav.inspect", kinds: [] },
  { key: "http", glyph: "⇄", label: "nav.http", kinds: ["http-burst"] },
  { key: "netsim", glyph: "⚡", label: "nav.netsim", kinds: ["netsim"] },
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
          title={l.label}
          aria-pressed={lang === l.code}
          onClick={() => setLang(l.code)}
        >
          {l.short}
        </button>
      ))}
    </div>
  );
}

function Shell() {
  const { host, jobs, stopJob, stopAll, log, clearLog } = useStore();
  const { t } = useI18n();
  const [view, setView] = useState<ViewKey>("osc");
  const logEndRef = useRef<HTMLDivElement>(null);
  const [autoscroll, setAutoscroll] = useState(true);
  const [consoleOpen, setConsoleOpen] = useState(true);

  useEffect(() => {
    if (!autoscroll || !consoleOpen) return;
    // Scroll the log's own container, not the page — scrollIntoView on an
    // ancestor-scrolling element makes the whole layout jump.
    const end = logEndRef.current;
    const list = end?.parentElement;
    if (list) list.scrollTop = list.scrollHeight;
  }, [log, autoscroll, consoleOpen]);

  return (
    <div className={"app" + (consoleOpen ? "" : " console-collapsed")}>
      <aside className="sidebar">
        <Brand name={t("app.name")} tagline={t("app.tagline")} by={t("app.by")} />
        {NAV.map((n) => {
          const running = jobs.filter((j) => n.kinds.includes(j.kind)).length;
          return (
            <button
              key={n.key}
              className={"nav-item" + (view === n.key ? " active" : "")}
              aria-current={view === n.key ? "page" : undefined}
              onClick={() => setView(n.key)}
            >
              <span className="glyph" aria-hidden="true">{n.glyph}</span>
              <span className="label">{t(n.label)}</span>
              {running > 0 && (
                <span className="badge" title={t("app.activeJobs", { n: running })}>{running}</span>
              )}
            </button>
          );
        })}
        <div className="fill" />
        <div className="foot">
          <span>{t("app.version", { version: APP_VERSION })}</span>
          <span>{t("app.activeJobs", { n: jobs.length })}</span>
        </div>
      </aside>

      <header className="header">
        <div className="title">Signal <b>Lab</b></div>
        <div className="spacer" />
        {host && (
          <div className="host-chip">
            <span className="live-dot" />
            {t("app.host")} <b>{host.hostname}</b> · <b>{host.local_ip}</b>
          </div>
        )}
        <LanguageSwitch />
        <button className="danger sm" onClick={stopAll} disabled={jobs.length === 0}>
          {t("app.stopAll")}
        </button>
      </header>

      <main className="main">
        {view === "osc" && <OscView />}
        {view === "broadcast" && <BroadcastView />}
        {view === "inspect" && <InspectView />}
        {view === "http" && <HttpView />}
        {view === "netsim" && <NetsimView />}
        {view === "storm" && <StormView />}
        {view === "scan" && <ScanView />}
      </main>

      <section className="console">
        <div className="console-head">
          <button
            className="ghost sm console-toggle"
            onClick={() => setConsoleOpen(!consoleOpen)}
            aria-expanded={consoleOpen}
            title={consoleOpen ? t("console.collapse") : t("console.expand")}
          >
            {consoleOpen ? "▾" : "▸"}
          </button>
          <p className="section-label">{t("console.title")}</p>
          <div className="jobs-strip">
            {jobs.map((j) => (
              <div className="job-pill" key={j.id} title={j.label}>
                <span className="pulse" />
                <span>#{j.id} {j.label}</span>
                <button className="danger" onClick={() => stopJob(j.id)}>
                  {t("common.stop").toLowerCase()}
                </button>
              </div>
            ))}
            {jobs.length === 0 && (
              <span style={{ color: "var(--text-faint)", fontFamily: "var(--mono)", fontSize: 11 }}>
                {t("console.empty")}
              </span>
            )}
          </div>
          <div className="spacer" />
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
              <span className="msg">{t(l.key, l.params)}</span>
            </div>
          ))}
          <div ref={logEndRef} />
        </div>
      </section>
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
