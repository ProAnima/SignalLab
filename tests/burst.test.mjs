import test from "node:test";
import assert from "node:assert/strict";
import { fmtRate, latencyParts, setFormatLanguage } from "../src/lib/format.ts";

const EN = { b: "B", kb: "KB", mb: "MB", gb: "GB" };

test("a burst's latency reads as precisely as it is short, in seconds from one", () => {
  setFormatLanguage("en", EN);
  assert.deepEqual(latencyParts(0), ["0.00", "unit.ms"]);
  assert.deepEqual(latencyParts(0.437), ["0.44", "unit.ms"], "a loopback answer is not 0 ms");
  assert.deepEqual(latencyParts(12.34), ["12.3", "unit.ms"]);
  assert.deepEqual(latencyParts(523.4), ["523", "unit.ms"]);
  assert.deepEqual(latencyParts(999.4), ["999", "unit.ms"]);
  assert.deepEqual(latencyParts(1520), ["1.52", "unit.s"]);
  assert.deepEqual(latencyParts(10_000), ["10.0", "unit.s"], "a timeout stays as short as a fast answer");
  setFormatLanguage("ru", { b: "Б", kb: "КБ", mb: "МБ", gb: "ГБ" });
  assert.deepEqual(latencyParts(1520), ["1,52", "unit.s"], "in the reader's decimal sign");
  assert.equal(fmtRate(1500).replace(/\s/g, " "), "1 500");
  setFormatLanguage("en", EN);
});

test("a rate reads as it was typed", () => {
  setFormatLanguage("en", EN);
  assert.equal(fmtRate(50), "50");
  assert.equal(fmtRate(0.5), "0.50");
  assert.equal(fmtRate(0.25), "0.25");
  assert.equal(fmtRate(12.5), "12.5");
  assert.equal(fmtRate(100000), "100,000");
});
