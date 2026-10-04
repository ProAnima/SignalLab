import {
  createContext, useCallback, useContext, useEffect, useMemo, useState,
  type ReactNode,
} from "react";
import { en, type TKey } from "./locales/en";
import { CODES, dictOf, dirOf, intlOf, LOCALES, SOURCE, type Lang } from "./locales/index";
import { format, pickLanguage, type Params } from "./translate";
import { setFormatLanguage } from "./format";

export type { TKey, Lang };

/** The languages, for the switch: code, own name, two letters, which way it reads. */
export const LANGS = LOCALES.map((locale) => ({ code: locale.code, label: locale.name, short: locale.short, dir: dirOf(locale.code) }));

const STORAGE_KEY = "signal-lab.lang";

/**
 * Any dictionary key, or free text. Console lines carry raw error strings from
 * the engine, which are not translatable — `t` passes unknown keys through
 * unchanged, so both work. The `& {}` keeps autocomplete alive for real keys.
 */
export type TextKey = TKey | (string & {});

export type Translate = (key: TextKey, params?: Params) => string;

/** The saved choice, else the first of the system's languages the interface has, else English. */
function detect(): Lang {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    const known = CODES.find((code) => code === saved);
    if (known) return known;
  } catch {
    // localStorage can be unavailable in a locked-down webview; fall through.
  }
  const preferred = navigator.languages?.length ? navigator.languages : [navigator.language ?? ""];
  return pickLanguage(preferred, CODES, SOURCE);
}

/** `t` for one language, outside React too (tests, one-off texts). */
export function translator(lang: Lang): Translate {
  const dict = dictOf(lang) as Record<string, string | undefined>;
  const source = en as Record<string, string | undefined>;
  // Unknown key → the source language, then the key itself (which is how raw
  // engine strings pass through untouched).
  const intl = intlOf(lang);
  return (key, params) => format(dict[key] ?? source[key] ?? key, params, intl);
}

interface I18n {
  lang: Lang;
  setLang: (lang: Lang) => void;
  t: Translate;
}

const Ctx = createContext<I18n | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(detect);

  useEffect(() => {
    document.documentElement.lang = lang;
    // A right-to-left language mirrors the page; styles/rtl.css keeps the data in it left to right.
    document.documentElement.dir = dirOf(lang);
  }, [lang]);

  const setLang = useCallback((next: Lang) => {
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // Preference just won't persist; switching still works this session.
    }
    setLangState(next);
  }, []);

  const t = useMemo(() => translator(lang), [lang]);
  // Before the children render, so their numbers and sizes are in this language.
  setFormatLanguage(intlOf(lang), { b: t("unit.b"), kb: t("unit.kb"), mb: t("unit.mb"), gb: t("unit.gb") });

  const value = useMemo(() => ({ lang, setLang, t }), [lang, setLang, t]);
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useI18n(): I18n {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("useI18n must be used within I18nProvider");
  return ctx;
}

/** Shorthand for the common case of only needing the translate function. */
export function useT(): Translate {
  return useI18n().t;
}
