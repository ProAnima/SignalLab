/** The WebSocket screen against the tour's echo service, and the echo template in the experiment editor. */
import { T, textOf, until, button, hasButton, control, panel, unnamed, click, type, go, openInspector, type StepArgs, type Expect } from "../dsl";
import { openTemplate, runAndWait, selectNode } from "./experiment";

export async function websocket(expect: Expect, args: StepArgs) {
  const shown = await go("ws");
  const connection = panel(shown, T("ws.connection"));
  await type(control(connection, "URL"), `ws://127.0.0.1:${args.port}/tour`);
  await type(control(connection, T("exp.wsProtocols")), "tour.v1, tour.v0");
  await click(button(connection, T("http.addHeader")));
  const [name, value] = [...connection.querySelectorAll<HTMLInputElement>(`input[aria-label="${T("http.headerName")}"], input[aria-label="${T("http.headerValue")}"]`)];
  await type(name, "X-Tour");
  await type(value, "e2e");
  const nameless = unnamed(shown);
  expect("websocket: every field and button has a name", nameless.length === 0, nameless.join(" | "));

  await click(button(connection, T("ws.connect")));
  await until("the connection", () => textOf(connection).includes(T("ws.open")), 10000);
  expect("the service chose the first subprotocol", textOf(connection).includes("tour.v1"), textOf(connection));
  const messages = panel(shown, T("ws.messages"));
  const rows = (dir: "rx" | "tx") => [...messages.querySelectorAll<HTMLButtonElement>(`.ws-row.${dir}`)];
  await until("the greeting", () => rows("rx").some((row) => textOf(row.querySelector(".ws-text")) === "welcome"));

  const compose = panel(shown, T("ws.message"));
  await type(compose.querySelector("textarea")!, '{"hello":"tour"}');
  await click(button(compose, T("common.send")));
  const echo = await until("the echo", () => rows("rx").find((row) => textOf(row.querySelector(".ws-text")) === '{"hello":"tour"}'));
  expect("what was sent is listed too", rows("tx").some((row) => textOf(row.querySelector(".ws-text")) === '{"hello":"tour"}'));
  await click(echo, "the echo");
  const detail = await until("the message", () => shown.querySelector<HTMLElement>(".ws-detail"));
  expect("a JSON message reads formatted", textOf(detail).includes('"hello": "tour"'), textOf(detail));

  await click(button(compose, T("exp.wsBinary")));
  await type(compose.querySelector("textarea")!, "ca fe");
  await click(button(compose, T("common.send")));
  await until("the bytes back", () => rows("rx").some((row) => textOf(row.querySelector(".ws-text")) === "ca fe"));
  expect("bytes go as a binary message and come back as one", rows("rx").some((row) => textOf(row).includes(T("ws.binary"))));
  await click(button(compose, T("exp.wsText")));

  await click(button(connection, T("ws.disconnect")));
  await until("closed", () => textOf(connection).includes(T("ws.closedByYou", { code: 1000 })), 10000);
  expect("the connection closes with 1000, and can be opened again", !!hasButton(connection, T("ws.connect")));

  // A message and its echo a millisecond apart: the Inspector has both.
  const inspector = await openInspector();
  await click(button(inspector, "ws"));
  const frames = await until("the WebSocket frames", () => {
    const rows = [...inspector.querySelectorAll("tbody tr[data-frame]")].map((row) => textOf(row));
    return rows.filter((row) => row.includes('TEXT {"hello":"tour"}')).length >= 2 && rows;
  }, 5000).catch(() => [...inspector.querySelectorAll("tbody tr[data-frame]")].map((row) => textOf(row)));
  const pair = frames.filter((row) => row.includes('TEXT {"hello":"tour"}'));
  expect("the Inspector holds the message and its echo", pair.length === 2, frames.slice(0, 8).join(" | "));
  await click(button(inspector, "ws"));
}

/** The echo template: connect, a JSON ping, the same JSON back, close. */
export async function experimentWs(expect: Expect, args: StepArgs) {
  const editor = await openTemplate("exp.templateWsEcho");
  await click(button(editor, T("exp.params")));
  const params = await until("the parameters", () => document.querySelector<HTMLElement>(".experiment-params"));
  await type(control(params, T("exp.paramValue")), `ws://127.0.0.1:${args.port}/echo`);
  await click(button(params, T("exp.close")));
  const send = await selectNode(editor, "exp.node.ws_send", 0, expect);
  expect("the send names the template's connection", (control(send, T("exp.wsConnection")) as HTMLSelectElement).value === "socket");
  await selectNode(editor, "exp.node.ws_connect", 0, expect);
  await selectNode(editor, "exp.node.wait_ws", 0, expect);
  await selectNode(editor, "exp.node.ws_close", 0, expect);
  const { outcome, rows } = await runAndWait(editor);
  expect("the WebSocket echo run passes", outcome === T("exp.passed"), `${outcome} · ${rows.join(" | ")}`);
  expect("the timeline says it connected and closed", rows.some((row) => row.includes(T("exp.node.ws_connect"))) && rows.some((row) => row.includes(T("exp.step.wsClosed", { by: "client", code: 1000 }))), rows.join(" | "));
}
