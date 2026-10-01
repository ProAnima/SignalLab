// Every kind of element a tooltip sits on, with the real TooltipLayer and the
// real stylesheet: driven in WebKitGTK — the Linux desktop app's webview — by
// scripts/webkit.mjs with real pointer and key events.
import { useRef } from "react";
import { createRoot } from "react-dom/client";
import { TooltipLayer } from "../../src/components/TooltipLayer";
import "../../src/styles.css";

function Harness() {
  const dialog = useRef<HTMLDialogElement>(null);
  return <div style={{ padding: 120, display: "grid", gap: 24, justifyItems: "start" }}>
    <button id="enabled" data-tip="Отправить запрос и сохранить ответ · Ctrl+Enter">Enabled</button>
    <div className="btn-row"><button id="disabled" disabled data-tip="Нет полной копии кадра — повторить байт в байт нельзя">Disabled</button></div>
    <div className="field" style={{ width: 300 }}><label id="label" data-tip="0 — без ограничения">Раундов</label><input id="field" /></div>
    <label id="wrap" data-tip="Подсказка подписи" style={{ display: "grid", width: 300 }}>Текст <input id="wrapped" /></label>
    <fieldset disabled style={{ border: 0, padding: 0 }}><button id="inFieldset" data-tip="Кнопка в отключённом fieldset">In fieldset</button></fieldset>
    <button id="open" onClick={() => dialog.current?.showModal()}>Open dialog</button>
    <dialog ref={dialog} style={{ padding: 40, background: "var(--surface-2)", border: "1px solid var(--border-strong)" }}>
      <button id="inDialog" data-tip="Поверх модального окна">In dialog</button>
    </dialog>
    <TooltipLayer />
  </div>;
}

createRoot(document.getElementById("root")!).render(<Harness />);
