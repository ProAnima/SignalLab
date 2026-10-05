---
layout: home
title: Signal Lab
description: "Signal Lab est un banc d’essai pour OSC et les protocoles réseau : il envoie, capture, émule et dégrade le trafic des équipements de régie, des appareils et des services, et en fait des tests reproductibles."
hero:
  name: Signal Lab
  text: "Un banc d’essai pour OSC et les protocoles réseau"
  tagline: "Envoyez et observez le trafic que parlent vos équipements et vos services, remplacez l’appareil ou l’API qui n’est pas encore là, cassez le réseau exprès — puis relancez les mêmes vérifications depuis l’application, un script ou la CI."
  actions:
    - theme: brand
      text: "Commencer"
      link: /fr/guide/
    - theme: alt
      text: "Télécharger"
      link: https://github.com/ProAnima/SignalLab/releases/latest
features:
  - title: "Tous les protocoles du rack"
    details: "OSC avec arguments typés, UDP et TCP bruts, HTTP avec authentification Basic, Bearer et Digest, WebSocket et MQTT 3.1.1 — plus la diffusion, la multidiffusion, le balayage de sous-réseaux et une écoute de découverte."
    link: /fr/protocols/osc
  - title: "Un seul Inspecteur pour tout"
    details: "Chaque trame que les outils envoient et reçoivent, sur une seule chronologie, décodée et avec ses octets. Filtrez-la, exportez-la, et rejouez une trame capturée octet par octet."
    link: /fr/tools/inspector
  - title: "Une bibliothèque de signaux"
    details: "Nommez le message qui a fonctionné, rangez-le dans un dossier et renvoyez-le de n’importe où avec Ctrl+K — OSC, UDP brut, une requête HTTP ou une publication MQTT."
    link: /fr/tools/signals
  - title: "Émulateurs"
    details: "Jouez l’autre côté — une API HTTP fictive, un appareil OSC, UDP ou TCP, un broker MQTT — avec des règles, des séquences, des délais, des défaillances et des pannes programmées."
    link: /fr/tools/emulators
  - title: "Dégradation du réseau"
    details: "Un relais entre un client et sa cible qui ajoute latence, gigue, pertes, doublons, réordonnancement ou une limite de bande passante, ou qui réinitialise les connexions TCP — avec une graine, pour que le même trafic connaisse le même sort."
    link: /fr/tools/impairment
  - title: "Expériences"
    details: "Des scénarios de test visuels — envoyer, attendre la réponse, la vérifier, bifurquer, boucler, exécuter des branches en parallèle — avec paramètres, profils, secrets et un rapport pour chaque exécution."
    link: /fr/experiments/
  - title: "Tests de charge"
    details: "Soumettez une requête HTTP à un débit constant, une rampe, des paliers, un pic ou des arrivées aléatoires, mesurez de p50 à p99, les erreurs et le débit atteint, et faites échouer l’exécution sur des seuils."
    link: /fr/experiments/load
  - title: "Automatisation"
    details: "Exécutez les expériences sans fenêtre avec la ligne de commande signallab — codes de sortie, rapports JUnit, une GitHub Action — ou confiez-les à un assistant d’IA via MCP."
    link: /fr/automation/cli
  - title: "Serveur et API"
    details: "La même interface dans un navigateur et le même moteur sur un PC de laboratoire ou dans Docker, avec connexion par jeton — et une API HTTP pour chaque commande et chaque exécution."
    link: /fr/server/
---

Signal Lab fonctionne comme application de bureau sous Windows et Linux, ou comme serveur que vous ouvrez dans un
navigateur. Vous découvrez l’outil ? Lisez [ce qu’est Signal Lab](guide/index.md), [installez-le](guide/install.md),
puis suivez les [premiers pas](guide/first-steps.md) : un message envoyé et reçu, un signal
enregistré, une API émulée qui répond et une petite expérience exécutée — le tout sur cet ordinateur.
