---
title: The studio's hub
---

# The studio's hub

Signal Lab talks to one service of the studio's: **ProAnima Hub**
(`https://hub.proanima.net`, its own repository `ProAnima/pas-Hub`), which
serves every app of ProAnimaStudio. Signal Lab is its project `signal-lab`.
The hub does two things for it:

- **Updates.** The desktop app's updater asks the hub which release to install.
- **Feedback.** The **✉** form goes to the hub, which mails it to the developers.

The app holds no secret for either. Its code is open, so a secret could not be
kept there; the hub keeps the mailbox's password and decides who gets which
release.

## Updates {#updates}

`plugins.updater.endpoints` in `src-tauri/tauri.conf.json`, asked in turn:

1. `https://hub.proanima.net/v1/signal-lab/update/{{target}}/{{arch}}/{{current_version}}`
   — the release promoted in the hub's `stable` channel, if it is newer and this
   install is within its rollout: `200` with that release's own `latest.json`
   (its signature untouched; its files are links through the hub to GitHub, so
   downloads are counted). `204` says there is nothing, and the updater looks
   no further — so a paused channel or a partial rollout holds.
2. `https://github.com/ProAnima/SignalLab/releases/latest/download/latest.json`
   — only when the hub cannot be reached or answers an error: the latest
   published release, for everyone.

Either way the updater installs nothing whose signature does not match the
public key built into the app (`plugins.updater.pubkey`), and only the release
workflow signs. The check sends `X-Install-Id`, a random UUID made once
(`installId` in `src/lib/platform.ts`, kept with the app's settings), so a
rollout reaches the same installs as it widens; the hub keeps it with the
version, system and channel, and no address. *Check once a day* in About says
so in its tip, and turns the automatic look off.

**After a release is published** (and only then — drafts never reach the hub):
in the hub's console, *Signal Lab → Releases*, *Read from GitHub* (or wait for
the hourly read), choose the version in `stable`, set the rollout (say 20 %,
then 100 %) and save. Until a version is promoted, the hub offers nothing and
the app stays as it is.

## Feedback {#feedback}

`POST https://hub.proanima.net/v1/signal-lab/feedback`, `multipart/form-data`
(`engine/src/feedback.rs`, the command `feedback_send`):

| Field | | Limit |
| --- | --- | --- |
| `message` | required, the text | 20 000 characters |
| `email` | optional, becomes Reply-To | one address |
| `meta` | JSON: `version`, `os`, `arch`, `mode`, `lang`, `screen` | 16 KB |
| `screenshot` | files; PNG, JPEG, WebP or GIF, known by their bytes | 6, 8 MB each |
| `log` | files; UTF-8 text | 4, 2 MB each |
| | everything together | 15 MB |

The engine checks these limits before it uploads anything (they are the hub's,
`src/feedback/form.rs` there: change both together). The hub answers
`202 {"id"}` — the id is in the mail too — or `{"error": {"code", "params"}}`,
which the app shows in the reader's language: `feedback.message_required`,
`message_too_long`, `email_invalid`, `too_many_files`, `file_type`,
`file_too_large`, `too_large`, `rate_limited` (`retry_after_s`), `send_failed`
(`id`), `invalid`, `disabled` (feedback switched off for the project). Anything
else the hub says is `feedback.failed` with its status; a network failure is a
`transport.*` cause about the hub. Limits per sender: 5 in ten minutes, 20 a
day.

**The project's mail** is set in the hub's console, *Signal Lab → Settings*:
to `info@proanima.net`, from `Signal Lab <signal-labs@proanima.net>`, SMTP
`smtp.timeweb.ru`, port 465, TLS, login `signal-labs@proanima.net`; the
password is typed there once (stored sealed, never shown again), and *Send a
test mail* proves it. Feedback is on only when all of that is set.

## Another hub {#another-hub}

`SIGNALLAB_HUB_URL` points the feedback form elsewhere: set when the engine is
compiled, or in a running engine's environment (the end-to-end tour points it
at its stand-in, `scripts/e2e/fixtures.mjs`; `engine/tests/feedback.rs` at
one of its own). The updater's addresses are the app's configuration: another
hub means another `endpoints` list, which `tests/delivery.test.mjs` keeps equal
to `engine/src/hub.rs`.
