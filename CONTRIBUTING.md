# Contributing to Signal Lab

Thank you for helping. Bug reports, ideas, translations, documentation and code
are all welcome.

## Before you start

- **Questions and bugs:** open an [issue](https://github.com/ProAnima/SignalLab/issues/new/choose).
  For a security problem, read [SECURITY.md](SECURITY.md) instead — please do not
  open a public issue.
- **A larger change:** open an issue first and describe what you want to do, so we
  can agree on the approach before you spend time on it.
- Everyone taking part follows the [code of conduct](CODE_OF_CONDUCT.md).

## Setting up

Signal Lab is a React/TypeScript interface over a Rust engine, run as a Tauri
desktop app or as a server for browsers. You need Node 24, Rust (the version in
`rust-toolchain.toml`) and, on Windows, the MSVC build tools and WebView2.

```bash
npm install
npm run tauri dev     # the desktop app with live reload
```

The [contributors' documentation](https://proanima.github.io/SignalLab/develop/)
explains the architecture, building, the checks, localization and how the
documentation is written. [CLAUDE.md](CLAUDE.md) lists the invariants the code
keeps — read the ones near what you change.

## Making a change

1. Work on a branch of your fork; keep a pull request to one subject.
2. Follow the code around you: its naming, comments and idioms.
3. Add or change tests with the behaviour. Every engine error has a translated
   text; every visible text comes from the dictionaries in `src/lib/locales/`
   (English is the source; the build fails on a missing key in any language).
4. Change the documentation in `docs/` in the same pull request when what users
   see changes — English first; see
   [Writing the documentation](https://proanima.github.io/SignalLab/develop/writing-docs.html).
5. Note user-visible changes under **Unreleased** in [CHANGELOG.md](CHANGELOG.md).
6. Run the checks before you push:

   ```bash
   npm run check        # what CI runs: unit tests, types and build, clippy, Rust tests
   npm run e2e          # the end-to-end tour of every screen
   ```

## Things that are easy to break

- **Never add a default target that points at a host you do not own.** Storm,
  Scanner and Broadcast send real traffic; defaults stay on loopback (a test
  checks).
- The engine never builds sentences: it fails with an error code the interface
  translates.
- Every clickable thing is a real `<button>`; every field has a name; help goes
  in tooltips, not captions on screen.
- The server is safe by default: no token means loopback only.

## Translations

The interface and the documentation speak eleven languages. To correct a
translation, edit `src/lib/locales/<code>.ts` (the interface) or
`docs/<code>/…` (the documentation) and open a pull request. Adding a language
is described in [Localization](https://proanima.github.io/SignalLab/develop/localization.html).

## License

By contributing you agree that your contribution is licensed under the
[MIT License](LICENSE), like the rest of the project.
