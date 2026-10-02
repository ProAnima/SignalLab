/** The Emulators screen: an HTTP emulator made, talked to, mocked into and taken down again. */
import {
  T, sleep, textOf, until, screen, control, button, hasButton, buttonWith, checkbox, metric, panel, unnamed, click, type, setChecked, go, result, type StepArgs, type Expect,
} from "../dsl";

/**
 * An HTTP emulator made on its screen: it answers the HTTP screen, Mock this
 * turns that answer into a route of its own, the exchange is listed with its
 * rule, a restart takes the new route, and it is stopped and deleted again.
 */
export async function emulators(expect: Expect, args: StepArgs) {
  const shown = await go("emulators");
  const library = panel(shown, T("emu.library"));
  expect("the starter set is in the library", library.querySelectorAll(".emu-item").length >= 4, textOf(library));
  await click(button(library, `＋ ${T("emu.new.http")}`));
  const editor = await until("the new emulator", () => textOf(shown.querySelector(".emu-item.active")).includes(T("emu.newName.http")) && shown.querySelector<HTMLElement>(".emu-editor"));
  const local = `127.0.0.1:${args.port}`;
  await type(control(editor, T("emu.bind")), local);
  await type(control(editor, T("emu.path")), "/e2e/:id");
  await type(control(editor, T("emu.body")), '{"id":"{{request.params.id}}","via":"emulator"}');
  expect("a route offers its URL", !!hasButton(editor, T("emu.copyRouteUrl")));
  await sleep(600);
  expect("it checks itself while being written: nothing in the way", !shown.querySelector(".emu-problem"), textOf(shown.querySelector(".emu-problem")));
  await click(button(shown, T("emu.start")));
  await until("it to answer", () => textOf(shown.querySelector(".emu-state")) === T("emu.runningOn", { local }));
  expect("the console lists its job", textOf(document.querySelector(".jobs-strip")).includes(T("job.emulator", { name: T("emu.newName.http"), local })), textOf(document.querySelector(".jobs-strip")));
  const nameless = unnamed(shown);
  expect("emulators: every field and button has a name", nameless.length === 0, nameless.join(" | "));

  // The HTTP screen talks to it.
  const http = await go("http");
  const request = panel(http, T("http.request"));
  await type(request.querySelector("select")!, "GET");
  await type(control(request, "URL"), `http://${local}/e2e/7`);
  await click(button(request, T("common.send")));
  const verdict = await until("the emulator's answer", () => result(request)?.classList.contains("ok") && result(request));
  expect("the emulator answers 200", textOf(verdict).startsWith("200"), textOf(verdict));
  const response = panel(http, T("http.response"));
  const body = response.querySelector<HTMLTextAreaElement>("textarea")!;
  expect("with the id from its path", body.value.includes('"id": "7"') && body.value.includes('"via": "emulator"'), body.value.slice(0, 120));

  // Mock this: that answer becomes a route of the same emulator, first in its list.
  await click(buttonWith(response, T("http.mockThis")));
  const dialog = await until("the Mock this dialog", () => document.querySelector<HTMLDialogElement>("dialog[open]"));
  const into = control(dialog, T("http.mockInto")) as HTMLSelectElement;
  const option = [...into.options].find((item) => item.text.startsWith(T("emu.newName.http")));
  if (!option) throw new Error(`no ${T("emu.newName.http")} to mock into: ${[...into.options].map((item) => item.text).join(", ")}`);
  await type(into, option.value);
  await click(button(dialog, T("http.mockAdd")));
  const back = await until("the route on the Emulators screen", () => {
    const now = screen();
    return textOf(now.querySelector(".emu-rule .emu-rule-summary")) === "GET /e2e/7 → 200" && now;
  });
  expect("Mock this adds the route first", true);
  expect("the running emulator offers a restart for it", !!hasButton(back, T("emu.restart")));

  // What it received, as it arrived.
  const live = panel(back, T("emu.live"));
  const row = await until("the exchange", () => [...live.querySelectorAll("tbody tr")].map(textOf).find((text) => text.includes("GET /e2e/7")));
  expect("the request is listed with its rule and answer", row.includes("#1") && row.includes("200 OK"), row);
  expect("and counted", metric(live, T("emu.total")) === "1", metric(live, T("emu.total")));
  await click(button(back, T("emu.restart")));
  await until("the restart", () => !hasButton(back, T("emu.restart")) && textOf(back.querySelector(".emu-state")) === T("emu.runningOn", { local }));
  expect("a restart takes the rules as they are now", true);

  await click(button(back, T("emu.stop")));
  await until("it to stop", () => textOf(back.querySelector(".emu-state")) === T("emu.notRunning"));
  await click(button(back, T("emu.delete")));
  await click(button(back, T("emu.confirmDelete")));
  await until("it gone from the library", () => ![...back.querySelectorAll(".emu-item")].some((element) => textOf(element).includes(T("emu.newName.http"))));
  expect("stopped and deleted", true);
}

/**
 * An MQTT broker made on its screen: the MQTT screen connects to it, a
 * command published there is routed and answered by the broker's device
 * rule, the exchange is listed, and an outage can be set before it goes.
 */
export async function emulatorMqtt(expect: Expect, args: StepArgs) {
  const shown = await go("emulators");
  await click(button(panel(shown, T("emu.library")), `＋ ${T("emu.new.mqtt")}`));
  const editor = await until("the new broker", () => textOf(shown.querySelector(".emu-item.active")).includes(T("emu.newName.mqtt")) && shown.querySelector<HTMLElement>(".emu-editor"));
  const local = `127.0.0.1:${args.port}`;
  await type(control(editor, T("emu.bind")), local);
  expect("a new broker starts with a device rule", control(editor, T("emu.topicFilter")).value === "lab/+/set", control(editor, T("emu.topicFilter")).value);
  await sleep(600);
  expect("it checks itself: nothing in the way", !shown.querySelector(".emu-problem"), textOf(shown.querySelector(".emu-problem")));
  await click(button(shown, T("emu.start")));
  await until("it to listen", () => textOf(shown.querySelector(".emu-state")) === T("emu.runningOn", { local }));
  const nameless = unnamed(shown);
  expect("the broker's editor: every field and button has a name", nameless.length === 0, nameless.join(" | "));

  // The MQTT screen is its client: a command goes in, the lamp's state comes back.
  const mqtt = await go("mqtt");
  await type(control(mqtt, T("mq.host")), "127.0.0.1");
  await type(control(mqtt, T("common.port")), args.port);
  await click(button(mqtt, T("mq.connect")));
  await until("the connection", () => hasButton(mqtt, T("mq.disconnect")));
  const publisher = panel(mqtt, T("mq.publish"));
  await type(control(publisher, T("mq.topic")), "lab/lamp/set");
  await type(control(publisher, T("sig.payload")), "ON");
  await setChecked(checkbox(publisher, T("mq.retain")), false);
  await click(button(publisher, T("mq.publishBtn")));
  await type(control(mqtt, T("mq.topics")), "lamp");
  const state = await until("the device's answer in the tree", () => [...mqtt.querySelectorAll(".topic-row")].find((row) => textOf(row).includes("lab/lamp/state")));
  expect("the broker routes it, and its rule answers as the lamp", textOf(state).includes("ON"), textOf(state));
  await click(button(mqtt, T("mq.disconnect")));
  await until("the disconnect", () => hasButton(mqtt, T("mq.connect")));

  // What it received, and an outage set before it is taken down.
  const back = await go("emulators");
  const live = panel(back, T("emu.live"));
  const row = await until("the exchange", () => [...live.querySelectorAll("tbody tr")].map(textOf).find((text) => text.includes("lab/lamp/set")));
  expect("the command is listed with its rule and what it published", row.includes("#1") && row.includes("lab/lamp/state"), row);
  await click(button(back, T("emu.stop")));
  await until("it to stop", () => textOf(back.querySelector(".emu-state")) === T("emu.notRunning"));
  const settings = back.querySelector<HTMLElement>(".emu-editor")!;
  await setChecked(checkbox(settings, T("emu.outage")), true);
  expect("an outage has its up and down times", control(settings, T("emu.outageUp")).value === "10000" && control(settings, T("emu.outageDown")).value === "3000");
  await click(button(back, T("emu.delete")));
  await click(button(back, T("emu.confirmDelete")));
  await until("it gone from the library", () => ![...back.querySelectorAll(".emu-item")].some((element) => textOf(element).includes(T("emu.newName.mqtt"))));
  expect("stopped and deleted", true);
}
