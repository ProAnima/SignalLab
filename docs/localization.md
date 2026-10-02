# Localization: languages of the interface

Status: 2026-10-02. Languages today: English (the source) and Russian.

Everything a person reads in Signal Lab comes from one dictionary per language.
This page says where texts live, how they are written, what is checked, and how
to add a language.

## 1. Where texts live

| What | Where |
|---|---|
| Every text of the interface | `src/lib/locales/en.ts` — the source; it defines the `Dict` type |
| A translation | `src/lib/locales/<code>.ts`, typed `Dict`: a missing or unknown key is a compile error |
| The list of languages | `src/lib/locales/index.ts` — `LOCALES`: code, own name, two letters, dictionary |
| Filling in a text | `src/lib/translate.ts` — placeholders, plurals, numbers; pure and tested |
| The current language | `src/lib/i18n.tsx` — `useT()`, `useI18n()`, the saved choice and detection |
| Engine failures | the engine reports a code (`EngineError`), the dictionary has `err.<code>` |
| Starter signals | the engine writes them on first run; the interface renames them by id (`seed.<id>.name`, `.note`) |
| The server's sign-in page | `server/src/routes.rs` (`LoginText`), chosen by `Accept-Language` |

The language is the saved choice (`signal-lab.lang`), otherwise the first of the
system's languages (`navigator.languages`) that Signal Lab has, by base language
(`pt-BR` → `pt`), otherwise English. Switching re-renders everything at once:
console lines and failures are stored as a key with values, never as finished
text.

## 2. Writing a text

- **One key per thing a person reads**, named by screen: `osc.*`, `mq.*`,
  `ins.*`, `sig.*`, `exp.*`; shared ones under `common.*`, units under `unit.*`,
  engine failures under `err.*`, node fields under `field.*`.
- **Values are placeholders**, never glued on: `"sent {address} → {target}"`.
  `{name}` is the value exactly as given — ports and ids stay as they are; a
  grouped number is passed already formatted (`fmtNum(n)`).
- **Counts are plurals**, in the ICU form:
  `{n, plural, one {# signal} other {# signals}}`. `#` is the number written the
  language's way. Each language lists the forms its rules have — English `one`,
  `other`; Russian `one`, `few`, `many`, `other` (`other` covers fractions);
  `=0 {…}` gives an exact value its own text. Pass the number itself, not
  `fmtNum(n)`. A label of the form `"Topics: {n}"` needs no plural.
- **Help goes in a tooltip**: a `…Hint` key shown through `data-tip` on the
  control or its label (CLAUDE.md, "No explanatory captions on screen").
- **Numbers and sizes** go through `fmtNum` / `fmtBytes` (`src/lib/format.ts`),
  which follow the current language — grouping, decimal sign, `unit.*` names.
  Times in the console and the timeline are `HH:MM:SS`, the same everywhere.
- **Protocol words stay**: OSC, QoS, retain, URL, TTL, hex — and what the
  Inspector decodes from the wire.

## 3. What is checked

`npm run check` runs all of these; CI runs it on Windows and Linux.

| Check | Where |
|---|---|
| Every language has every key and no other | `tsc` (`Dict`) and `tests/i18n.test.mjs` |
| Every text asks for the same values as English, plurals included | `tests/i18n.test.mjs` |
| Every plural has `other` and every form the language's rules need, and no form they do not have | `tests/i18n.test.mjs` |
| Every key the code names by a literal exists | `tests/i18n.test.mjs` |
| Every engine code has an `err.<code>` text, and no text is left without a code | `engine/src/error.rs` (test) |
| The starter set in the engine and its texts agree | `tests/i18n.test.mjs` |
| The sign-in page follows `Accept-Language` | `server/tests/server.rs` |
| No English left on any screen in Russian, frame and bottom panel included | the e2e tour, step `russian` |
| Every icon button has a tooltip and every field a name | the e2e tour, per screen |

## 4. Adding a language

1. Copy `src/lib/locales/ru.ts` to `src/lib/locales/<code>.ts` (`de`, `pt`, …),
   rename the export, translate every text. Keep placeholders as they are; write
   the plural forms your language has (`Intl.PluralRules("<code>")
   .resolvedOptions().pluralCategories` lists them).
2. Add one line to `LOCALES` in `src/lib/locales/index.ts`: code, the
   language's own name, two letters, the dictionary. The switch in the header,
   detection, `<html lang>`, plural rules and number formats follow from it.
3. Add a `LoginText` for the server's sign-in page in `server/src/routes.rs` and
   list it in `LOGIN_TEXTS`.
4. `npm run check`. The tests name every text that is missing, asks for other
   values or lacks a plural form.
5. Run the tour in the new language once (`npm run e2e`, then look at the
   screenshots in `artifacts/e2e/`) for texts that do not fit. The `russian`
   step is the model for a step that fails on leftover English.

## 5. Not translated, on purpose

- **Documents a person made**: signal names, notes, folders, experiments. The
  starter set is translated once, when it is created, and is theirs afterwards.
- **What the system says**: the operating system's or a library's own text
  arrives as the `detail` of a failure and is shown folded, as it is.
- **The wire**: Inspector summaries and verdicts are protocol notation.
- **Logs on a server**: `tracing` output and job labels in them are English for
  whoever reads the server's logs.
