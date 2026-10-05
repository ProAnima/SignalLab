---
layout: home
title: Signal Lab
description: Signal Lab은 공연 제어 장비, 장치, 서비스의 트래픽을 보내고, 캡처하고, 에뮬레이션하고, 네트워크 장애를 적용해 반복 가능한 테스트로 만드는 OSC 및 네트워크 프로토콜 테스트 랩입니다.
hero:
  name: Signal Lab
  text: OSC 및 네트워크 프로토콜 테스트 랩
  tagline: 장비와 서비스가 주고받는 트래픽을 보내고 지켜보며, 아직 없는 장치나 API의 자리를 대신하고, 네트워크를 일부러 망가뜨립니다. 그런 다음 같은 검사를 앱, 스크립트, CI에서 다시 실행합니다.
  actions:
    - theme: brand
      text: 시작하기
      link: /ko/guide/
    - theme: alt
      text: 다운로드
      link: https://github.com/ProAnima/SignalLab/releases/latest
features:
  - title: 랙의 모든 프로토콜
    details: 타입이 지정된 인수를 쓰는 OSC, 원시 UDP와 TCP, Basic·Bearer·Digest 인증을 지원하는 HTTP, WebSocket, MQTT 3.1.1에 더해 브로드캐스트, 멀티캐스트, 서브넷 스윕, 탐색 리스너까지 갖추었습니다.
    link: /ko/protocols/osc
  - title: 모든 트래픽을 하나의 인스펙터에서
    details: 도구가 보내고 받는 모든 프레임을 하나의 타임라인에서 디코딩된 내용과 바이트로 보여 줍니다. 필터링하고 내보내며, 캡처한 프레임을 바이트 단위 그대로 재전송할 수 있습니다.
    link: /ko/tools/inspector
  - title: 신호 라이브러리
    details: 잘 동작한 메시지에 이름을 붙여 폴더에 정리해 두고, 어디서든 Ctrl+K로 다시 보냅니다. OSC, 원시 UDP, HTTP 요청, MQTT 발행 모두 가능합니다.
    link: /ko/tools/signals
  - title: 에뮬레이터
    details: 상대편 역할을 맡습니다. 모의 HTTP API, OSC·UDP·TCP 장치, MQTT 브로커가 규칙, 시퀀스, 지연, 장애, 일정에 따른 다운까지 재현합니다.
    link: /ko/tools/emulators
  - title: 네트워크 장애
    details: 클라이언트와 대상 사이의 릴레이가 지연 시간, 지터, 손실, 중복, 순서 바뀜, 대역폭 제한을 더하거나 TCP 연결을 리셋합니다. 시드를 따르므로 같은 트래픽은 같은 결과를 겪습니다.
    link: /ko/tools/impairment
  - title: 실험
    details: 보내기, 응답 대기, 검사, 분기, 루프, 병렬 분기 실행으로 이루어진 시각적 테스트 흐름입니다. 매개변수, 프로필, 시크릿을 쓰며 실행마다 보고서가 남습니다.
    link: /ko/experiments/
  - title: 부하 테스트
    details: HTTP 요청에 일정한 속도, 램프, 계단식, 스파이크, 무작위 도착 부하를 걸고 p50부터 p99까지의 지연, 오류, 실제 달성한 속도를 측정하며, 임계값을 넘으면 실행을 실패로 판정합니다.
    link: /ko/experiments/load
  - title: 자동화
    details: signallab 명령줄로 창 없이 실험을 실행합니다. 종료 코드, JUnit 보고서, GitHub Action을 지원하며, MCP로 AI 어시스턴트에게 맡길 수도 있습니다.
    link: /ko/automation/cli
  - title: 서버와 API
    details: 같은 인터페이스를 브라우저에서, 같은 엔진을 랩 PC나 Docker에서 토큰으로 로그인해 사용합니다. 모든 명령과 실행에 HTTP API가 있습니다.
    link: /ko/server/
---

Signal Lab은 Windows와 Linux에서 데스크톱 앱으로 실행하거나, 브라우저로 여는 서버로 실행합니다.
처음이십니까? [Signal Lab이 무엇인지](guide/index.md) 읽고 [설치한](guide/install.md) 다음,
[첫 단계](guide/first-steps.md)를 따라 해 보십시오. 메시지를 보내고 받고, 신호를 저장하고,
에뮬레이션된 API의 응답을 받고, 작은 실험을 실행하는 과정을 모두 이 컴퓨터에서 진행합니다.
