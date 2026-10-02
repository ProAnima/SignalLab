import { en, type Dict } from "./en.ts";
import { ru } from "./ru.ts";

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
  dict: Dict;
}

export const LOCALES = [
  { code: "en", name: "English", short: "EN", dict: en },
  { code: "ru", name: "Русский", short: "RU", dict: ru },
] as const satisfies readonly Locale[];

export type Lang = (typeof LOCALES)[number]["code"];

/** The language every other one is written from; a text missing elsewhere falls back to it. */
export const SOURCE: Lang = "en";

export const CODES: readonly Lang[] = LOCALES.map((locale) => locale.code);

export function dictOf(lang: Lang): Dict {
  return LOCALES.find((locale) => locale.code === lang)?.dict ?? en;
}
