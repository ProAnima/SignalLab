---
title: Data and templates
description: Parameters and profiles, the template language with its generators, values extracted from responses, comparisons, and secrets that never leave the engine.
---

# Data in experiments

Values move through a run: a parameter chooses the target, a field of one
response becomes a header of the next request, a generated id goes out in a
command and comes back in a check. This page covers where those values come
from and how a field uses them.

| Source | Written as | Set where |
| --- | --- | --- |
| Parameter | `{{api}}` or `{{params.api}}` | [[ui:exp.params]] panel, a profile, [[ui:exp.runWith]] |
| Variable | `{{token}}` or `{{vars.token}}` | a node during the run: [[ui:exp.node.extract]], a wait, a send that waits for a reply |
| Secret | `{{secret.API_TOKEN}}` | the computer's credential store, or the server's environment and files |
| Built-in value | `{{run.seed}}`, `{{now.iso}}`, `{{counter}}` | the run itself |
| Generator | `{{uuid}}`, `{{random_int(1, 100)}}` | drawn from the run's seed |

## Parameters {#parameters}

A parameter is a named text value that any templated field can use. Keep
targets in parameters, so a change of address is one edit instead of one per
node.

### Adding a parameter {#add-parameter}

1. Press [[ui:exp.params]] (`{ }`) in the editor's toolbar.
2. On the [[ui:exp.noProfile]] tab, press [[ui:exp.addParam]].
3. Type the [[ui:exp.paramName]] and the [[ui:exp.paramValue]], for example
   `api` and `http://127.0.0.1:8080`.
4. In a node's field, write `{{api}}/login`.

Every change in the panel is an edit of the experiment: it is saved with it
and undone with <kbd>Ctrl</kbd>+<kbd>Z</kbd> like any other.

### Rules {#parameter-rules}

| Rule | Limit |
| --- | --- |
| Name | starts with a letter or `_`, then letters, digits and `_` |
| Reserved names | `vars`, `params`, `secret`, `run`, `node`, `now`, `uuid`, `counter`, `random_int`, `random_float`, `pick` |
| Parameters per experiment | 64 |
| Size of one value | 64 KiB |
| Names | unique; a variable may not have a parameter's name |

A value is plain text and is inserted as written: `{{…}}` inside a value is
not resolved. When a field asks for a part of a parameter
(`{{config.ports[0]}}`), the value is read as JSON; a value that is not JSON
has no parts.

A parameter whose name is invalid, reserved or taken twice does not keep the
experiment from saving, so you can keep typing; the experiment does not run
until the name is fixed.

## Profiles {#profiles}

A profile is a named set of parameter values — *Laptop*, *Stage*, *Venue* — so
switching the target is a choice, not an edit of every node. A profile
changes some parameters; the others keep their default.

### Making a profile {#make-profile}

1. Open [[ui:exp.params]] and press [[ui:exp.addProfile]]. A new tab opens.
2. Rename it in [[ui:exp.profileName]].
3. For each parameter the profile changes, type its value. An empty field keeps
   the default, shown greyed in the field; [[ui:exp.resetToDefault]] (↺) clears
   a value.
4. Press [[ui:exp.makeActive]] to run with it. The active profile's tab carries
   ● [[ui:exp.activeProfile]]. [[ui:exp.makeActive]] on the
   [[ui:exp.noProfile]] tab goes back to the defaults.

Once an experiment has profiles, a [[ui:exp.profile]] list in the toolbar
switches between them. The active profile is used by runs, the preview and
[[ui:exp.sendNow]], and it is saved in the experiment, so an exported file opens
with the same targets. [[ui:exp.removeProfile]] deletes the profile on screen.

| Rule | Limit |
| --- | --- |
| Profiles per experiment | 32 |
| Name | 1–64 characters, unique (spaces at the ends do not count) |
| Values | only parameters that exist; at most 64 |

Renaming or removing a parameter changes it in every profile at once.

### Which value a run uses {#precedence}

Later wins:

1. the parameter's default, on the [[ui:exp.noProfile]] tab;
2. the active profile's value, if it sets one;
3. a value typed in [[ui:exp.runWith]] for this run only — see
   [running with other values](runs.md#run-with).

[[ui:exp.runWith]] may only set parameters the experiment has. The run report
records the profile, the values typed for the run and every value it used.

### Profiles that would not run {#profile-issues}

Each time the experiment is checked, the other profiles and the defaults are
checked too. One that would fail — say, a URL that is not `http://` or
`https://` — carries ⚠ in the tabs and in the toolbar's list, and its tooltip
says why. It does not stop runs with the profile in use.

## Templates {#templates}

Text inside `{{ }}` is an expression; everything else in a field is kept
exactly as written.

```text
{{api}}/users/{{user.id}}?trace={{uuid}}
Bearer {{secret.API_TOKEN}}
```

- Spaces inside the braces do not matter: `{{ token }}` is `{{token}}`.
- `\{{` writes a literal `{{`.
- A `}}` on its own is plain text.
- A value is inserted as it is, without quotes. In a JSON body, write the
  quotes yourself: `"id": "{{uuid}}"`.

### Names {#names}

| Expression | Value |
| --- | --- |
| `{{name}}` | the variable `name` if one is set on this path, otherwise the parameter `name` |
| `{{vars.name}}` | the variable only |
| `{{params.name}}` | the parameter only |
| `{{secret.NAME}}` | the stored secret `NAME` — see [secrets](#secrets) |
| `{{name.field}}` | a field of a JSON value |
| `{{name[0]}}` | an element of a JSON array |
| `{{name["a b"]}}`, `{{name['a b']}}` | a field whose name has other characters |

A field name after `.` may hold letters, digits, `_` and `-`. Steps chain:
`{{reply.args[0]}}`, `{{order.items[2].sku}}`.

### How values are written {#value-text}

| Value | Written as |
| --- | --- |
| text | the text |
| number | its shortest form: `42`, `0.5` |
| `true`, `false` | `true`, `false` |
| `null` | `null` |
| object, array | compact JSON: `["x","y"]` |

### Built-in values {#built-ins}

| Expression | Value |
| --- | --- |
| `{{run.id}}` | the run's job number; `0` in the preview and in [[ui:exp.sendNow]] |
| `{{run.seed}}` | the seed of this run |
| `{{node.id}}` | the id of the node being executed |
| `{{now}}` | the current time, Unix milliseconds |
| `{{now.iso}}` | the current time in UTC, ISO 8601 with milliseconds: `2026-09-30T12:34:56.789Z` |
| `{{counter}}` | how many times this node has run in this run, this time included, from 1 |

`{{counter}}` counts per node: in a [Loop](flow.md#loop) body it is the
iteration's number, in a node that [repeats](flow.md#repeat) the send's number.
`run`, `node` and `now` have only the fields listed; anything else is an error.

### Generators {#generators}

| Expression | Value |
| --- | --- |
| `{{uuid}}` or `{{uuid()}}` | a version 4 UUID |
| `{{random_int(min, max)}}` | a whole number from `min` to `max`, both included; whole-number arguments, `min` ≤ `max` |
| `{{random_float(min, max)}}` | a number from `min` up to, but not including, `max`, with 3 decimals; `min` < `max` |
| `{{random_float(min, max, digits)}}` | the same with `digits` decimals, 0–9 |
| `{{pick(a, b, c)}}` | one of the arguments, at least one |

Arguments are separated by commas. One in quotes (`"dark blue"` or `'a, b'`)
may hold anything but its own quote; one without quotes may hold letters,
digits and `_ - . : / +`. An empty argument is an error.

Every generator draws from the run's seed. The values of one execution of a
node depend only on the seed, the node's id and how many times the node has
run, so parallel branches never change each other's values, and a run with
the same seed generates the same values again. Draws within one node follow
the order of its fields. `{{now}}` and `{{run.id}}` are not reproducible. See
[seeds](runs.md#seeds).

### Suggestions {#suggestions}

Typing `{{` in a templated field, or pressing <kbd>Ctrl</kbd>+<kbd>Space</kbd>,
opens a list in four groups: [[ui:exp.suggest.params]] with their values,
[[ui:exp.suggest.vars]] set upstream of this node with the node that sets them
(a reply's fields too, such as `reply.args[0]`), [[ui:exp.suggest.secrets]] and
[[ui:exp.suggest.generators]]. <kbd>↑</kbd> and <kbd>↓</kbd> choose,
<kbd>Enter</kbd> or <kbd>Tab</kbd> inserts, <kbd>Esc</kbd> closes the list and
keeps the field.

### Unknown names are errors {#unknown-names}

A name without a value never becomes an empty string. Before a run, every
name a field uses must be a parameter, a valid secret name, or a variable set
on **every** path that leads to the node. The editor points at the node and
the field:

| Problem | Before the run | During the run |
| --- | --- | --- |
| A name nobody sets | `name.unknown` | — |
| A variable set on some paths only | `name.not_on_every_path` | — |
| `{{params.x}}` without a parameter `x` | `param.unknown` | — |
| A field that a value does not have | — | `template.no_field` |
| Unclosed `{{`, an empty `{{}}`, a malformed argument | `template.*`, with the position | — |

The texts of these codes are in [Errors](../reference/errors.md).

## Which fields take templates {#templated-fields}

| Node | Templated fields |
| --- | --- |
| [[ui:exp.node.http]] | URL, header names and values, body, the user name and password of Basic and Digest, the Bearer token |
| [[ui:exp.node.osc]] | target, address, text arguments; with a reply: its address pattern and rule values |
| [[ui:exp.node.udp]] | target, payload; with a reply: its pattern |
| [[ui:exp.node.tcp]] | host, payload |
| [[ui:exp.node.mqtt]] | broker host, topic, payload |
| [[ui:exp.node.log]] | message |
| [[ui:exp.node.assert_body]] | expected text |
| [[ui:exp.node.assert_header]] | header name, expected text |
| [[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], a [[ui:exp.node.loop]]'s exit condition | value, expected value |
| [[ui:exp.node.wait_osc]] | address pattern, rule values |
| [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_ws]] | pattern |
| [[ui:exp.node.wait_mqtt]] | broker and topic (parameters only), pattern |
| [[ui:exp.node.wait_http]] | path pattern, conditions |
| [[ui:exp.node.impairment]] | listen and target (parameters only) |
| [[ui:exp.node.ws_connect]] | URL, header names and values |
| [[ui:exp.node.ws_send]] | payload |
| [[ui:exp.node.ws_close]] | reason |

Numbers — ports, timeouts, delays, statuses, typed OSC numbers — and the
listening addresses of waits are literal. An [[ui:exp.node.emulator]] renders
its own replies with what arrived (`{{request.…}}`) and the parameters; see
[faults](faults.md#emulator).

**Parameters only.** Some fields are opened before the first step, when no
variable exists yet: a [[ui:exp.node.wait_mqtt]]'s broker and topic, an
[[ui:exp.node.impairment]]'s listen and target. They take text and parameters,
nothing else (`node.params_only`).

**Checked like literals.** A field that uses only parameters is resolved
before the run and checked as the text the run will send: a URL must be
`http://` or `https://`, an OSC target `IP:port` or `host:port`, a header name valid. A field
with variables or generators is checked when it runs.

## Preview {#preview}

When the selected node has a template, its properties show what it will do
with the values known now: [[ui:exp.preview]] for a send,
[[ui:exp.previewWait]] for a wait, [[ui:exp.previewCheck]] for a comparison.
The engine resolves it, with the same code a run uses, so the preview never
disagrees with the run.

- Parameters come from the active profile.
- Variables come from what the editor has seen in this session: the last run's
  steps, and [[ui:exp.sendNow]].
- A stored secret shows as `••••`.
- A name without a value yet stays as written, and the preview lists it. A
  secret that is not stored is listed apart.
- Generators use the experiment's pinned seed, or `0` when none is pinned, as
  the node's first execution. With a pinned seed, the preview shows the
  generated values the node's first execution in a run will send.

## Extracting values {#extract}

[[ui:exp.node.extract]] reads one value of the latest HTTP response on its
path and writes it into a variable.

| Field | What |
| --- | --- |
| [[ui:exp.variable]] | the variable to write; the naming rules of parameters apply |
| [[ui:exp.extractFrom]] | where the value comes from (below) |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] or [[ui:exp.pattern]] | what to read, depending on the source |

| [[ui:exp.extractFrom]] | Reads | Value |
| --- | --- | --- |
| [[ui:exp.from.json]] | the body as JSON, at a path | the JSON value: text, number, object, array |
| [[ui:exp.from.header]] | the first header with that name, in any case | text |
| [[ui:exp.from.status]] | the status code | a number |
| [[ui:exp.from.body]] | the whole body | text |
| [[ui:exp.from.regex]] | the first match in the body | capture group 1 if the pattern has one, else the whole match |

**JSON paths.** `$.token`, `$.items[0].id`, `$["a b"]`, `$['a b']['c-d']`;
the leading `$.` may be left out (`token`, `items[0].id`), and `$` alone is
the whole body.

**Regular expressions** use the syntax of the Rust `regex` engine, which has no
look-around and no back-references. The match is searched anywhere in the
body; anchor it with `^` and `$` when that matters.

The step fails, naming what is missing, when:

- no HTTP request ran before it on this path (`check.no_response`; the editor
  already refuses a graph where none can, `graph.needs_http`);
- the body is not JSON, or the path is not in it;
- the header is not there, or the pattern does not match;
- the body is over the 256 KiB a response keeps, for a JSON path or the whole
  body, and for a pattern that matched nothing in the part kept
  (`extract.truncated`).

The timeline shows the value written: `token = abc123`.

::: tip Extract by clicking
[[ui:exp.sendNow]] on an [[ui:exp.node.http]] shows its JSON response. Click a
value in it: an [[ui:exp.node.extract]] node is added after the request, with
the path filled in and a name taken from the key, and the value is known to
the preview at once.
:::

## Variables {#variables}

A variable holds a JSON value. These nodes write one:

| Node | Writes | On the output |
| --- | --- | --- |
| [[ui:exp.node.extract]] | the extracted value | its output |
| [[ui:exp.node.wait_osc]], [[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_http]], [[ui:exp.node.wait_ws]] | what arrived, default name `reply` (`request` for HTTP) | [[ui:exp.portMatched]] only |
| [[ui:exp.node.osc]], [[ui:exp.node.udp]] with [[ui:exp.expectReply]] | the reply, default name `reply` | its output |

What a wait writes is an object; later fields read its parts:

| Wait | Fields |
| --- | --- |
| OSC | `address`, `args`, `from`, `ms` |
| UDP | `text`, `hex`, `bytes`, `from`, `ms`, and `match` with a pattern |
| MQTT | `topic`, and the fields of UDP |
| WebSocket | the fields of UDP, and `json` when the message is JSON |
| HTTP request | `method`, `path`, `query`, `headers`, `body`, `json`, `params`, `from`, `ms` |

`ms` is the time from the branch's latest action to the arrival. The exact
contents are in [the nodes' reference](nodes.md).

### Where a variable is known {#visibility}

A variable exists from the output that writes it onward, on the paths that
pass through that output:

- After a merge of alternative paths — the [[ui:exp.yes]] and [[ui:exp.no]]
  of a branch meeting again — only what **every** path set is known.
- After [[ui:exp.node.join]], what **any** branch into it set is known: all of
  them ran.
- After a [[ui:exp.node.loop]]'s [[ui:exp.portDone]] or [[ui:exp.portLimit]],
  and in its exit condition, what every iteration of the body sets is known.
- A wait's variable is not known after its [[ui:exp.portTimeout]] output.

Each parallel branch works on its own copy of the variables. A Join merges the
copies in the order of its incoming wires, the later wire winning a name both
set, so the result never depends on which branch finished first. See
[how a run moves](flow.md#parallel).

## Comparing values {#compare}

[[ui:exp.node.assert_value]] fails the run when a comparison does not hold;
[[ui:exp.node.branch_value]] leaves through [[ui:exp.yes]] or [[ui:exp.no]];
a [[ui:exp.node.loop]] uses the same comparison as its exit condition. Each has
a [[ui:exp.value]], a [[ui:exp.operator]] and an [[ui:exp.expected]] value, and
both texts are templates:

| [[ui:exp.value]] | [[ui:exp.operator]] | [[ui:exp.expected]] |
| --- | --- | --- |
| `{{status}}` | [[ui:exp.op.lt]] | `300` |
| `{{reply.args[0]}}` | [[ui:exp.op.eq]] | `{{nonce}}` |

| [[ui:exp.operator]] | Holds when |
| --- | --- |
| [[ui:exp.op.eq]], [[ui:exp.op.ne]] | the two are equal (not equal) — as numbers when both are numbers (`200` equals `200.0`), else as exact text, case included |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | as numbers; a side that is not a number fails the step (`compare.not_numbers`) instead of a quiet *no* |
| [[ui:exp.op.contains]] | the value contains the expected text, case included |
| [[ui:exp.op.matches]] | the regular expression in the expected value matches anywhere in the value |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | the value is empty, or not, after trimming spaces; the expected value is not used |

A number is text that reads as one after trimming spaces: `42`, `-1.5`,
`1e3`. The timeline shows the comparison as made, `401 = 200`, each side cut
to 120 characters.

## Secrets {#secrets}

A token or password goes into a field as `{{secret.NAME}}`. The experiment
file keeps only the name; the value stays where it is stored and never
reaches the interface.

### Where secrets live {#secret-stores}

| Where Signal Lab runs | Store | From the interface |
| --- | --- | --- |
| Desktop app on Windows | the Windows Credential Manager, under the service `SignalLab`, one entry per name | set, replace, remove |
| Desktop app on Linux | none: a run that needs a secret fails with `secret.unsupported` | — |
| Server | the environment variable `SIGNALLAB_SECRET_<NAME>`, else the file `<NAME>` in its secrets folder, `/run/secrets/signallab` unless set otherwise | read-only |
| `signallab` on the command line | as a server, or the Windows Credential Manager with `--secrets system` | — |

The heading of the [[ui:exp.secrets]] section has a tooltip that says which of
these applies where you are: the Windows store, the server's environment and
files, or — in the desktop app on Linux — that there is no store. There,
`secret.unsupported` says the secrets are kept in the Windows Credential
Manager, which the system does not have.

A secret belongs to the computer or the server, not to one experiment: two
experiments that use `{{secret.API_TOKEN}}` use the same value.

On a server, the environment variable wins over the file. A file's trailing
line break is not part of the value, and an empty file counts as no secret.
The server's folder is set with `--secrets-dir` or `SIGNALLAB_SECRETS_DIR`;
see [the server](../server/index.md). For the command line, see
[`signallab run`](../automation/cli.md#cli-run).

| Rule | Limit |
| --- | --- |
| Name | starts with a letter or `_`, then letters, digits and `_`; at most 128 characters |
| Value | not empty, at most 16 KiB |

### Setting a secret {#set-secret}

On Windows:

1. Open [[ui:exp.params]]. The [[ui:exp.secrets]] section lists every secret
   the experiment's fields use, each [[ui:exp.secretStored]] or
   [[ui:exp.secretMissing]].
2. Press [[ui:exp.secretSet]] next to the name, or [[ui:exp.addSecret]] for a
   name no field uses yet.
3. Type the value — the field shows dots — and press [[ui:exp.secretSave]] or
   <kbd>Enter</kbd>. The field is cleared; nothing can read the value back.

[[ui:exp.secretReplace]] stores a new value and [[ui:exp.secretRemove]]
deletes it from the credential store. A name stored in this session is offered
in the suggestions too.

In a browser connected to a server the section only says
[[ui:exp.secretOnServer]] or [[ui:exp.secretNotOnServer]]: set the value where
the server runs, in one of two ways:

```bash
# in the server's environment
SIGNALLAB_SECRET_API_TOKEN='…'
# or as a file in its secrets folder
printf '%s' '…' > /run/secrets/signallab/API_TOKEN
```

A file is read each time a run starts, so a changed file counts from the next
run; a changed environment variable needs the server restarted.

### Before a run {#secret-check}

Every secret the run's fields use must be stored. A missing one stops the run
before any traffic, at the first node and field that use it
(`secret.missing`). [[ui:exp.sendNow]] checks the same for its node.

### Masking {#masking}

While a run or a [[ui:exp.sendNow]] uses secrets, every occurrence of their
values is replaced by `••••` in everything that leaves the engine:

- step texts, errors and the variables a step wrote;
- the run report;
- [[ui:exp.sendNow]]'s result, the HTTP response it shows included;
- the [[ui:dock.inspector]]'s frames, captured while the run lasts — in a hex
  dump each byte of a value becomes `*`, so offsets stay true.

Basic authentication sends `name:password` in base64; when either part holds a
secret, that base64 text is masked too. The traffic itself carries the real
value. The preview shows a stored secret as `••••`. An
[[ui:exp.node.emulator]]'s replies cannot use secrets.

## Commands {#commands}

The preview is [`experiment_resolve`](../api/commands.md#experiment_resolve);
secrets are listed, set and removed with
[`secret_status`](../api/commands.md#secret_status),
[`secret_set`](../api/commands.md#secret_set) and
[`secret_delete`](../api/commands.md#secret_delete). No command returns a
secret's value.
