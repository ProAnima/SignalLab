/**
 * The documentation (docs/, built into dist/docs) as the interface opens it:
 * F1 or the header's button shows the page of the screen in view, in the
 * interface's language. The site's own structure is docs/.vitepress/structure.ts,
 * which takes this list too; tests/docs.test.mjs checks every page exists.
 */

/** The page of each screen (the keys of `NAV` in App.tsx), as `section/page`. */
export const SCREEN_PAGES: Record<string, string> = {
  experiment: "experiments/index",
  signals: "tools/signals",
  emulators: "tools/emulators",
  osc: "protocols/osc",
  mqtt: "protocols/mqtt",
  broadcast: "protocols/broadcast",
  http: "protocols/http",
  ws: "protocols/websocket",
  netsim: "tools/impairment",
  storm: "tools/storm",
  scan: "tools/scanner",
};

/** Where the documentation starts when a screen has no page of its own. */
export const START_PAGE = "guide/index";

/** The published documentation, for builds that have none inside (`npm run dev`). */
export const DOCS_ONLINE = "https://proanima.github.io/SignalLab/";

/** A page's path inside the documentation, in `lang`: `ru/protocols/osc.html`. English is at the root. */
export function docsPath(page: string, lang: string): string {
  return `${lang === "en" ? "" : `${lang}/`}${page}.html`;
}

/** The page F1 opens on `screen`. */
export const screenPage = (screen: string): string => SCREEN_PAGES[screen] ?? START_PAGE;
