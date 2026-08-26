import {
  createContext, useCallback, useContext, useEffect, useMemo, useState,
  type ReactNode,
} from "react";
import { en, type Dict, type TKey } from "./locales/en";
import { ru } from "./locales/ru";

export type { TKey };

export type Lang = "en" | "ru";

export const LANGS: { code: Lang; label: string; short: string }[] = [
  { code: "en", label: "English", short: "EN" },
  { code: "ru", label: "Русский", short: "RU" },
];

const DICTS: Record<Lang, Dict> = { en, ru };
const STORAGE_KEY = "signal-lab.lang";

/**
 * Any dictionary key, or free text. Console lines carry raw error strings from
 * the engine, which are not translatable — `t` passes unknown keys through
 * unchanged, so both work. The `& {}` keeps autocomplete alive for real keys.
 */
export type TextKey = TKey | (string & {});

export type Translate = (key: TextKey, params?: Record<string, string | number>) => string;

function detect(): Lang {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "en" || saved === "ru") return saved;
  } catch {
    // localStorage can be unavailable in a locked-down webview; fall through.
  }
  return navigator.language?.toLowerCase().startsWith("ru") ? "ru" : "en";
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
  }, [lang]);

  const setLang = useCallback((next: Lang) => {
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // Preference just won't persist; switching still works this session.
    }
    setLangState(next);
  }, []);

  const t = useCallback<Translate>(
    (key, params) => {
      const dict = DICTS[lang] as Record<string, string | undefined>;
      // Unknown key → fall back to English, then to the key itself (which is
      // how raw engine error strings pass through untouched).
      const raw = dict[key] ?? (en as Record<string, string | undefined>)[key] ?? key;
      if (!params) return raw;
      return raw.replace(/\{(\w+)\}/g, (match, name: string) =>
        name in params ? String(params[name]) : match
      );
    },
    [lang]
  );

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
