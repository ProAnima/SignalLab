import { useEffect, useRef, useState } from "react";
import { StoreProvider, useStore } from "./lib/store";
import { OscView } from "./views/OscView";
import { HttpView } from "./views/HttpView";
import { NetsimView } from "./views/NetsimView";
import { StormView } from "./views/StormView";
import { ScanView } from "./views/ScanView";
import { fmtTime } from "./lib/format";

type ViewKey = "osc" | "http" | "netsim" | "storm" | "scan";

const NAV: { key: ViewKey; glyph: string; label: string; kinds: string[] }[] = [
  { key: "osc", glyph: "∿", label: "OSC", kinds: ["osc-monitor", "osc-gen"] },
  { key: "http", glyph: "⇄", label: "HTTP", kinds: ["http-burst"] },
  { key: "netsim", glyph: "⚡", label: "Impairment", kinds: ["netsim"] },
  { key: "storm", glyph: "☰", label: "Storm", kinds: ["storm"] },
  { key: "scan", glyph: "⊹", label: "Scanner", kinds: ["scan"] },
];

function Shell() {
  const { host, jobs, stopJob, stopAll, log, clearLog } = useStore();
  const [view, setView] = useState<ViewKey>("osc");
  const logEndRef = useRef<HTMLDivElement>(null);
  const [autoscroll, setAutoscroll] = useState(true);

  useEffect(() => {
    if (autoscroll) logEndRef.current?.scrollIntoView({ block: "end" });
  }, [log, autoscroll]);

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="brand">
          <div className="dot" />
          <div className="name">Signal Lab<small>OSC · NET · PROTO</small></div>
        </div>
        {NAV.map((n) => {
          const running = jobs.filter((j) => n.kinds.includes(j.kind)).length;
          return (
            <div key={n.key} className={"nav-item" + (view === n.key ? " active" : "")} onClick={() => setView(n.key)}>
              <span className="glyph">{n.glyph}</span>
              <span className="label">{n.label}</span>
              {running > 0 && <span className="badge">{running}</span>}
            </div>
          );
        })}
        <div className="fill" />
        <div className="foot">v0.1 · {jobs.length} active</div>
      </aside>

      <header className="header">
        <div className="title">Signal <b>Lab</b></div>
        <div className="spacer" />
        {host && <div className="host-chip">host <b>{host.hostname}</b> · <b>{host.local_ip}</b></div>}
        <button className="danger sm" onClick={stopAll} disabled={jobs.length === 0}>Stop all</button>
      </header>

      <main className="main">
        {view === "osc" && <OscView />}
        {view === "http" && <HttpView />}
        {view === "netsim" && <NetsimView />}
        {view === "storm" && <StormView />}
        {view === "scan" && <ScanView />}
      </main>

      <section className="console">
        <div className="console-head">
          <p className="section-label">Console</p>
          <div className="jobs-strip">
            {jobs.map((j) => (
              <div className="job-pill" key={j.id} title={j.label}>
                <span className="pulse" />
                <span>#{j.id} {j.label}</span>
                <button className="danger" onClick={() => stopJob(j.id)}>stop</button>
              </div>
            ))}
            {jobs.length === 0 && <span style={{ color: "var(--text-faint)", fontFamily: "var(--mono)", fontSize: 11 }}>no active jobs</span>}
          </div>
          <div className="spacer" />
          <label className="checkbox" style={{ fontSize: 11 }}>
            <input type="checkbox" checked={autoscroll} onChange={(e) => setAutoscroll(e.target.checked)} /> auto-scroll
          </label>
          <button className="ghost sm" onClick={clearLog}>Clear</button>
        </div>
        <div className="log-list">
          {log.map((l) => (
            <div className={"log-line " + l.level} key={l.id}>
              <span className="t">{fmtTime(l.ts)}</span>
              <span className="tag">{l.tag}</span>
              <span className="msg">{l.msg}</span>
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
    <StoreProvider>
      <Shell />
    </StoreProvider>
  );
}
