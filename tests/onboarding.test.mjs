import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  DONE_KEY, FIRST_STEPS_PAGE, TOUR_STEPS, back, first, isFirst, isLast, keyAction, markDone, next, opensByItself, progress, seek, settle,
} from "../src/lib/onboarding.ts";
import { placeCard, CARD_GAP, TIP_MARGIN, ARROW_INSET } from "../src/lib/tooltip.ts";
import { en } from "../src/lib/locales/en.ts";

const all = () => true;
const without = (...gone) => (target) => !gone.includes(target);
const at = (id) => TOUR_STEPS.findIndex((step) => step.id === id);

test("the steps: a welcome and a finish in the middle, every text in the dictionary, every target in the shell", () => {
  assert.equal(TOUR_STEPS[0].target, null, "the welcome is a card in the middle");
  assert.equal(TOUR_STEPS.at(-1).target, null, "so is the finish");
  assert.equal(new Set(TOUR_STEPS.map((step) => step.id)).size, TOUR_STEPS.length, "ids are unique");
  for (const step of TOUR_STEPS) {
    assert.ok(step.title in en, `${step.id}: ${step.title}`);
    assert.ok(step.text in en, `${step.id}: ${step.text}`);
  }
  // Targets by data-tour, never by a class: each one is written in the shell's source.
  const shell = ["src/App.tsx", "src/components/LanguageMenu.tsx"].map((file) => readFileSync(new URL(`../${file}`, import.meta.url), "utf8")).join("\n");
  for (const { target } of TOUR_STEPS) {
    if (!target) continue;
    const literal = shell.includes(`data-tour="${target}"`);
    const navItem = target.startsWith("nav-") && shell.includes("data-tour={`nav-${n.key}`}") && shell.includes(`key: "${target.slice(4)}"`);
    assert.ok(literal || navItem, `data-tour="${target}" is in the shell`);
  }
  assert.ok(readFileSync(new URL(`../docs/${FIRST_STEPS_PAGE}.md`, import.meta.url), "utf8").length > 0, "the last card's page exists");
});

test("Next walks every step and ends after the last; Back goes back and stops at the first", () => {
  const seen = [];
  for (let index = first(TOUR_STEPS, all); index !== null; index = next(TOUR_STEPS, index, all)) seen.push(TOUR_STEPS[index].id);
  assert.deepEqual(seen, TOUR_STEPS.map((step) => step.id));
  assert.equal(back(TOUR_STEPS, 3, all), 2);
  assert.equal(back(TOUR_STEPS, 0, all), 0, "Back on the first stays there");
  assert.ok(isFirst(TOUR_STEPS, 0, all) && !isFirst(TOUR_STEPS, 1, all));
  assert.ok(isLast(TOUR_STEPS, TOUR_STEPS.length - 1, all) && !isLast(TOUR_STEPS, 0, all));
  assert.deepEqual(progress(TOUR_STEPS, 2, all), { n: 3, total: TOUR_STEPS.length });
});

test("a step whose element is missing or hidden is passed over, both ways, and counted out", () => {
  const shown = without("nav-signals", "language");
  assert.equal(next(TOUR_STEPS, at("experiment"), shown), at("emulators"));
  assert.equal(back(TOUR_STEPS, at("emulators"), shown), at("experiment"));
  assert.equal(next(TOUR_STEPS, at("jobs"), shown), at("docs"));
  assert.deepEqual(progress(TOUR_STEPS, at("emulators"), shown), { n: at("emulators") , total: TOUR_STEPS.length - 2 });
  // Nothing of the shell at all: the two cards in the middle are still a tour that ends.
  const nothing = () => false;
  assert.equal(first(TOUR_STEPS, nothing), 0);
  assert.equal(next(TOUR_STEPS, 0, nothing), TOUR_STEPS.length - 1);
  assert.equal(next(TOUR_STEPS, TOUR_STEPS.length - 1, nothing), null);
  assert.equal(seek(TOUR_STEPS, 1, 1, nothing), TOUR_STEPS.length - 1);
});

test("the step on show losing its element moves on, never to a dead end", () => {
  assert.equal(settle(TOUR_STEPS, at("dock"), all), at("dock"), "still there: stays");
  assert.equal(settle(TOUR_STEPS, at("dock"), without("dock")), at("jobs"), "gone: the next one");
  assert.equal(settle(TOUR_STEPS, at("about"), without("about")), at("finish"));
  const onlyMiddle = () => false;
  for (let index = 0; index < TOUR_STEPS.length; index++) {
    const settled = settle(TOUR_STEPS, index, onlyMiddle);
    assert.ok(settled !== null && TOUR_STEPS[settled].target === null, `${TOUR_STEPS[index].id} settles on a card that can be shown`);
  }
});

test("keys: Enter and the arrow toward the end go on, the other arrow back — mirrored right to left; Esc skips", () => {
  assert.equal(keyAction("Enter", false), "next");
  assert.equal(keyAction("ArrowRight", false), "next");
  assert.equal(keyAction("ArrowLeft", false), "back");
  assert.equal(keyAction("ArrowRight", true), "back");
  assert.equal(keyAction("ArrowLeft", true), "next");
  assert.equal(keyAction("Escape", true), "skip");
  assert.equal(keyAction("a", false), null);
});

test("it opens by itself until finished or skipped; without storage, once a session", () => {
  const store = new Map();
  const storage = { getItem: (key) => store.get(key) ?? null, setItem: (key, value) => store.set(key, value) };
  assert.equal(DONE_KEY, "signal-lab.onboarding.done");
  assert.equal(opensByItself(storage, false), true, "first launch");
  markDone(storage);
  assert.ok(store.has(DONE_KEY));
  assert.equal(opensByItself(storage, false), false, "done: never again by itself");
  const broken = { getItem: () => { throw new Error("denied"); }, setItem: () => { throw new Error("denied"); } };
  assert.equal(opensByItself(broken, false), true, "storage refused: shown");
  assert.doesNotThrow(() => markDone(broken));
  assert.equal(opensByItself(broken, true), false, "…once this session");
  assert.equal(opensByItself(null, false), true);
  assert.equal(opensByItself(null, true), false);
});

const view = { width: 1200, height: 800 };
const card = { width: 360, height: 180 };

test("the card goes on the side asked for, the end side following the reading direction", () => {
  const sidebar = { left: 0, top: 0, width: 80, height: 800 };
  const ltr = placeCard(sidebar, card, view, ["end"], false);
  assert.equal(ltr.side, "right");
  assert.equal(ltr.left, 80 + CARD_GAP);
  assert.equal(ltr.arrow, card.height / 2, "the arrow points at the sidebar's middle");
  const rtlSidebar = { left: 1120, top: 0, width: 80, height: 800 };
  const rtl = placeCard(rtlSidebar, card, view, ["end"], true);
  assert.deepEqual([rtl.side, rtl.left], ["left", 1120 - CARD_GAP - card.width]);
  const button = { left: 1000, top: 10, width: 28, height: 28 };
  const below = placeCard(button, card, view, ["below"]);
  assert.deepEqual([below.side, below.top], ["below", 38 + CARD_GAP]);
  assert.equal(below.left, view.width - TIP_MARGIN - card.width, "kept inside the window");
  assert.equal(below.arrow, 1014 - below.left, "the arrow still points at the button");
});

test("a side with no room is not used; the card never leaves the window", () => {
  const dock = { left: 80, top: 760, width: 1120, height: 40 };
  assert.equal(placeCard(dock, card, view, ["below"]).side, "above", "no room below the bottom panel");
  const huge = { left: 0, top: 0, width: 1200, height: 800 };
  const inside = placeCard(huge, card, view, ["end"]);
  assert.ok(inside.left >= TIP_MARGIN && inside.left + card.width <= view.width - TIP_MARGIN);
  assert.ok(inside.top >= TIP_MARGIN && inside.top + card.height <= view.height - TIP_MARGIN);
  const corner = placeCard({ left: 0, top: 0, width: 4, height: 4 }, card, view, ["below"]);
  assert.equal(corner.arrow, ARROW_INSET, "the arrow keeps off the card's corner");
});
