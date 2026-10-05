/**
 * The documentation's pages, in reading order: one list for every language.
 * A page's title is its own `title:` in each language's file; a section's name
 * is in `i18n.ts`. `tests/docs.test.mjs` checks that every language has every
 * page here and nothing else, and that `develop/` stays English.
 */
export interface Section {
  /** The folder, and the key of its name in `i18n.ts`. */
  id: string;
  pages: string[];
}

export const SECTIONS: Section[] = [
  { id: "guide", pages: ["index", "install", "interface", "first-steps", "concepts"] },
  { id: "protocols", pages: ["osc", "udp-tcp", "http", "websocket", "mqtt", "broadcast"] },
  { id: "tools", pages: ["signals", "inspector", "emulators", "impairment", "storm", "scanner"] },
  { id: "experiments", pages: ["index", "nodes", "data", "flow", "load", "faults", "runs"] },
  { id: "automation", pages: ["cli", "ci", "mcp"] },
  { id: "server", pages: ["index", "security"] },
  { id: "api", pages: ["index", "commands", "events", "run"] },
  { id: "reference", pages: ["shortcuts", "errors", "files", "troubleshooting", "changelog"] },
];

/** For contributors, in English only: building, checking, delivering, the design. */
export const DEVELOP: Section = {
  id: "develop",
  pages: ["index", "building", "writing-docs", "delivery", "localization", "hub", "roadmap", "design-data", "design-reactive", "design-load", "design-declarations"],
};

/** Pages written by `scripts/docs.mjs` before a build, from the interface's dictionaries (not in git). */
export const GENERATED = ["reference/errors"];

/** Which page each screen of the app opens with F1: the interface's own list. */
export { SCREEN_PAGES } from "../../src/lib/docs.ts";
