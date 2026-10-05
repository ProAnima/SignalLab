---
title: Writing the documentation
---

# Writing the documentation

The documentation is a VitePress site in `docs/`: the user guide, the protocols
and tools, experiments, automation, the server and its HTTP API, in every
language of the interface, and these pages for contributors, in English. It is
built into the app and the server (`/docs/`, offline) and published on GitHub
Pages.

```bash
npm run docs:dev     # live reload at http://localhost:5173/docs/
npm run docs:build   # dist/docs, as the app and the server serve it
npm test             # tests/docs.test.mjs among them
```

## Where things are {#layout}

| Path | What |
| --- | --- |
| `docs/<section>/<page>.md` | English, the source of every other language |
| `docs/<code>/<section>/<page>.md` | the same page in `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi`, `ar` |
| `docs/develop/` | for contributors, English only |
| `docs/.vitepress/structure.ts` | the sections and their pages, in order; which page F1 opens on each screen |
| `docs/.vitepress/i18n.ts` | what the site itself says (section names, buttons) in every language |
| `docs/.vitepress/config.mts` | the site: languages, sidebar, search, the plugins below |
| `scripts/docs.mjs` | generated pages, the build, moving inline scripts out (the server's CSP) |

A page's title in the sidebar is its own `title:`. `reference/errors.md` is
written from the dictionaries by `scripts/docs.mjs generate` — never by hand,
and not in git.

## Who reads it {#audience}

People bringing up show control, installations and networked devices, testers
of APIs and services, and whoever automates either. Write for someone who has
the app open and wants to get something done:

- **Lead with the task**, then the details: "To watch what a device sends…",
  not "The monitor is a component that…".
- **Second person, present tense, short sentences.** No marketing, no
  "simply", no "just".
- **Every number is the code's**: defaults, limits, units, ranges. Look them up
  in the engine (`engine/src`), never guess. If a limit exists, say what happens
  at it.
- **Only what exists.** Nothing planned, nothing removed. When unsure, read the
  code or leave it out.
- **No source paths in user pages** (`engine/src/…` belongs in `develop/`).
  Users see screens, fields, commands, files and URLs.

## Names from the interface {#ui-labels}

Never type what a button, field, screen or tab is called. Write its key from
`src/lib/locales/en.ts` instead:

```md
Press [[ui:exp.addNode]], then pick [[ui:exp.node.http]].
```

The build replaces `[[ui:key]]` with that text in the page's language, so a
translated page names everything exactly as the app shows it in that language,
and a label that is renamed or removed breaks the build instead of leaving the
page wrong. Only texts with nothing to fill in are labels (no `{n}`); find a key
by searching `en.ts` for the English text. Labels never go in headings.

## Structure {#structure}

- Front matter: `title:` (short, as the sidebar shows it) and `description:`
  (one sentence, for search engines and link previews).
- One `#` heading, the page's subject. Sections are `##`, steps or items `###`.
- **Every `##` and `###` heading has an explicit id**, in English, kebab-case:
  `## Sending a message {#send}`. Translations keep the same ids, so a link to
  `protocols/osc.md#send` and the app's help work in every language. A test
  checks every language has the same ids.
- Reference pages use fixed ids the tests know: `{#node-<type>}` for each kind
  of node, `{#<command>}` for each API command, `{#cli-<subcommand>}` for each
  command of `signallab`, `{#event-<channel>}` for each event.
- Procedures are numbered lists. Options are tables (name, what, default).
- Link to other pages relatively, with the `.md`: `[emulators](../tools/emulators.md#rules)`.

## Formatting {#formatting}

- Code, commands, addresses, file names, JSON keys, template expressions:
  `` `inline code` `` or fenced blocks with a language (`bash`, `json`,
  `powershell`, `yaml`). `{{templates}}` are safe anywhere.
- Keys: `<kbd>Ctrl</kbd>+<kbd>K</kbd>`.
- Callouts: `::: tip`, `::: info`, `::: warning`, `::: danger` (only for what
  can harm equipment, data or networks), `::: details` for long examples.
- Examples use loopback (`127.0.0.1`) and documentation addresses
  (`192.0.2.0/24`, `example.com`), never someone else's host.

## Translating {#translating}

English is the source. A translation keeps, unchanged: every `[[ui:…]]`, every
heading id, code, commands, file names, URLs, link targets, JSON, front-matter
keys. It translates the title, description, headings' text and prose. Terms
follow the interface's dictionary for the same language (the labels already
do). A page that changes in English changes in every language in the same
commit; the tests fail on a language that lacks a page or an id.
