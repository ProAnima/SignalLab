---
title: "Design: data in experiments"
---

# Milestone 3 — data in experiments: detailed design

::: info A design note
Written while the feature was being designed and kept as the record of why it is
the way it is. What shipped is described in the user documentation, which is
the one to trust where the two differ.
:::

Status: milestone 3 implemented — PR 3.1, 3.2 (section 10) and 3.3 (section 11), 2026-10-01. Parent plan: [ROADMAP.md](./roadmap.md#3-data-parameters-templates-extraction-generic-checks).

Values must move through a run: a parameter chooses the target, a response field becomes the next request's header, a generated id travels in a command and comes back in a check. This document fixes the language, the file format, the nodes, the validation rules and the interface so they can be built in small pieces without changing their meaning later.

## Delivery in three pull requests

| PR | Contents |
| --- | --- |
| **3.1 (this slice)** | Template language and resolver; generators with a run seed; document v2 with parameters and seed; templates in every text field of action and check nodes; **Extract**, **Check value** and **Branch on value** nodes; static checks; resolved preview; `{{` autocomplete; *Send now* with templates; **Extract as variable** from a response; seed in the report with one-click pinning. |
| 3.2 | Target profiles (named parameter sets switched from the toolbar), **Run with…** overrides. |
| 3.3 | Secrets `{{secret.NAME}}` in the Windows Credential Manager, masked in the timeline and reports. |

## 1. Template language

### Syntax

```
template := ( text | escape | expression )*
escape   := "\{{"                          → a literal "{{"
expression := "{{" space* reference space* "}}"
reference  := call | path
call     := ident "(" [ argument ( "," argument )* ] ")"
argument := number | 'text' | "text" | word          word = [A-Za-z0-9_.:/+-]+
path     := ident ( "." ident | "[" digits "]" | "[" quoted "]" )*
ident    := [A-Za-z_][A-Za-z0-9_]*
```

- Whitespace inside `{{ }}` and around call arguments is ignored.
- A `{{` without its `}}`, an empty `{{}}`, an unknown function or a malformed argument is a **syntax error** reported with the field and the character position.
- Text outside expressions is copied exactly, including `}}` on its own.

### Names

| Reference | Meaning |
| --- | --- |
| `name` | Variable `name` if one is set on this path, otherwise parameter `name` |
| `vars.name` | Variable only |
| `params.name` | Parameter only |
| `name.field`, `name[0]`, `name["a b"]` | A field of a variable or parameter holding JSON |
| `run.id`, `run.seed` | The job id and seed of the current run |
| `node.id` | The id of the node being executed |
| `secret.NAME` | Reserved for PR 3.3; an error until then |

Reserved heads — `vars`, `params`, `secret`, `run`, `node`, `now`, `uuid`, `counter` and the function names — cannot be used as parameter or variable names. A variable may not have the same name as a parameter, so a bare name is never ambiguous.

An unknown name is an error that names it. It never expands to an empty string.

### Generators

| Expression | Value |
| --- | --- |
| `{{uuid}}` / `{{uuid()}}` | Random UUID v4 |
| `{{now}}` | Unix time in milliseconds |
| `{{now.iso}}` | UTC time, `2026-09-30T12:34:56.789Z` |
| `{{counter}}` | How many times this node has executed in this run, starting at 1 |
| `{{random_int(a, b)}}` | Integer in `a..=b`; `a ≤ b` |
| `{{random_float(a, b)}}`, `{{random_float(a, b, digits)}}` | Number in `[a, b)` with `digits` decimals (default 3, at most 9) |
| `{{pick(a, b, …)}}` | One of the arguments |

**Determinism.** Every run has a seed: the document's `seed` if set, otherwise a fresh random one, recorded in the report. The random stream of a node's execution is derived from `(seed, node id, execution count)` with SplitMix64, so parallel branches cannot change each other's values and a rerun with the same seed repeats them exactly. Draws within one node follow field order. `now` is the only generator that is not reproducible.

### Rendering values

Strings are inserted as they are; numbers in their shortest form (`42`, `0.5`); booleans as `true`/`false`; `null` as `null`; arrays and objects as compact JSON. Values are inserted raw: a template inside a JSON body is responsible for its own quotes (`"id": "{{uuid}}"`).

### Where templates are allowed

| Node | Templated fields |
| --- | --- |
| HTTP request | URL, header names and values, body |
| OSC message | Target, address, string arguments |
| UDP datagram | Target, payload |
| TCP message | Host, payload |
| MQTT publish | Broker host, topic, payload |
| Log | Message |
| Response text / header checks | Expected text, header name |
| Check value, Branch on value | Value, expected value |

Numeric fields (ports, timeouts, delays, statuses, typed OSC numbers) stay literal in 3.1; they need a typed "number or template" field and follow later.

## 2. Document version 2

```json
{
  "version": 2,
  "name": "Login and read state",
  "params": [
    { "name": "api", "value": "http://127.0.0.1:8080" },
    { "name": "user", "value": "lab" }
  ],
  "seed": null,
  "nodes": [ … ],
  "edges": [ … ]
}
```

- `params`: ordered list, at most 64. Names follow `ident`, are unique and not reserved. Values are literal text (no templates), at most 64 KiB each.
- `seed`: `null` (a new seed per run) or an integer `0 ≤ seed ≤ 2^53 − 1`, so it survives JavaScript without rounding.
- **Migration.** Version 1 documents are read as version 2 with `params: []` and `seed: null`. Saving always writes version 2. Bundled templates are stored as version 2. A version the app does not know is refused with its number.

## 3. Variables

- Written by **Extract** (3.1) and later by waits and devices. A variable holds a JSON value.
- Each branch has its own set. **Parallel branch** gives both branches a copy; **Join** merges them in the order of its incoming connections (the later wins on a name clash), so the result does not depend on which branch finished first.
- The timeline event of a writing step lists what it wrote; the report keeps the events, the seed and the parameter values.

## 4. New nodes

### Extract (group *Data*)

| Field | Meaning |
| --- | --- |
| `variable` | Name to write |
| `from` | `json`, `header`, `status`, `body`, `regex` |
| `expr` | JSON path (`$.token`, `$.items[0].id`, `$["a b"]`; a leading `$.` may be omitted), header name, or regular expression |

Reads the latest HTTP response on the executed path (same rule as the HTTP checks). Missing data fails the step and says what was missing. A truncated body cannot be parsed as JSON and fails explicitly; a regex that does not match a truncated body is inconclusive and fails explicitly. For a regex, capture group 1 is used when present, otherwise the whole match.

### Check value (group *Checks*) and Branch on value (group *Flow*)

Fields `value`, `op`, `expected`; both text fields are templates. Branch on value has **Yes** and **No** outputs.

| `op` | Passes when |
| --- | --- |
| `eq`, `ne` | Equal / not equal — numerically when both sides are numbers, otherwise as exact text |
| `lt`, `le`, `gt`, `ge` | Numeric comparison; a side that is not a number is an error, not a quiet "false" |
| `contains` | `value` contains `expected` |
| `matches` | `value` matches the regular expression `expected` |
| `empty`, `not_empty` | `value` is empty / not empty after trimming (`expected` is ignored) |

A failed check reports both sides, shortened to 120 characters each.

## 5. Static checks (before a run)

Run on every edit (as today) and again by `experiment_start`:

1. Every templated field parses; errors name the node, the field and the position.
2. Every function call has valid arguments (`random_int(5, 1)` is rejected).
3. Every regular expression that does not depend on templates compiles.
4. Every name is known: a parameter, a generator, or a variable **set on every path** before the node. Paths through a **Join** add up (all branches have run); paths into any other node must all set it.
5. Parameters and extracted variables do not share names and do not use reserved names.
6. Extract, like the HTTP checks, needs an earlier HTTP request on every path.
7. A field whose templates use only parameters is resolved and checked like a literal (URL scheme, OSC `IP:port` or `host:port`); fields with variables or generators are checked after resolution, at run time.

## 6. Engine

- `engine/template.rs` — parser, static name analysis, resolver, generators and SplitMix64; pure and unit-tested without a Tauri runtime.
- `engine/experiment.rs` — `NodeKind::{Extract, AssertValue, BranchValue}`; `BranchContext.vars`; `render_node` produces a node with every templated field resolved; deterministic Join merge; `validate` gains the checks above; the run reports its seed and parameters.
- `engine/experiment_files.rs` — version 1 → 2 migration.
- Commands (the four-edit rule from CLAUDE.md applies):
  - `experiment_resolve(document, node_id, vars)` → the node with templates resolved plus the names that had no value. Used by the preview and by *Send now*, so the UI never implements the language itself.
  - `experiment_extract_after(document, node_id, response)` → the variables that the Extract nodes directly following an HTTP node would set from this response. *Send now* on a request fills in values for the next *Send now* without a full run.
- Run events carry `vars` (what the step wrote); `experiment://ended` carries `seed`.

## 7. Interface

- **Parameters** (`{ }` in the toolbar): a panel with name/value rows, add and remove, and the seed (empty = new every run). Edits are undoable like any field.
- **Template fields** that contain `{{…}}` carry an accent marker. Typing `{{` (or `Ctrl+Space`) opens suggestions: parameters (with value), variables set upstream (with the node that sets them) and generators (with a description). ↑/↓ choose, Enter or Tab insert, Esc closes the list without leaving the field.
- **Resolved preview** under the form of a templated node: what will be sent with current parameters and known variable values; names without a value are listed in amber.
- **Known values** come from the last run and from *Send now*: after a request is sent, the Extract nodes that follow it are evaluated against its response.
- **Extract as variable**: in a *Send now* JSON response every value is clickable; a click inserts an Extract node after the request with the JSON path filled in and a name suggested from the key, and the value becomes known at once.
- **Seed**: the timeline header shows the seed of the last run with **Pin** to store it in the document.

## 8. Tests

- Rust: parsing and syntax errors; rendering of every value kind; names and precedence; every generator, their bounds and determinism by seed/node/execution; JSON path subset; comparison operators; extraction from JSON, headers, status, body, regex and truncated bodies; validation of names per path, Join union, reserved names, shadowing, parameter-only resolution of URLs and OSC targets; v1 → v2 migration; bundled templates round-trip.
- Editor (`npm test`): catalogue coverage for the new nodes in both locales; upstream variable suggestions; JSON path building for *Extract as variable*.

## 9. Not in 3.1

Profiles, Run with…, secrets (3.2, 3.3); templates in numeric fields; filters such as `| urlencode` or `| json`; waits and their reply variables (milestone 4).

---

## 10. PR 3.2 — target profiles and Run with…

The same experiment runs against a laptop, a stage rack and a venue network. Parameters already carry the targets; a **profile** is a named set of parameter values, so switching the target means choosing a name, not editing nodes. **Run with…** changes values for one run without touching the document.

### Document version 3

```json
{
  "version": 3,
  "params": [{ "name": "api", "value": "http://127.0.0.1:8080" }, { "name": "device", "value": "127.0.0.1:9000" }],
  "profiles": [
    { "name": "Stage", "values": { "api": "http://10.0.0.20:8080" } },
    { "name": "Venue", "values": { "api": "http://10.1.0.20:8080", "device": "10.1.0.31:9000" } }
  ],
  "profile": "Stage",
  "seed": null,
  "nodes": [ … ], "edges": [ … ]
}
```

- `params` keep the **default** values; a profile overrides some of them and inherits the rest.
- `profile` is the active profile (`null` — defaults only). It is part of the document, so an exported experiment opens with the same targets, and switching is one undoable edit.
- At most 32 profiles; names are non-empty, at most 64 characters, unique.
- **Migration.** Versions 1 and 2 are read as version 3 with no profiles. A build that does not know version 3 refuses the file instead of silently dropping its profiles on save.

### Effective values

`default ← active profile ← run overrides`, left to right, later wins. Every consumer uses the same function (`experiment_data::effective_params`): the runner, validation, the resolved preview and *Send now*. The report records the profile name, the overrides and the effective values.

### Drafts versus runs

A document being edited must always save, even half-typed. Structural checks (saving) limit only sizes: parameter and profile counts, value sizes, the seed range. Names, duplicates, profile keys that no longer exist and a missing active profile are **run-time** checks, reported by validation like any other problem. The editor renames or removes a parameter in every profile at once, so the usual edits never leave such keys behind.

### Every profile is checked

Validation gates a run with the effective values of the active profile. In the same pass it validates every other profile (and the defaults) and returns their problems as **profile issues** that do not block the run: the switcher marks a profile with ⚠ and says what would fail before anyone switches to it.

### Run with…

A menu on the Run button opens a small form:

- the profile for this run (the active one preselected),
- each parameter with its effective value as placeholder; a typed value overrides it for this run only,
- the seed: empty uses the pinned seed or a new one; **Last seed** fills in the seed of the previous run.

Enter runs. The form remembers its overrides for the session, so repeated runs with the same change are one click. Overrides naming unknown parameters are refused. The timeline shows the profile and whether values were overridden.

### Engine and API

- `effective_params(doc, profile, overrides)`, `check_profiles(doc)` (run-time rules), `profile_issues(doc)`.
- `validate(doc)` uses the active profile; `validate_with(doc, params)` takes an explicit set.
- `experiment_validate` returns the profile issues on success.
- `experiment_start(document, overrides, seed)`; `experiment://ended` carries `profile` and `overridden`.

### Tests

Precedence of defaults, profile and overrides; migration from versions 1 and 2; drafts with duplicate or invalid names still save but do not run; unknown profile keys, a missing active profile and unknown overrides are rejected; a broken non-active profile is reported as an issue and does not block the active one; editor helpers rename and remove a parameter across profiles.

---

## 11. PR 3.3 — secrets

A token or password must work in a field (`Authorization: Bearer {{secret.API_TOKEN}}`) without ever being written into an experiment file, a report, a screenshot of the timeline or the editor's memory.

### Storage

- Values live in the operating system's credential store — the Windows Credential Manager (macOS: Keychain) — under the service `SignalLab`, one entry per name. The experiment stores **only names**, so a shared file says which secrets it needs, never their values.
- Names follow the parameter rules (`[A-Za-z_][A-Za-z0-9_]*`). Secrets belong to the machine, not to one experiment: two experiments using `{{secret.API_TOKEN}}` use the same value.
- On a platform without a supported store the engine says so; it never falls back to keeping values in memory or on disk.

### The value never reaches the interface

- Commands: `secret_status(names)` → which names are stored; `secret_set(name, value)`; `secret_delete(name)`. There is no command that returns a value.
- *Send now* on an experiment node is executed by the engine (`experiment_send_node`) with the same action code a run uses; it returns the result, not the resolved request. (In 3.1 the editor resolved the node and sent it through the direct commands; with secrets that would put values in the webview.) Extraction from the response happens there too.
- The resolved preview shows `••••` for a stored secret and lists a missing one.

### Masking

While a run or a *Send now* uses secrets, every occurrence of their values is replaced by `••••` in what leaves the engine: step details and errors, extracted variables, *Send now* results (including the HTTP response it returns), run reports, and Inspector frames (summary, decode and the bytes behind the hex dump). The traffic itself carries the real value.

### Validation

A run (and **Run with…**) is refused before any traffic if a referenced secret is not stored on this machine: `login: header 1 value: secret API_TOKEN is not stored on this machine — set it under Parameters → Secrets`.

### Interface

- The Parameters panel has a **Secrets** section: every secret the experiment references, with its state (*stored* / *not set*), **Set…** / **Replace…** (a password field that is cleared after saving) and **Remove**; **＋ Secret** stores a value under a new name before it is used.
- `{{` suggestions include `secret.NAME` for names the experiment references or that were stored in this session.

### Engine

- `engine/secrets.rs`: a `SecretStore` trait with the credential-store implementation and an in-memory one for tests; masking helpers; the redaction registry the Inspector consults.
- `template::Scope` gains the secret values of the run; `secret.NAME` resolves from them.
- The runner loads the referenced secrets once at start; a missing one is a validation error.

### Tests

Resolution of `secret.NAME` and the error for a missing one; masking of details, errors, variables, responses and reports; a run is refused when a referenced secret is not stored (in-memory store); the Inspector redaction replaces values in text and in payload bytes; names follow the rules.
