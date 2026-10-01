# Changelog

Notable changes to Signal Lab. Versions follow [semantic versioning](https://semver.org);
dates are release dates.

Write changes under **Unreleased** as they land. `npm run release -- <version>` turns
that section into the version's section, and the release workflow uses it as the
release notes — so what is written here is what users read. See
[docs/delivery.md](docs/delivery.md).

## [Unreleased]

### Added

- **Experiments that use data.** Parameters with defaults and target profiles, a
  `{{template}}` language with seeded generators, *Extract* from a response,
  *Check value* and *Branch on value*, a resolved preview of every field, and
  *Run with…* for one-run overrides and a seed.
- **Secrets** from the Windows Credential Manager (`{{secret.NAME}}`), masked in the
  timeline, previews, responses, reports and the Inspector.
- **Wait for OSC** (OSC 1.0 address patterns, argument rules) and **Wait for UDP**
  (any, contains, regex, hex) with *Matched* and an optional *Timeout* output.
  Listeners open when the run starts, so a fast reply is not missed; the reply is a
  variable (`{{reply.args[0]}}`). *Listen now* runs one wait on its own. New template
  *OSC ping → reply*.
- **Errors in your language.** The experiment engine reports a code, values, the node
  and field, and the system's own text; the interface shows *Node · Field — message*
  with the technical detail folded underneath, in English or Russian. Network
  failures are told apart: refused, timed out, name not found, unreachable, port in
  use, TLS.
- **Linux builds** (`.deb`, `.rpm`, AppImage) next to the Windows installers.
- **CI** on every push and pull request (Windows and Linux), a release workflow that
  builds both platforms from a tag into a draft release with checksums, and the
  local commands `npm run check`, `npm run build:linux` and `npm run release`.

### Changed

- Switching screens keeps everything: typed values, the last response, running
  monitors and the scroll position.
- The version is written once, in `package.json`; the app, the installers and the
  engine read it from there.

### Fixed

- `package-lock.json` carried version 0.1.0 instead of the app's version.

## [0.3.1] - 2026-08-27

### Fixed

- The 0.3.0 installers were built from the commit before the cancellation fixes; 0.3.1
  ships them. A scan no longer reports an empty table, and Stop stops the work it
  started (storm, impairment relay, broadcast, HTTP burst, scanner).

## [0.3.0] - 2026-08-26

### Added

- **Signal library**: named OSC, UDP, HTTP and MQTT packets, grouped and searchable,
  fired from any screen with `Ctrl+K`; any Inspector frame can be saved as a signal.
- **MQTT 3.1.1 client**: one live connection as a job, a topic tree, QoS 0/1/2,
  retained values and last-will.

### Fixed

- Job telemetry was dropped before the job id was known, so fast jobs looked empty;
  Stop aborted only the supervising task and left the work running.

## [0.2.0] - 2026-08-26

### Added

- The app's own icon, and NSIS and MSI installers in English and Russian.
- Broadcast, multicast and CIDR sweep with a discovery responder, the packet Inspector,
  a bilingual interface, and the MIT license.

### Fixed

- Storm and the broadcast beacon reported a count up to 250 ms old at the end of a run.
- Checkbox, row and slider alignment.

[Unreleased]: https://github.com/ProAnima/SignalLab/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/ProAnima/SignalLab/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ProAnima/SignalLab/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ProAnima/SignalLab/releases/tag/v0.2.0
