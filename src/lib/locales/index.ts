import { en, type Dict } from "./en.ts";
import { ru } from "./ru.ts";
import { es } from "./es.ts";
import { fr } from "./fr.ts";
import { de } from "./de.ts";
import { pt } from "./pt.ts";
import { zh } from "./zh.ts";
import { ja } from "./ja.ts";
import { ko } from "./ko.ts";
import { hi } from "./hi.ts";
import { ar } from "./ar.ts";

/**
 * The languages of the interface. Adding one is a dictionary typed `Dict`
 * (so a missing key is a compile error) and one line here — see
 * docs/localization.md. Plural rules and number formats come from `Intl` for
 * the code, so nothing else needs to know the language.
 */
export interface Locale {
  /** BCP 47 code: picks the dictionary, `<html lang>`, plural rules and number formats. */
  code: string;
  /** The language's own name for itself, as the switch shows it. */
  name: string;
  /** Two letters on the switch. */
  short: string;
  /** `rtl`: the page reads right to left (Arabic); what is data — addresses, hex, the canvas — stays left to right. */
  dir?: "rtl";
  /** The tag numbers and plural rules use, where it is not `code` (Arabic: Latin digits, as ports and addresses have). */
  intl?: string;
  dict: Dict;
}

export const LOCALES = [
  { code: "en", name: "English", short: "EN", dict: en },
  { code: "ru", name: "Русский", short: "RU", dict: ru },
  { code: "es", name: "Español", short: "ES", dict: es },
  { code: "fr", name: "Français", short: "FR", dict: fr },
  { code: "de", name: "Deutsch", short: "DE", dict: de },
  { code: "pt", name: "Português", short: "PT", dict: pt },
  { code: "zh", name: "中文", short: "ZH", dict: zh },
  { code: "ja", name: "日本語", short: "JA", dict: ja },
  { code: "ko", name: "한국어", short: "KO", dict: ko },
  { code: "hi", name: "हिन्दी", short: "HI", dict: hi },
  { code: "ar", name: "العربية", short: "AR", dir: "rtl", intl: "ar-u-nu-latn", dict: ar },
] as const satisfies readonly Locale[];

export type Lang = (typeof LOCALES)[number]["code"];

/** The language every other one is written from; a text missing elsewhere falls back to it. */
export const SOURCE: Lang = "en";

export const CODES: readonly Lang[] = LOCALES.map((locale) => locale.code);

export function dictOf(lang: Lang): Dict {
  return LOCALES.find((locale) => locale.code === lang)?.dict ?? en;
}

function localeOf(lang: Lang): Locale {
  return LOCALES.find((locale) => locale.code === lang) ?? LOCALES[0];
}

/** The tag `Intl` formats this language's numbers and picks its plural forms with. */
export function intlOf(lang: Lang): string {
  return localeOf(lang).intl ?? lang;
}

/** Which way the language reads. */
export function dirOf(lang: Lang): "ltr" | "rtl" {
  return localeOf(lang).dir ?? "ltr";
}
