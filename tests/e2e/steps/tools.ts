/** The tool screens, each against its loopback fixture: OSC, MQTT, Broadcast, Network simulator, Storm, Scanner, HTTP. */
import { T, sleep, textOf, numberIn, until, screen, control, checkbox, button, hasButton, buttonWith, metric, panel, unnamed, click, type, setChecked, key, go, result, type Key, type StepArgs, type Expect } from "../dsl";

export async function osc(expect: Expect, args: StepArgs) {
  const shown = await go("osc");
  const monitor = panel(shown, T("osc.monitor"));
  await type(control(monitor, T("common.bind")), `127.0.0.1:${args.port}`);
  await click(button(monitor, T("osc.listen")));
  await until("the monitor to listen", () => hasButton(monitor, T("common.stop")));
  expect("the monitor runs as a job", !!document.querySelector(".job-pill"));

  const sender = panel(shown, T("osc.sender"));
  await type(control(sender, T("common.target")), `127.0.0.1:${args.port}`);
  await type(control(sender, T("common.address")), "/e2e/osc");
  // Start from one float argument and add a string, through the argument editor.
  while (sender.querySelectorAll(".row.tight").length > 1) await click(sender.querySelector<HTMLButtonElement>(".row.tight button")!, "remove argument");
  if (!sender.querySelector(".row.tight")) await click(button(sender, T("common.addArgument")));
  const first = sender.querySelector(".row.tight")!;
  await type(first.querySelector("select")!, "float");
  await type(first.querySelector("input")!, "0.5");
  await click(button(sender, T("common.addArgument")));
  const second = sender.querySelectorAll(".row.tight")[1];
  await type(second.querySelector("select")!, "str");
  await type(second.querySelector("input")!, "hello e2e");
  await click(button(sender, T("common.send")));
  const sent = await until("the send result", () => result(sender)?.classList.contains("ok") && result(sender));
  expect("Send reports the bytes and the target", textOf(sent).includes(`/e2e/osc · `) && textOf(sent).includes(`127.0.0.1:${args.port}`), textOf(sent));
  const row = await until("the message in the monitor", () => [...monitor.querySelectorAll("tbody tr")].find((tr) => textOf(tr).includes("/e2e/osc")));
  expect("the monitor decodes both arguments", textOf(row).includes("0.5") && textOf(row).includes("hello e2e"), textOf(row));

  // Enter in the address field sends again; the result counts the repeat.
  key(control(sender, T("common.address")), { key: "Enter", code: "Enter" });
  await until("a repeat count", () => textOf(result(sender)).includes("×2"));
  expect("Enter sends again (×2)", true);

  // The generator, into the same monitor.
  const generator = panel(shown, T("osc.generator"));
  await type(control(generator, T("common.target")), `127.0.0.1:${args.port}`);
  await type(control(generator, T("osc.address")), "/e2e/lfo");
  await type(control(generator, T("osc.rate")), 20);
  await click(button(generator, T("osc.startGen")));
  await until("generated messages", () => [...monitor.querySelectorAll("tbody tr")].some((tr) => textOf(tr).includes("/e2e/lfo")));
  const canvas = generator.querySelector("canvas");
  expect("the generator's scope draws", !!canvas && canvas.width > 0);
  await click(button(generator, T("osc.stopGen")));
  await until("the generator to stop", () => hasButton(generator, T("osc.startGen")));
  expect("the generator stops", true);

  // Wait for this: the message becomes a Wait for OSC in the experiment.
  const waitButton = [...monitor.querySelectorAll("tbody tr")].find((tr) => textOf(tr).includes("/e2e/osc"))?.querySelector<HTMLButtonElement>(`button[aria-label="${T("osc.waitForThis")}"]`);
  if (!waitButton) throw new Error("no Wait for this on the message");
  await click(waitButton, "Wait for this");
  const editor = await until("the experiment editor", () => !document.querySelector<HTMLElement>(".experiment-host")!.hidden && document.querySelector(".experiment-properties h2"));
  expect("Wait for this adds a Wait for OSC", textOf(editor) === T("exp.node.wait_osc"), textOf(editor));
  const properties = document.querySelector(".experiment-properties")!;
  expect("…on the monitor's port, for this address", control(properties, T("exp.listenOn")).value === `127.0.0.1:${args.port}` && control(properties, T("exp.addressPattern")).value === "/e2e/osc");
  expect("…with the string argument as a rule", [...properties.querySelectorAll<HTMLInputElement>(".experiment-rule input")].some((input) => input.value === "hello e2e"));
  await click(button(properties, T("exp.delete")));
  await go("osc");
  // The monitor keeps running for the Signals step.
  expect("the monitor survived the trip", hasButton(panel(screen(), T("osc.monitor")), T("common.stop")) !== null);
}

export async function oscStop(expect: Expect) {
  const shown = await go("osc");
  const monitor = panel(shown, T("osc.monitor"));
  await until("the signal in the monitor", () => [...monitor.querySelectorAll("tbody tr")].some((tr) => textOf(tr).includes("/e2e/signal")));
  expect("the library signal reached the monitor", true);
  await click(button(monitor, T("common.stop")));
  await until("the monitor to stop", () => hasButton(monitor, T("osc.listen")));
  expect("the monitor stops", true);
}

export async function mqtt(expect: Expect, args: StepArgs) {
  const shown = await go("mqtt");
  await type(control(shown, T("mq.host")), "127.0.0.1");
  await type(control(shown, T("common.port")), args.port);
  await click(button(shown, T("mq.connect")));
  await until("the connection", () => hasButton(shown, T("mq.disconnect")));
  await until("the # subscription", () => textOf(shown).includes(T("mq.subscriptions")) && textOf(shown).includes("qos0"));
  expect("connects and subscribes to #", true);

  const publisher = panel(shown, T("mq.publish"));
  await type(control(publisher, T("mq.topic")), "e2e/lights/hall");
  await type(control(publisher, T("sig.payload")), "on");
  await setChecked(checkbox(publisher, T("mq.retain")), true);
  await click(button(publisher, T("mq.publishBtn")));
  await type(control(shown, T("mq.topics")), "hall");
  const found = await until("the topic in the tree", () => [...shown.querySelectorAll<HTMLButtonElement>(".topic-row")].find((row) => textOf(row).includes("e2e/lights/hall")));
  expect("the published topic comes back with its value", textOf(found).includes("on"), textOf(found));
  await click(found, "the topic");
  const detail = await until("the topic's panel", () => [...shown.querySelectorAll(".panel")].find((element) => textOf(element.querySelector(".section-label")) === "e2e/lights/hall"));

  // A new subscription gets the broker's retained value, marked as retained.
  await type(control(shown, T("mq.addSubscription")), "e2e/+/hall");
  await click(button(shown, T("mq.subscribe")));
  await until("the second subscription", () => textOf(shown).includes("e2e/+/hall"));
  await until("the retained replay", () => textOf(detail).includes(`${T("mq.retain")} ${T("mq.yes")}`));
  expect("a new subscription receives the retained value, marked R", [...shown.querySelectorAll(".topic-row")].some((row) => textOf(row).includes("e2e/lights/hall") && textOf(row).includes("R")));
  const drop = [...shown.querySelectorAll(".row.tight")].find((row) => textOf(row).includes("e2e/+/hall"))?.querySelector<HTMLButtonElement>("button");
  if (!drop) throw new Error("no drop button on the subscription");
  await click(drop, "drop the subscription");
  await until("the subscription to go", () => !textOf(shown).includes("e2e/+/hall"));
  expect("subscribe and unsubscribe", true);

  await click(buttonWith(detail, T("mq.waitForThis")), "Wait for this");
  const editor = await until("the experiment editor", () => !document.querySelector<HTMLElement>(".experiment-host")!.hidden && document.querySelector(".experiment-properties h2"));
  expect("Wait for this adds a Wait for MQTT", textOf(editor) === T("exp.node.wait_mqtt"), textOf(editor));
  const properties = document.querySelector(".experiment-properties")!;
  expect("…on this topic", control(properties, T("exp.topicFilter")).value === "e2e/lights/hall");
  await click(button(properties, T("exp.delete")));

  const back = await go("mqtt");
  const again = [...back.querySelectorAll(".panel")].find((element) => textOf(element.querySelector(".section-label")) === "e2e/lights/hall")!;
  await click(button(again, T("mq.clearRetained")));
  await click(button(again, T("mq.clearConfirm")));
  await sleep(300);
  await click(button(back, T("mq.disconnect")));
  await until("the disconnect", () => hasButton(back, T("mq.connect")));
  expect("disconnects", true);
}

export async function broadcast(expect: Expect, args: StepArgs) {
  const shown = await go("broadcast");
  await click(button(shown, T("bc.mode.list")));
  await type(control(shown, T("bc.targetList")), `127.0.0.1:${args.port}`);
  const discovery = panel(shown, T("bc.discovery"));
  await type(control(discovery, T("common.bind")), `127.0.0.1:${args.port}`);
  await type(control(discovery, T("bc.joinGroups")), "");
  await setChecked(checkbox(discovery, T("bc.respond")), true);
  await click(button(discovery, T("bc.startListen")));
  await until("discovery to listen", () => hasButton(discovery, T("bc.stopListen")));

  await click(button(shown, T("bc.sendOnce")));
  await until("the emit result", () => shown.querySelector(".metrics"));
  expect("Send once reaches one target without errors", metric(shown, T("bc.targets")) === "1" && metric(shown, T("common.errors")) === "0",
    `${metric(shown, T("bc.targets"))} targets, ${metric(shown, T("common.errors"))} errors`);
  const peer = await until("the probe in discovery", () => [...discovery.querySelectorAll("tbody tr")].find((tr) => textOf(tr).includes("127.0.0.1:")));
  expect("discovery sees the probe as OSC", textOf(peer).includes("osc"), textOf(peer));
  await until("the answer", () => Array.from({ length: 50 }, (_, k) => T("bc.peersReplies", { n: k + 1 })).some((text) => textOf(discovery).includes(text)));
  expect("…and answers it", true);

  await type(control(shown, T("bc.beaconRate")), 10);
  await click(button(shown, T("bc.startBeacon")));
  await until("beacon rounds", () => numberIn(metric(shown, T("bc.rounds"))) >= 3, 6000);
  expect("the beacon repeats", true);
  await click(button(shown, T("bc.stopBeacon")));
  await click(button(discovery, T("bc.stopListen")));
  await until("both to stop", () => hasButton(shown, T("bc.startBeacon")) && hasButton(discovery, T("bc.startListen")));
}

export async function netsimStart(expect: Expect, args: StepArgs) {
  const shown = await go("netsim");
  await type(control(shown, T("ns.listen")), `127.0.0.1:${args.relay}`);
  await type(control(shown, T("ns.target")), `127.0.0.1:${args.sink}`);
  for (const [label, value] of [["ns.latency", 10], ["ns.jitter", 0], ["ns.loss", 0], ["ns.duplicate", 0], ["ns.corrupt", 0]] as [Key, number][]) {
    await type(control(shown, T(label)), value);
  }
  await click(button(shown, T("ns.startRelay")));
  await until("the relay", () => hasButton(shown, T("ns.stopRelay")));
  expect("the relay starts", true);
}

export async function netsimCheck(expect: Expect, args: StepArgs) {
  const shown = screen();
  await until(`${args.n} forwarded`, () => numberIn(metric(shown, T("ns.forwarded"))) === args.n);
  expect(`forwards all ${args.n} datagrams with no loss configured`, metric(shown, T("ns.dropped")) === "0");
  // A preset while it runs: applied at once, without dropping the port.
  await click(button(shown, T("ns.preset.offline")));
  await until("the relay to go offline", () => textOf(shown).includes(T("ns.now", { profile: T("ns.preset.offline") })));
  expect("a profile changes while the relay runs", true);
  await click(button(shown, T("ns.stopRelay")));
  await until("the relay to stop", () => hasButton(shown, T("ns.startRelay")));
  return { forwarded: numberIn(metric(shown, T("ns.forwarded"))) };
}

/**
 * A TCP relay in front of the fixture's API: the profile shows a stream's
 * fates, not a datagram's; a request sent through it from the HTTP screen is
 * answered, 150 ms later each way; the relay counts the connection.
 */
export async function netsimTcp(expect: Expect, args: StepArgs) {
  const shown = await go("netsim");
  await type(control(shown, T("ns.protocol")), "tcp");
  const labels = [...shown.querySelectorAll("label")].map(textOf);
  expect("a TCP relay's profile: resets and half-open connections, no packet loss", labels.some((label) => label.startsWith(T("ns.reset"))) && labels.some((label) => label.startsWith(T("ns.stall"))) && !labels.some((label) => label.startsWith(T("ns.loss"))), labels.join(" | "));
  await type(control(shown, T("ns.listen")), `127.0.0.1:${args.relay}`);
  await type(control(shown, T("ns.target")), `127.0.0.1:${args.port}`);
  await click(button(shown, T("ns.preset.lan")));
  await type(control(shown, T("ns.latency")), 150);
  await type(control(shown, T("ns.jitter")), 0);
  const nameless = unnamed(shown);
  expect("every field has a name", nameless.length === 0, nameless.join(" | "));
  await click(button(shown, T("ns.startRelay")));
  await until("the relay", () => hasButton(shown, T("ns.stopRelay")));

  const http = await go("http");
  const request = panel(http, T("http.request"));
  await type(request.querySelector("select")!, "GET");
  await type(control(request, "URL"), `http://127.0.0.1:${args.relay}/e2e/through-tcp-relay`);
  const before = textOf(result(request));
  await click(button(request, T("common.send")));
  const verdict = await until("the answer through the relay", () => result(request)?.classList.contains("ok") && textOf(result(request)) !== before && result(request));
  const ms = Number(/· ([\d,.]+) ms/.exec(textOf(verdict))?.[1]?.replace(/,/g, "") ?? NaN);
  expect("a request through the TCP relay is answered, 150 ms later each way", textOf(verdict).startsWith("200") && ms >= 290, textOf(verdict));

  const relay = await go("netsim");
  // Its counters come four times a second: the request and its answer, both ways.
  await until("the request and its answer counted", () => numberIn(metric(relay, T("ns.forwarded"))) >= 2);
  expect("the relay counts the connection and what it forwarded", numberIn(metric(relay, T("ns.connections"))) === 1, textOf(relay.querySelector(".metrics")));
  await click(button(relay, T("ns.stopRelay")));
  await until("the relay to stop", () => hasButton(relay, T("ns.startRelay")));
  await type(control(relay, T("ns.protocol")), "udp");
}

export async function storm(expect: Expect, args: StepArgs) {
  const shown = await go("storm");
  const run = async (protocol: "udp" | "tcp", port: number, size: number) => {
    await type(control(shown, T("common.target")), `127.0.0.1:${port}`);
    await type(control(shown, T("common.protocol")), protocol);
    await type(control(shown, T("st.payloadSize")), size);
    await type(control(shown, T("st.rate")), 200);
    await type(control(shown, T("st.duration")), 1);
    await click(button(shown, T("st.launch")));
    await until("the storm to start", () => hasButton(shown, T("st.stop")));
    await until("the storm to end by itself", () => hasButton(shown, T("st.launch")), 8000);
    return { packets: numberIn(metric(shown, T("common.packets"))), errors: numberIn(metric(shown, T("common.errors"))) };
  };
  // Datagrams larger than the Inspector's preview, which inspectWhole then reads whole.
  const udp = await run("udp", args.sink, 3000);
  expect("a 1 s UDP storm at 200 pps sends about 200", udp.packets >= 150 && udp.packets <= 260 && udp.errors === 0, JSON.stringify(udp));
  const tcp = await run("tcp", args.tcp, 64);
  expect("a TCP storm sends without errors", tcp.packets > 0 && tcp.errors === 0, JSON.stringify(tcp));
  return { udp: udp.packets, tcp: tcp.packets };
}

export async function scan(expect: Expect, args: StepArgs) {
  const shown = await go("scan");
  await type(control(shown, T("sc.host")), "127.0.0.1");
  await type(control(shown, T("sc.fromPort")), args.port - 2);
  await type(control(shown, T("sc.toPort")), args.port + 2);
  await type(control(shown, T("common.timeoutMs")), 300);
  await click(button(shown, T("sc.startScan")));
  await until("the scan to finish", () => textOf(shown).includes("5 / 5 (100%)") && hasButton(shown, T("sc.startScan")), 10000);
  const open = [...shown.querySelectorAll("tbody tr")].map((tr) => textOf(tr.querySelector("td")));
  expect(`finds the open port ${args.port}`, open.includes(String(args.port)), open.join(", "));
}

export async function http(expect: Expect, args: StepArgs) {
  const shown = await go("http");
  const request = panel(shown, T("http.request"));
  await type(request.querySelector("select")!, "GET");
  await type(control(request, "URL"), `http://127.0.0.1:${args.port}/e2e?from=tour`);
  await click(button(request, T("common.send")));
  const verdict = await until("the response", () => result(request)?.classList.contains("ok") && result(request));
  expect("GET answers 200", textOf(verdict).startsWith("200"), textOf(verdict));
  const response = panel(shown, T("http.response"));
  const body = response.querySelector<HTMLTextAreaElement>("textarea")!;
  expect("the JSON body is shown formatted", body.value.includes('"ok": true') && body.value.includes('"path": "/e2e?from=tour"'), body.value.slice(0, 120));
  await click(buttonWith(response, T("http.responseHeaders", { n: 5 }).replace("5", "").trim()));
  expect("response headers open", textOf(response).includes("x-fixture"));
  await click(button(response, T("http.rawBody")));
  expect("raw shows the body as it came", !body.value.includes('\n  "ok"'));
  await click(button(response, T("http.formatJson")));

  await type(request.querySelector("select")!, "POST");
  await type(control(request, T("http.body")), '{"e2e":1}');
  await click(button(request, T("common.send")));
  await until("the POST response", () => body.value.includes('"method": "POST"'));
  expect("POST sends its body", body.value.includes('\\"e2e\\":1') || body.value.includes('{\\"e2e\\":1}'), body.value.slice(0, 200));
  await type(request.querySelector("select")!, "GET");

  const burst = panel(shown, T("http.burst"));
  await type(control(burst, T("common.concurrency")), 4);
  await type(control(burst, T("http.total")), 40);
  await type(control(burst, T("http.duration")), 0);
  await click(button(burst, T("http.startBurst")));
  await until("the burst to finish", () => numberIn(metric(burst, T("http.sent"))) === 40 && hasButton(burst, T("http.startBurst")), 15000);
  expect("a burst of 40 all succeed", metric(burst, T("http.ok")) === "40" && metric(burst, T("http.failed")) === "0",
    `${metric(burst, T("http.ok"))} ok, ${metric(burst, T("http.failed"))} failed`);
  // In ms; from a second on the screen says seconds.
  const latency = (caption: string) => {
    const text = metric(burst, caption);
    return numberIn(text) * (text.endsWith(T("unit.ms")) ? 1 : 1000);
  };
  const latencies = [T("http.p50"), T("http.p90"), T("http.p95"), T("http.p99")].map(latency);
  const [least, most] = [T("http.min"), T("http.max")].map(latency);
  expect("its percentiles are read and in order", latencies[0] > 0 && [least, ...latencies, most].every((value, i, all) => i === 0 || all[i - 1] <= value),
    `${least} ≤ ${latencies.join(" ≤ ")} ≤ ${most}`);
  expect("an unpaced burst misses nothing, so says nothing about it", !textOf(burst).includes(T("http.missed")));

  // At a rate: 20 at 50/s take ~0.4 s, and every one starts on time.
  await type(control(burst, T("http.total")), 20);
  await type(control(burst, T("http.rate")), 50);
  await click(button(burst, T("http.startBurst")));
  await until("the paced burst to finish", () => numberIn(metric(burst, T("http.sent"))) === 20 && hasButton(burst, T("http.startBurst")), 15000);
  const achieved = numberIn(metric(burst, T("http.rpsOf", { rate: "50" })));
  expect("a paced burst keeps its rate and misses nothing", achieved > 20 && achieved <= 60 && metric(burst, T("http.missed")) === "0",
    `${achieved} rps, ${metric(burst, T("http.missed"))} missed`);
  await type(control(burst, T("http.rate")), 0);

  // Digest: the fixture checks the answer with its own hashing.
  await type(control(request, "URL"), `http://127.0.0.1:${args.port}/digest/screen`);
  await type(control(request, T("field.auth")), "digest");
  await type(control(request, T("field.username")), "tour");
  await type(control(request, T("field.password")), "e2e-secret");
  const authNameless = unnamed(request);
  expect("the authentication fields have names", authNameless.length === 0, authNameless.join(" | "));
  await click(button(request, T("common.send")));
  const answered = await until("the Digest answer", () => result(request)?.classList.contains("ok") && result(request), 10000);
  expect("Digest answers the server's challenge and gets 200", textOf(answered).startsWith("200"), textOf(answered));
  expect("…and the response says it was challenged", textOf(response).includes(T("http.digestAnswered")), textOf(response).slice(0, 200));
  await type(control(request, T("field.auth")), "none");

  // The screen's cookie jar: /login sets one, /me needs it, Clear forgets it.
  const jarPanel = () => [...shown.querySelectorAll<HTMLElement>(".panel")].find((element) => textOf(element.querySelector(".section-label")).startsWith(T("http.cookies", { n: 0 })))!;
  await type(control(request, "URL"), `http://127.0.0.1:${args.port}/login`);
  await click(button(request, T("common.send")));
  await until("the cookie", () => textOf(jarPanel()).includes("tour"), 10000);
  expect("the jar lists what the server set", textOf(jarPanel().querySelector("tbody")).includes("HttpOnly"), textOf(jarPanel()));
  await type(control(request, "URL"), `http://127.0.0.1:${args.port}/me`);
  await click(button(request, T("common.send")));
  const me = await until("the answer to /me", () => result(request)?.classList.contains("ok") && result(request), 10000);
  expect("a later request carries the cookie", textOf(me).startsWith("200"), textOf(me));
  await click(button(jarPanel(), T("common.clear")));
  await until("the jar emptied", () => textOf(jarPanel()).includes(T("http.noCookies")));
  await click(button(request, T("common.send")));
  await until("refused without it", () => result(request)?.classList.contains("warn"), 10000);
  expect("after Clear it is refused again", textOf(result(request)).startsWith("401"), textOf(result(request)));

  await click(button(request, T("common.toExperiment")));
  const editor = await until("the experiment editor", () => !document.querySelector<HTMLElement>(".experiment-host")!.hidden && document.querySelector(".experiment-properties h2"));
  expect("To experiment adds the request as a node", textOf(editor) === T("exp.node.http"));
  await click(button(document.querySelector(".experiment-properties")!, T("exp.delete")));
}
