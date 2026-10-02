import test from "node:test";
import assert from "node:assert/strict";
import { AUTH_SCHEMES, withScheme } from "../src/lib/httpAuth.ts";
import { LOCALES } from "../src/lib/locales/index.ts";

test("switching the scheme keeps credentials that carry over and drops the rest", () => {
  const basic = { scheme: "basic", username: "lab", password: "p" };
  assert.deepEqual(withScheme(basic, "digest"), { scheme: "digest", username: "lab", password: "p" }, "Basic ⇄ Digest keep the name and password");
  assert.deepEqual(withScheme(basic, "bearer"), { scheme: "bearer", token: "" }, "a password is not a token");
  assert.deepEqual(withScheme({ scheme: "bearer", token: "t" }, "bearer"), { scheme: "bearer", token: "t" });
  assert.deepEqual(withScheme({ scheme: "bearer", token: "t" }, "basic"), { scheme: "basic", username: "", password: "" });
  assert.deepEqual(withScheme(basic, "none"), { scheme: "none" });
});

test("every scheme has a name in every language", () => {
  for (const { code, dict } of LOCALES) for (const scheme of AUTH_SCHEMES) assert.ok(dict[`http.auth.${scheme}`], `${code}: ${scheme}`);
});
