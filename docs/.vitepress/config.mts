import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import MiniSearch from "minisearch";
import { defineConfig, type DefaultTheme } from "vitepress";
import { LOCALES } from "../../src/lib/locales/index";
import { TEXT } from "./i18n";
import { DEVELOP, SECTIONS } from "./structure";
import { tokenize } from "./tokenize";
import { uiLabels } from "./ui-labels";
import { versionMarks } from "./version";
import { vueSafe } from "./vue-safe";

// The search index in words as each script has them — Chinese and Japanese have no
// spaces between words (tokenize.ts). Not through the search's options: those are
// site data, where a function reaches the browser as code to `eval`, which the
// server's CSP forbids. So the build's own MiniSearch is given it here, and the
// browser reads queries with the same tokenizer (theme/index.ts).
const add = MiniSearch.prototype.add;
MiniSearch.prototype.add = function (this: MiniSearch & { _options: { tokenize: typeof tokenize } }, document) {
  this._options.tokenize = tokenize;
  return add.call(this, document);
};

const docs = join(dirname(fileURLToPath(import.meta.url)), "..");
const version: string = JSON.parse(readFileSync(join(docs, "../package.json"), "utf8")).version;
const REPOSITORY = "https://github.com/ProAnima/SignalLab";

/** `/docs/` inside the app and the server; `/SignalLab/` on GitHub Pages (`DOCS_BASE`). */
const base = process.env.DOCS_BASE ?? "/docs/";

const CODES: string[] = LOCALES.map((locale) => locale.code);
/** A page's language: the first folder when it is one, English otherwise. */
const languageOf = (relativePath: string) => {
  const first = relativePath.split(/[\\/]/)[0];
  return CODES.includes(first) && first !== "en" ? first : "en";
};

/** A page's `title:` from its front matter: the sidebar says what the page says it is. */
function titleOf(page: string): string {
  const file = join(docs, `${page}.md`);
  if (!existsSync(file)) throw new Error(`docs/${page}.md is missing (generated pages: node scripts/docs.mjs generate)`);
  const front = /^---\r?\n([\s\S]*?)\r?\n---/.exec(readFileSync(file, "utf8"));
  const title = front && /^title:\s*(.+)$/m.exec(front[1]);
  if (!title) throw new Error(`docs/${page}.md has no title in its front matter`);
  return title[1].trim().replace(/^(["'])(.*)\1$/, "$2");
}

const linkTo = (page: string) => `/${page.replace(/(^|\/)index$/, "$1")}`;

function sidebar(code: string): DefaultTheme.SidebarItem[] {
  const prefix = code === "en" ? "" : `${code}/`;
  const text = TEXT[code];
  return [
    ...SECTIONS.map((section) => ({
      text: text.sections[section.id as keyof typeof text.sections],
      collapsed: false,
      items: section.pages.map((page) => ({ text: titleOf(`${prefix}${section.id}/${page}`), link: linkTo(`${prefix}${section.id}/${page}`) })),
    })),
    {
      text: code === "en" ? text.sections.develop : `${text.sections.develop} (${text.englishOnly})`,
      collapsed: true,
      items: DEVELOP.pages.map((page) => ({ text: titleOf(`develop/${page}`), link: linkTo(`develop/${page}`) })),
    },
  ];
}

function themeOf(code: string): DefaultTheme.Config {
  const text = TEXT[code];
  const home = code === "en" ? "/" : `/${code}/`;
  return {
    nav: [
      { text: text.sections.guide, link: `${home}guide/` },
      { text: text.sections.api, link: `${home}api/` },
      { text: `${text.download} ${version}`, link: `${REPOSITORY}/releases/latest` },
    ],
    sidebar: sidebar(code),
    outline: { level: [2, 3], label: text.outline },
    docFooter: { prev: text.prev, next: text.next },
    returnToTopLabel: text.returnToTop,
    sidebarMenuLabel: text.sidebarMenu,
    darkModeSwitchLabel: text.appearance,
    lightModeSwitchTitle: text.lightTheme,
    darkModeSwitchTitle: text.darkTheme,
    langMenuLabel: text.langMenu,
    skipToContentLabel: text.skipToContent,
    editLink: { pattern: `${REPOSITORY}/edit/main/docs/:path`, text: text.editLink },
    notFound: { title: text.notFound.title, quote: text.notFound.quote, linkText: text.notFound.link, linkLabel: text.notFound.linkLabel },
  };
}

const searchText = (code: string) => {
  const text = TEXT[code].search;
  return {
    translations: {
      button: { buttonText: text.button, buttonAriaLabel: text.button },
      modal: {
        displayDetails: text.details,
        resetButtonTitle: text.reset,
        backButtonTitle: text.back,
        noResultsText: text.noResults,
        footer: { selectText: text.select, selectKeyAriaLabel: "Enter", navigateText: text.navigate, navigateUpKeyAriaLabel: "↑", navigateDownKeyAriaLabel: "↓", closeText: text.close, closeKeyAriaLabel: "Esc" },
      },
    },
  };
};

const key = (code: string) => (code === "en" ? "root" : code);

export default defineConfig({
  base,
  srcDir: ".",
  outDir: process.env.DOCS_OUT ?? ".vitepress/dist",
  cacheDir: ".vitepress/cache",
  // The page map in a file of its own: the server's CSP allows no inline script.
  metaChunk: true,
  cleanUrls: false,
  appearance: "dark",
  lastUpdated: false,
  title: "Signal Lab",
  head: [["link", { rel: "icon", type: "image/png", href: `${base}icon.png` }]],
  // Pages that are not pages: the API description and the old design notes' sources.
  srcExclude: ["**/README.md", "api/openapi.json"],
  locales: Object.fromEntries(LOCALES.map((locale) => [key(locale.code), {
    label: locale.name,
    lang: locale.code,
    dir: "dir" in locale ? locale.dir : "ltr",
    link: locale.code === "en" ? "/" : `/${locale.code}/`,
    description: TEXT[locale.code].description,
    themeConfig: themeOf(locale.code),
  }])),
  themeConfig: {
    logo: { src: "/icon.png", alt: "" },
    siteTitle: "Signal Lab",
    socialLinks: [{ icon: "github", link: REPOSITORY }],
    search: {
      provider: "local",
      options: {
        locales: Object.fromEntries(CODES.map((code) => [key(code), searchText(code)])),
      },
    },
  },
  markdown: {
    config: (md) => {
      versionMarks(md, version);
      uiLabels(md, Object.fromEntries(LOCALES.map((locale) => [locale.code, locale.dict as unknown as Record<string, string>])), languageOf);
      vueSafe(md);
    },
  },
});
