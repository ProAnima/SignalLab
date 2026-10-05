---
layout: home
title: Signal Lab
description: Signal Lab ist ein Testlabor für OSC und Netzwerkprotokolle, das den Verkehr von Showsteuerungstechnik, Geräten und Diensten sendet, mitschneidet, emuliert und stört und daraus wiederholbare Tests macht.
hero:
  name: Signal Lab
  text: Ein Testlabor für OSC und Netzwerkprotokolle
  tagline: "Senden und beobachten Sie den Verkehr, den Ihre Geräte und Dienste sprechen, vertreten Sie das Gerät oder die API, die es noch nicht gibt, stören Sie das Netzwerk mit Absicht — und führen Sie dieselben Prüfungen erneut aus: aus der App, einem Skript oder der CI."
  actions:
    - theme: brand
      text: Loslegen
      link: /de/guide/
    - theme: alt
      text: Herunterladen
      link: https://github.com/ProAnima/SignalLab/releases/latest
features:
  - title: Jedes Protokoll im Rack
    details: OSC mit typisierten Argumenten, rohes UDP und TCP, HTTP mit Basic-, Bearer- und Digest-Authentifizierung, WebSocket und MQTT 3.1.1 — dazu Broadcast, Multicast, Subnetz-Sweeps und ein Erkennungsempfänger.
    link: /de/protocols/osc
  - title: Ein Inspektor für alles
    details: Jeder Frame, den die Werkzeuge senden und empfangen, in einer Zeitleiste, dekodiert und mit seinen Bytes. Filtern und exportieren Sie sie, und senden Sie einen mitgeschnittenen Frame Byte für Byte erneut.
    link: /de/tools/inspector
  - title: Eine Bibliothek von Signalen
    details: Geben Sie der Nachricht, die funktioniert hat, einen Namen, legen Sie sie in einem Ordner ab und senden Sie sie mit Ctrl+K von überall erneut — OSC, rohes UDP, eine HTTP-Anfrage oder eine MQTT-Veröffentlichung.
    link: /de/tools/signals
  - title: Emulatoren
    details: Übernehmen Sie die Gegenseite — eine nachgebildete HTTP-API, ein OSC-, UDP- oder TCP-Gerät, einen MQTT-Broker — mit Regeln, Sequenzen, Verzögerungen, Fehlverhalten und Ausfällen nach Zeitplan.
    link: /de/tools/emulators
  - title: Netzwerkstörung
    details: Ein Relais zwischen einem Client und seinem Ziel, das Latenz, Jitter, Verlust, Duplikate, Umordnung oder eine Bandbreitengrenze hinzufügt oder TCP-Verbindungen zurücksetzt — mit Startwert, sodass derselbe Verkehr dasselbe Schicksal erleidet.
    link: /de/tools/impairment
  - title: Experimente
    details: Visuelle Testabläufe — senden, auf die Antwort warten, sie prüfen, verzweigen, Schleifen, Zweige parallel ausführen — mit Parametern, Profilen, Geheimnissen und einem Bericht für jeden Durchlauf.
    link: /de/experiments/
  - title: Lasttests
    details: Setzen Sie eine HTTP-Anfrage einer konstanten Rate, einer Rampe, Stufen, einer Spitze oder zufälligen Ankünften aus, messen Sie p50 bis p99, Fehler und die erreichte Rate, und lassen Sie den Durchlauf an Schwellenwerten scheitern.
    link: /de/experiments/load
  - title: Automatisierung
    details: Führen Sie Experimente ohne Fenster mit der Kommandozeile signallab aus — Exit-Codes, JUnit-Berichte, eine GitHub Action — oder überlassen Sie sie per MCP einem KI-Assistenten.
    link: /de/automation/cli
  - title: Server und API
    details: Dieselbe Oberfläche im Browser und dieselbe Engine auf einem Labor-PC oder in Docker, angemeldet mit einem Token — und eine HTTP-API für jeden Befehl und jeden Durchlauf.
    link: /de/server/
---

Signal Lab läuft als Desktop-App unter Windows und Linux oder als Server, den Sie im
Browser öffnen. Neu dabei? Lesen Sie, [was Signal Lab ist](guide/index.md), [installieren Sie es](guide/install.md),
und gehen Sie dann die [ersten Schritte](guide/first-steps.md): eine Nachricht senden und empfangen, ein Signal
speichern, eine emulierte API antworten lassen und ein kleines Experiment ausführen — alles auf diesem Computer.
