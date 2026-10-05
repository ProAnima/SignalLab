# Security policy

## Reporting a vulnerability

Please report security problems **privately**, not in a public issue:

- through GitHub: [Report a vulnerability](https://github.com/ProAnima/SignalLab/security/advisories/new)
  (Security → Advisories), or
- by e-mail to **info@proanima.net**.

Say what is affected (the desktop app, the server, the Docker image, the command
line), the version, and how to reproduce it. We answer within a few working
days, keep you informed while we fix it, and credit you in the release notes
unless you prefer otherwise.

## Supported versions

| Version | Supported |
| --- | --- |
| 1.x (the latest release) | yes |
| 0.x | no — please update |

The desktop app updates itself from signed releases; a server updates by
pulling the newest image.

## What is in scope

- The server's authentication and its Host, Origin and session rules, the
  token, the HTTP API and its WebSocket.
- How secrets are stored and masked, and anything that would let a page, a file
  or a network peer run commands or read secrets.
- The installers, the updater and release signing.

Signal Lab sends traffic where you point it: Storm, Scanner and Broadcast
reaching the hosts you typed is their purpose, not a vulnerability. The
[security model of the server](https://proanima.github.io/SignalLab/server/security.html)
describes what it protects and how.
