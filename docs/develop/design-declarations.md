---
title: Design declarations
---

# SignalLab design declarations

::: info A design note
Written while the feature was being designed and kept as the record of why it is
the way it is. What shipped is described in the user documentation, which is
the one to trust where the two differ.
:::

1. **One experiment, one visible result.** The graph is an executable document. Every run records the document version, each step's outcome, and a final result.
2. **The canvas is for relationships.** Parameters live in a side panel. One meaningful action is one node; fields such as URL, timeout, and headers do not become separate nodes.
3. **The shortest path is obvious.** Open a template, edit a target, run, inspect the highlighted step. Manual protocol tools remain available for single actions, keep what was typed, and hand a working request to the editor in one click. A new node continues the flow from the selected one and opens with its main field ready to type.
4. **Failures explain themselves.** Validation points at a node or connection. Runtime errors show the failed step and its observed value; no silent skips.
5. **Stop means stop.** An experiment owns its work and cancellation ends it. Every wait has a timeout, every repeat has a bound, and background activity has a visible owner.
6. **Playback is honest.** Display summaries, run events, and exact captured bytes have different storage. Truncated or missing bytes cannot be offered as an exact replay.
7. **Calm instrument UI.** A small palette of semantic colors, consistent spacing, clear type, strong focus states, and no decorative controls. The editor must remain usable at 900×600. The screen carries labels, values, states and errors — no explanatory captions: what a control means, its shortcut or its units is a localized tooltip on the control or its label.
8. **Portable definitions.** Experiment files are versioned data independent of the canvas implementation. Engine operations are shared with the direct protocol screens.
9. **Safe defaults.** Templates point to loopback. Broad traffic has explicit targets, bounded rates, and a visible preview before execution.
10. **One gesture, one undo.** A drag or an uninterrupted field edit is one history entry. Undo restores the complete document, including parameters and connections. Viewport navigation does not change the experiment; switching instruments preserves editing state.
11. **Navigation follows intent.** Search and timeline selection reveal and focus the corresponding node. Zoom keeps the canvas point under the cursor stable where scroll bounds allow it; fit shows the graph at a useful scale. Automatic arrangement changes positions only.
12. **Responsibilities have concrete boundaries.** Graph transformations and history are pure functions. A document hook owns load/save state; the viewport hook owns navigation. Rust validates documents once at the boundary, stores files separately from execution, and shares the same JSON templates with the UI. Introduce interfaces or abstractions when a real dependency needs substitution, without speculative layers.
13. **Opening is deliberate and reversible.** File parsing never replaces the working experiment. The user opens a checked preview; replacement is one undo action. Exports are independent snapshots, and opening a document never starts network work.

## First executable slice

A graph with Start, HTTP request, Delay, Assert status, status-based Branch, OSC send, UDP send, MQTT publish, HTTP body/header/latency checks, and End. It can be edited, validated, saved, run, stopped, and inspected. Listeners, bounded loops, parallel branches, and device populations follow after this path is reliable.

## Node catalogue and visual tokens

- The typed catalogue owns localized titles, descriptions, and categories. Node forms are separate from canvas interactions; transport execution stays in Rust.
- Actions use cyan, waits (*Observe*) mint, data steps blue, checks violet, and flow amber. A wait's Matched port is green; its optional Timeout port is amber and dashed. Color is accompanied by names, descriptions, and textual execution states. Selection keeps the mint focus ring.
- Use the shared 4/8/12/16 spacing scale, 6 px control radius, 8 px node/menu radius, and 12 px dialog radius. Canvas background, grid, and wires have named tokens.
- An output may have several wires: their nodes run in parallel, each branch with its own variables; a Join waits for every incoming wire and End completes the run once, after the last branch. A dragged wire always adds one; inserting into the flow is *A*, *Add next* or the ＋ on a wire. Ports take the pointer well beyond their dot (mostly away from the node) and a wire dropped beside a node connects to it.
- Palette search matches localized labels/descriptions and protocol identifiers. Arrow keys select; Enter inserts. Long lists scroll within the window.
- UI labels, flow events and errors have Russian and English translations. A failure is shown as *Node · Field — message*, with the system's own wording folded under *Technical details*; the same renderer (`lib/errors.ts`, `ErrorMessage`) serves the banner, the properties panel (next to the fields of the node that cannot run), the timeline, *Send now*, the console and the HTTP screen. Callers keep the failure, not its text, so switching language re-renders every message.
- Screens are mounted on first visit and kept hidden afterwards: switching tabs never discards typed values, responses, running monitors or the scroll position.
- Help is a tooltip. An element with `data-tip` gets one from the single `TooltipLayer`: above it (below when there is no room), after a short hover, at once on keyboard focus, gone on typing, a press, scrolling or Escape; it is drawn in the top layer, so it shows over modal dialogs, and it describes its element for screen readers. Hovering inside a text field does not show its label's tooltip. Empty states are a few words ("No packets yet"), and a label is a name and a unit ("Rate, pps"); what `0` means goes in the tooltip.
- HTTP checks consume the last response on the executed path. Body matching is case-sensitive; header names ignore case, header values do not. A missing match in a truncated body is inconclusive and fails explicitly.
- Text fields of actions and checks accept `{{templates}}`; the engine is the only resolver (editor preview and *Send now* ask it), so a field means the same thing in a run, a preview and a single send. Unknown names are errors, never empty strings. Details: [design-data.md](./design-data.md).
- Secret values live in the operating system's credential store and never leave the engine: no command returns one, *Send now* is executed by the engine, and everything reported while a value is in use — timeline, preview, responses, reports, Inspector — shows `••••` instead.
- MQTT publish currently supports a broker without credentials, QoS 0–2 and retain, with a 15-second overall timeout. Authentication profiles, listeners, retries and bounded loops remain later work.
