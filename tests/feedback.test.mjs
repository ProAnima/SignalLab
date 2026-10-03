import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { LIMITS, nameOf, refusalOf, scrub } from "../src/lib/feedback.ts";

test("the form's limits are the engine's, which are the hub's", () => {
  const engine = readFileSync("engine/src/feedback.rs", "utf8");
  const constant = (name) => {
    const value = new RegExp(`pub const ${name}: usize = ([^;]+);`).exec(engine)?.[1];
    return Function(`return (${value.replace(/_/g, "")})`)();
  };
  assert.equal(LIMITS.messageChars, constant("MAX_MESSAGE_CHARS"));
  assert.equal(LIMITS.images, constant("MAX_IMAGES"));
  assert.equal(LIMITS.imageBytes, constant("MAX_IMAGE_BYTES"));
  assert.equal(LIMITS.totalBytes, constant("MAX_TOTAL_BYTES"));
});

test("a screenshot is refused here before anything is uploaded", () => {
  const png = (size, name = "a.png") => ({ name, size, type: "image/png" });
  assert.equal(refusalOf(png(1000), []), null);
  assert.equal(refusalOf({ name: "a.svg", size: 10, type: "image/svg+xml" }, [])?.key, "feedback.notImage");
  assert.deepEqual(refusalOf(png(LIMITS.imageBytes + 1, "big.png"), []), { key: "feedback.imageTooLarge", params: { name: "big.png", mb: 8 } });
  const six = Array.from({ length: LIMITS.images }, (_, index) => ({ id: String(index), name: "x", size: 1, data: "", url: "" }));
  assert.equal(refusalOf(png(1), six)?.key, "feedback.tooManyImages");
  const heavy = [{ id: "1", name: "x", size: 7 << 20, data: "", url: "" }, { id: "2", name: "y", size: 7 << 20, data: "", url: "" }];
  assert.equal(refusalOf(png(2 << 20), heavy)?.key, "feedback.tooLarge");
});

test("a pasted image is named for when it was pasted; a picked file keeps its name", () => {
  const at = new Date(2026, 9, 3, 9, 5, 7);
  assert.equal(nameOf({ name: "image.png", type: "image/png" }, true, at), "screenshot-2026-10-03-090507.png");
  assert.equal(nameOf({ name: "", type: "image/jpeg" }, false, at), "screenshot-2026-10-03-090507.jpg");
  assert.equal(nameOf({ name: "scope.png", type: "image/png" }, false, at), "scope.png");
});

test("the log leaves out this computer's name, its address and the user's folders", () => {
  const where = { host: { hostname: "SRV-IAN-5", local_ip: "192.168.1.40" }, dataDir: "C:\\Users\\Ян\\Documents\\SignalLab" };
  const log = [
    "10 signals from C:\\Users\\Ян\\Documents\\SignalLab\\signals.json",
    "report saved: C:\\Users\\Ян\\Desktop\\run.json",
    "monitor on 192.168.1.40:9000 (SRV-IAN-5)",
    "sent to 192.168.1.20:9000",
    "/home/ian/.config/x and /Users/ian/Library/y",
  ].join("\n");
  assert.equal(scrub(log, where), [
    "10 signals from {data}\\signals.json",
    "report saved: C:\\Users\\…\\Desktop\\run.json",
    "monitor on {this-ip}:9000 ({host})",
    "sent to 192.168.1.20:9000",
    "/home/…/.config/x and /Users/…/Library/y",
  ].join("\n"), "a target the person sent to stays: it is what the developers need");
  assert.equal(scrub("on 127.0.0.1:9000", { host: { hostname: "x", local_ip: "127.0.0.1" } }), "on 127.0.0.1:9000", "loopback and a one-letter name are nobody's");
});
