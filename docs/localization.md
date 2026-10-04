# Localization: languages of the interface

Status: 2026-10-04. Languages today: English (the source), Russian, Spanish,
French, German, Portuguese (Brazilian), Chinese (Simplified), Japanese, Korean,
Hindi and Arabic — Arabic right to left.

Everything a person reads in Signal Lab comes from one dictionary per language.
This page says where texts live, how they are written, what is checked, and how
to add a language.

## 1. Where texts live

| What | Where |
|---|---|
| Every text of the interface | `src/lib/locales/en.ts` — the source; it defines the `Dict` type |
| A translation | `src/lib/locales/<code>.ts`, typed `Dict`: a missing or unknown key is a compile error |
| The list of languages | `src/lib/locales/index.ts` — `LOCALES`: code, own name, two letters, direction, number tag, dictionary |
| The switch | `src/components/LanguageMenu.tsx` — the header's flag and letters, opening the list; flags in `src/lib/flags.ts` |
| Right to left | `<html dir>` (`lib/i18n.tsx`), logical CSS (`margin-inline-start`, …), `src/styles/rtl.css` for what stays left to right |
| Filling in a text | `src/lib/translate.ts` — placeholders, plurals, numbers; pure and tested |
| The current language | `src/lib/i18n.tsx` — `useT()`, `useI18n()`, the saved choice and detection |
| Engine failures | the engine reports a code (`EngineError`), the dictionary has `err.<code>` |
| Starter signals | the engine writes them on first run; the interface renames them by id (`seed.<id>.name`, `.note`) |
| The server's sign-in page | `server/src/routes.rs` (`LoginText`), chosen by `Accept-Language` |
| The command line | `cli/build.rs` embeds every dictionary; `cli/src/i18n.rs` has each language's plural rules and number style (`Lang`) |
| The installer | `src-tauri/tauri.conf.json`, `nsis.languages`: NSIS's and Tauri's translations, picked from the system or the selector; ours where they lack one (`customLanguageFiles`): Tauri's messages in Hindi (`installer/Hindi.nsh`), and in Korean the page that asks who to install for, which NSIS has only in English (`installer/Korean.nsh`, Tauri's Korean and those five) |

The language is the saved choice (`signal-lab.lang`), otherwise the first of the
system's languages (`navigator.languages`) that Signal Lab has, by base language
(`pt-BR` → `pt`), otherwise English. Switching re-renders everything at once:
console lines and failures are stored as a key with values, never as finished
text.

The switch in the header shows the current language's flag and letters; it opens
the list of every language by its flag and its own name (*Deutsch*, *日本語*) —
the name someone who reads that language looks for, whatever the page is in now,
so the list is the same in every language; a translated name and the letters
beside it would only repeat it. The arrows, Home and End move in it, a letter
jumps to the next language that starts with it (its own name, its letters, or its
name in the current language or in English, from `Intl.DisplayNames`: "g" finds
Deutsch), Enter chooses, Escape closes. Flags are pictures beside a name (`alt=""`): English is
shown with the United Kingdom's flag, Portuguese with Brazil's, Arabic with Saudi
Arabia's; Spain's is its civil flag (the one with the arms weighs 80 KB). The
flags are from flag-icons (MIT).

**Right to left.** Arabic sets `<html dir="rtl">`: the page mirrors — sidebar on
the right, the panes and rows in reading order — because the stylesheet uses
inline-start/end, never a fixed side, where a side follows the reading. What is
data stays left to right (`styles/rtl.css`): the experiment canvas (its wires run
from outputs on the right to inputs on the left), hex dumps and code; a field
shows what is typed in it the way it reads (`unicode-bidi: plaintext` — an Arabic
name right to left, an address left to right). A vertical pane handle sizes the
pane on its left there, so the drag and the arrow keys turn round
(`Splitter`). Numbers in Arabic are written in Latin digits (`intl:
"ar-u-nu-latn"`), as the ports, addresses and values beside them are.

**Letter spacing.** The small caps labels are spaced out in Latin and Cyrillic.
Arabic letters join and Devanagari's hang from one headstroke, so spacing tears
them apart; Chinese, Japanese and Korean have no capitals to set off. In those
five no element gets any (`styles/rtl.css`, by `:lang()`).

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
| …nor in any other language; the menu lists each with its flag; Arabic mirrors the page and keeps the canvas left to right; nothing wider than the window | the e2e tour, step `languages` |
| Every tooltip is in each language (its script, for one that has its own) | `tests/tooltip.test.mjs` |
| The command line's plural rules and numbers equal `Intl`'s, and it has every language of `LOCALES` | `cli/src/i18n.rs` (tests) |
| Every language has a sign-in page; Arabic's is right to left | `server/tests/server.rs` |
| The installer has every language | `tests/installer.test.mjs` |
| Every icon button has a tooltip and every field a name | the e2e tour, per screen |

## 4. Adding a language

1. Copy `src/lib/locales/ru.ts` to `src/lib/locales/<code>.ts` (`it`, `tr`, …),
   rename the export, translate every text. Keep placeholders as they are; write
   the plural forms your language has (`Intl.PluralRules("<code>")
   .resolvedOptions().pluralCategories` lists them).
2. Add one line to `LOCALES` in `src/lib/locales/index.ts`: code, the
   language's own name, two letters, the dictionary, and `dir: "rtl"` for a
   language read right to left. Detection, `<html lang>`/`dir`, plural rules
   and number formats follow from it.
3. A flag for it in `src/lib/flags.ts`.
4. The command line: the code in `LANGUAGES` (`cli/build.rs`), a `Lang`
   variant, its plural rule and number style in `cli/src/i18n.rs` — the test
   there compares them with `Intl`'s; generate the expected values with Node.
5. A `LoginText` for the server's sign-in page in `server/src/routes.rs`, listed
   in `LOGIN_TEXTS`.
6. NSIS's name for it in `nsis.languages` (`src-tauri/tauri.conf.json`) and in
   `tests/installer.test.mjs`. Then `npm run tauri build -- --bundles nsis`: if
   Tauri has no messages in that language, write them as `Hindi.nsh` does; if
   NSIS warns that a `LangString` *is missing* for it, add those after Tauri's,
   as `Korean.nsh` does. Name the file in `customLanguageFiles`, plain UTF-8 —
   Tauri adds the BOM.
7. `npm run check`. The tests name every text that is missing, asks for other
   values or lacks a plural form.
8. `npm run e2e`: the `languages` step visits every screen in it; look at the
   screenshots in `artifacts/e2e/` for texts that do not fit.

## 5. Not translated, on purpose

- **Documents a person made**: signal names, notes, folders, experiments. The
  starter set is translated once, when it is created, and is theirs afterwards.
- **What the system says**: the operating system's or a library's own text
  arrives as the `detail` of a failure and is shown folded, as it is.
- **The wire**: Inspector summaries and verdicts are protocol notation.
- **Logs on a server**: `tracing` output and job labels in them are English for
  whoever reads the server's logs.
- **The license**: `LICENSE` is MIT, in English — the text that is legally the
  license; the About dialog names it. MIT asks for no acceptance, so neither
  the installer nor the app shows an agreement to accept.
