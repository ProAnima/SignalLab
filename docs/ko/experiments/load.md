---
title: 부하 테스트
description: HTTP 노드의 요청을 일정, 램프, 계단식, 스파이크, 무작위 도착 같은 부하 프로필로 보내고, 지연 시간, 오류, 달성한 속도를 측정해 임계값으로 판정하고, 두 실행을 비교합니다.
---

# HTTP 요청 부하 테스트

[[ui:exp.node.http]] 노드는 초당 요청 수 프로필에 따라 요청을 여러 번, 여러 개 동시에 보내고
돌아오는 결과를 측정할 수 있습니다. 지연 시간 백분위수, 오류, 도달한 속도입니다. 단계의 통과
여부는 임계값이 판정하며, [[ui:exp.compare]] 기능은 수치를 이전 실행의 수치와 나란히 보여 줍니다.

부하는 노드의 설정이지 별도의 노드가 아닙니다. 실험의 나머지 부분(에뮬레이터, 네트워크 장애
릴레이, 다른 분기)은 평소처럼 그 주위에서 실행됩니다.

## 요청에 부하 걸기 {#turn-on}

1. [[ui:exp.node.http]] 노드를 선택하고 요청을 채웁니다.
2. 노드의 속성에서 [[ui:exp.loadOn]] 옵션을 켭니다.
3. [[ui:exp.loadShape]]과 그 수치를 고릅니다. 그 아래의 [[ui:exp.loadChart]] 차트가 속도를 그리고,
   모두 몇 개의 요청을 몇 초 동안 보내는지 알려 줍니다.
4. [[ui:exp.loadConcurrency]] 값을 설정합니다. 한 번에 진행 중일 수 있는 요청의 수입니다.
5. [[ui:exp.thresholds]] 항목을 추가하거나 바꿉니다.
6. 실험을 실행합니다.

부하를 켜면 처음에는 30,000 ms 동안 초당 0에서 100 요청으로 올라가는 [[ui:exp.loadShape.ramp]]
프로필, 동시 요청 32개, 임계값 두 개([[ui:exp.metric.p95_ms]] < 500 ms,
[[ui:exp.metric.error_rate]] < 1%)로 시작합니다.

**부하는 반복과 재시도를 대신합니다.** 부하를 켜면 이 둘이 꺼지며, 부하와 함께 둘 중 하나라도
켜진 노드는 거부됩니다(`node.load_alone`). 실패한 요청은 다시 시도하지 않고 집계합니다. 부하
상태로 실행할 수 있는 것은 HTTP 요청뿐입니다(`node.load_unsupported`).

**요청은 한 번만 읽습니다.** 템플릿은 단계가 시작될 때 해석되므로 부하의 모든 요청은 같은
요청입니다. `{{counter}}`와 `{{uuid}}`도 모든 요청에 대해 한 값을 가집니다.
[템플릿](data.md#templates)을 참고하십시오.

**부하 전체에 클라이언트 하나.** [[ui:exp.cookies]] 옵션이 켜져 있으면 요청들은 실행의 쿠키
저장소를 공유하며, Digest 인증 상태도 하나를 공유하므로 챌린지 하나로 모든 요청에 응답합니다. 각
요청에는 노드 자체의 시간 제한이 적용됩니다.

**인스펙터에는 표본만 들어갑니다.** 100 ms마다 최대 한 번의 교환만 기록하므로, 부하가
[[ui:dock.inspector]]를 가득 채우지 않습니다.

## 프로필 {#profiles}

| [[ui:exp.loadShape]] | 설정 | 시간에 따른 속도 |
| --- | --- | --- |
| [[ui:exp.loadShape.constant]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | 처음부터 끝까지 같은 속도 |
| [[ui:exp.loadShape.ramp]] | [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadDuration]] | 한 속도에서 다른 속도로 직선으로 변화 |
| [[ui:exp.loadShape.steps]] | [[ui:exp.loadFrom]], [[ui:exp.loadStepBy]], [[ui:exp.loadEvery]], [[ui:exp.loadSteps]] | 처음 속도에서 시작해 단계마다 증가폭만큼 높이며, 각 단계는 같은 시간 동안 유지 |
| [[ui:exp.loadShape.spike]] | [[ui:exp.loadBase]], [[ui:exp.loadPeak]], [[ui:exp.loadAt]], [[ui:exp.loadSpikeFor]], [[ui:exp.loadDuration]] | 기본 속도로 가다가 정해진 시점부터 잠시 최고 속도, 그 뒤 다시 기본 속도 |
| [[ui:exp.loadShape.poisson]] | [[ui:exp.loadRate]], [[ui:exp.loadDuration]] | 무작위 도착, 평균적으로 지정한 속도 |

프로필 종류를 바꾸면 옮길 수 있는 값, 즉 실행 시간과 도달하는 최고 속도는 유지됩니다.

### 한도 {#limits}

| 설정 | 범위 |
| --- | --- |
| [[ui:exp.loadShape.constant]]과 [[ui:exp.loadShape.poisson]]의 [[ui:exp.loadRate]], [[ui:exp.loadPeak]] | 0.1–100,000 요청/초 |
| [[ui:exp.loadFrom]], [[ui:exp.loadTo]], [[ui:exp.loadBase]] | 0–100,000 요청/초 |
| [[ui:exp.loadShape.steps]]의 모든 단계(마지막 단계 포함) | 0–100,000 요청/초. 증가폭은 음수일 수 있습니다 |
| [[ui:exp.loadDuration]], [[ui:exp.loadEvery]] | 100–300,000 ms |
| [[ui:exp.loadSteps]] | 1–100, 그리고 모든 단계를 합쳐 최대 300,000 ms |
| 스파이크 | 0 ms보다 길어야 하며, 지속 시간이 끝나기 전에 끝나야 합니다 |
| [[ui:exp.loadConcurrency]] | 1–512 |
| [[ui:exp.thresholds]] | 최대 16개, 각 값은 0 이상의 숫자 |

요청이 하나도 나오지 않는 프로필은 거부됩니다(`load.nothing_planned`). 속도 범위는
[HTTP 버스트](../protocols/http.md)와 같습니다.

프로필은 실행 하나의 전체 길이인 300초까지 지속될 수 있습니다. 하지만 실행의
[시간 제한](flow.md#limit)은 모든 단계를 합쳐 계산하므로, 실험의 나머지 부분을 위한 여유를 남겨
두십시오.

### 요청 수 {#planned}

프로필의 요청 수는 속도를 시간에 따라 합한 값입니다.

| 프로필 | 요청 수 |
| --- | --- |
| [[ui:exp.loadShape.constant]], 1000 ms 동안 100/초 | 100 |
| [[ui:exp.loadShape.ramp]], 2000 ms 동안 0 → 100/초 | 100 |
| [[ui:exp.loadShape.steps]], 10/초에서 10/초씩 증가, 1000 ms 단계 3개 | 60 (10 + 20 + 30) |
| [[ui:exp.loadShape.spike]], 10/초에 1000 ms 시점부터 500 ms 동안 100/초, 전체 2000 ms | 65 |
| [[ui:exp.loadShape.poisson]], 10,000 ms 동안 200/초 | 평균 2000 |

## 일정 {#schedule}

n번째 요청은 프로필의 누적 요청 수가 n에 도달하는 순간에 보낼 차례가 되며, 첫 번째 요청은 바로
나갑니다. 모든 시점은 부하 시작 시각을 기준으로 계산하므로, 늦게 깨어나도 그 뒤의 요청이 밀리지
않고, 프로필이 나타내는 속도가 그대로 요청하는 속도가 됩니다.

[[ui:exp.loadShape.poisson]] 프로필은 도착 간격을 실행의 시드에서 무작위로 뽑습니다. 같은 시드는 같은
시점을 만들므로 무작위 부하도 정확히 반복할 수 있습니다. [시드](runs.md#seeds)를 참고하십시오.

**누락된 요청.** 진행 중인 요청은 최대 [[ui:exp.loadConcurrency]] 값만큼입니다. 모든 요청이 아직
응답을 기다리고 있으면 다음 요청은 빈자리를 기다립니다. 정해진 시점보다 50 ms 넘게 늦게 나가게
되면 늦게 보내지 않습니다. 그 요청과 그동안 차례가 된 다른 모든 요청을 건너뛰어 누락으로 집계하고,
아직 제시간에 보낼 수 있는 첫 요청부터 부하를 이어 갑니다. 누락된 요청이 많다면 서버나
[[ui:exp.loadConcurrency]] 설정이 프로필을 따라가지 못했다는 뜻입니다.

## 실행 중에 {#progress}

타임라인은 1초에 한 번 단계를 [[ui:exp.load]] 상태로 표시하며, 경과한 초, 보낸 요청, 지난 1초 동안의
속도, 지금까지의 p95, 실패한 요청을 함께 보여 줍니다. [[ui:common.stop]] 버튼은 부하를 즉시 끝내고
진행 중인 요청을 버립니다. 다른 분기에서 실패가 일어나면 1초 안에 부하가 끝납니다.

## 측정 항목 {#metrics}

마지막 응답 뒤에 단계의 측정값이 정해지며, 마지막 타임라인 이벤트와
[실행 보고서](runs.md#report)에 보관됩니다.

| 측정값 | 내용 |
| --- | --- |
| planned | 프로필이 합산한 요청 수([[ui:exp.loadShape.poisson]]: 평균) |
| sent | 응답을 받았거나 실패한 요청 |
| ok | 2xx 상태로 응답받은 요청 |
| failed | 그 밖의 상태이거나 아예 응답이 없는 요청 |
| missed | 모든 자리가 차 있을 때 차례가 되어 건너뛴 요청 |
| rps | 초당 보낸 요청: sent ÷ 프로필의 지속 시간. 마지막 요청이 나간 시각이 그보다 늦으면 ÷ 그 시각까지의 시간 |
| error_rate | sent 대비 failed의 비율(%) |
| min, mean, max | 가장 빠른 요청, 평균, 가장 느린 요청, ms |
| p50, p90, p95, p99 | 요청의 50, 90, 95, 99%가 그 이하였던 지연 시간, ms |
| received_bytes | 받은 본문 바이트의 합계 |
| statuses | 상태별 요청 수(`200`, `503`)와, 상태가 없으면 원인별 요청 수(`timeout`, `refused`, `reset` …) |
| seconds | 프로필의 각 초: 보낸 요청, 실패한 요청, 평균 지연 시간 |
| histogram | 지연 시간별 요청 수: 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10,000 ms 이하, 그리고 그보다 느린 요청 |

요청의 지연 시간은 요청을 보낸 시점부터 응답 전체를 읽을 때까지이며, 실패한 요청은 실패하는 데
걸린 시간으로 집계합니다. 백분위수는 폭 1%의 로그 구간에서 읽으며, 부하가 아무리 오래 실행되어도
실제 값과 0.5% 이내로 일치합니다.

## 임계값 {#thresholds}

임계값은 [[ui:exp.thresholdMetric]], [[ui:exp.thresholdOp]], [[ui:exp.thresholdValue]]으로 이루어진
한 행이며, [[ui:exp.thresholdAdd]] 버튼으로 추가합니다.

| [[ui:exp.thresholdMetric]] | 단위 |
| --- | --- |
| [[ui:exp.metric.p50_ms]], [[ui:exp.metric.p90_ms]], [[ui:exp.metric.p95_ms]], [[ui:exp.metric.p99_ms]] | ms |
| [[ui:exp.metric.mean_ms]], [[ui:exp.metric.max_ms]] | ms |
| [[ui:exp.metric.error_rate]] | 보낸 요청 대비 % |
| [[ui:exp.metric.rps]] | 달성한 초당 요청 수 |
| [[ui:exp.metric.missed]] | 요청 수 |

[[ui:exp.thresholdOp]] 항목은 `<`, `≤`, `>`, `≥` 중 하나입니다. 자주 쓰는 예는 다음과 같습니다.

| [[ui:exp.thresholdMetric]] | [[ui:exp.thresholdOp]] | [[ui:exp.thresholdValue]] | 단계가 실패하는 경우 |
| --- | --- | --- | --- |
| [[ui:exp.metric.p95_ms]] | `<` | 300 | 요청 스무 개 중 하나 이상이 300 ms 이상 걸렸을 때 |
| [[ui:exp.metric.error_rate]] | `<` | 1 | 요청의 1% 이상이 실패했을 때 |
| [[ui:exp.metric.rps]] | `≥` | 180 | 서버가 초당 180개의 요청을 받아 내지 못했을 때 |
| [[ui:exp.metric.missed]] | `≤` | 0 | 요청이 하나라도 건너뛰어졌을 때 |

파일에서 임계값은 `{ "metric": "p95_ms", "op": "lt", "value": 300 }`입니다. 지표는 `p50_ms`,
`p90_ms`, `p95_ms`, `p99_ms`, `mean_ms`, `max_ms`, `error_rate`, `rps`, `missed`이고, 비교는 `lt`,
`le`, `gt`, `ge`입니다.

임계값은 마지막 응답 뒤에 순서대로 읽습니다. 충족되지 않는 첫 임계값에서 단계가 실패하며
(`load.threshold`), 메시지에는 임계값과 측정값이 담기고, 실행도 함께 실패합니다. 임계값이 없으면 부하는 무엇을 측정했든 통과합니다.
다른 분기의 실패로 부하가 일찍 끝났다면 실행의 실패 원인은 임계값이 아니라 그 실패입니다.

## 결과 {#result}

단계가 통과하면 타임라인에 요청 수, 속도, p95, 실패 비율이 요약됩니다. 노드를 선택하면 속성에
[[ui:exp.loadResult]] 항목이 표시됩니다.

- 각 임계값과 측정값, ✓ [[ui:exp.thresholdHeld]] 또는 ✕ [[ui:exp.thresholdBroken]];
- [[ui:http.sent]], [[ui:exp.loadRps]], 비율과 함께 [[ui:exp.loadErrors]], [[ui:http.missed]];
- [[ui:http.p50]], [[ui:http.p90]], [[ui:http.p95]], [[ui:http.p99]],
  [[ui:http.avg]], [[ui:http.max]];
- [[ui:exp.loadPerSecond]]: 초마다의 요청(실패한 요청은 빨간색)과 평균 지연 시간을 나타내는 선;
- [[ui:exp.loadLatencies]]: 요청이 얼마나 걸렸는지의 분포;
- 상태와 원인, 각각의 개수.

명령줄도 같은 수치와 각 임계값의 판정을 출력합니다.
[`signallab run`](../automation/cli.md#cli-run)을 참고하십시오.

## 두 실행 비교하기 {#compare}

1. 실험을 두 번 이상 실행합니다.
2. 타임라인에서 [[ui:exp.compare]] 버튼을 누릅니다. 실행이 보고서를 저장한 뒤부터 나타나며,
   실행 중에는 비활성화됩니다.
3. 최신 실행이 [[ui:exp.compareAfter]], 그 직전 실행이 [[ui:exp.compareBefore]] 쪽에 놓이며, 각
   목록에서 다른 실행을 고를 수 있습니다.

목록에는 데이터 폴더의 보고서 중 이 실험(이름 기준)의 실행이 최신순으로 최대 50개 표시되며, 각
실행의 날짜와 시각, 끝난 방식, 시드가 함께 나옵니다. 명령줄이 같은 데이터 폴더를 사용했다면
명령줄에서 실행한 것도 들어 있습니다. 실험의 이름을 바꾸면 새 기록이 시작됩니다.

부하 단계마다(노드 기준으로 짝지음) 모든 지표의 [[ui:exp.compareBefore]], [[ui:exp.compareAfter]]
값과 [[ui:exp.compareChange]] 값을 단위와 %로 보여 주는 표가 나옵니다. 나빠지는 방향으로 5% 이상
변한 것(더 느려짐, 오류 증가, 누락된 요청 증가, 속도 저하)은 회귀로 보고 빨간색으로 표시합니다. 0에서
0이 아닌 값으로 바뀐 것도 회귀로 칩니다. 표 아래에는 두 실행에서 각 임계값의 판정이 나옵니다. 한
실행에만 있는 부하 단계는 [[ui:exp.compareOnlyBefore]] 또는 [[ui:exp.compareOnlyAfter]]로 표시되며
변화는 표시되지 않습니다. 부하 단계가 없는 실행에는 [[ui:exp.compareNoLoad]] 문구가 표시됩니다.

스크립트에서는 [`experiment_runs`](../api/commands.md#experiment_runs)가 실행 목록을 보여 주고,
[`experiment_compare`](../api/commands.md#experiment_compare)가 보고서 파일 이름으로 두 실행을
비교합니다. `signallab mcp`는 어시스턴트에게 같은 기능을 제공합니다([MCP](../automation/mcp.md)).

## 부하 뒤의 검사 {#checks-after}

부하는 자체 응답을 남기지 않습니다. 측정할 뿐 검사하지는 않습니다. 부하 뒤의 검사나
[[ui:exp.node.extract]] 노드는 모든 경로에서 그 앞에 부하 없는 다른 요청이 있어야 하며, 그렇지 않으면
실험이 실행되지 않습니다(`graph.needs_http`). 부하 상태의 API가 주는 응답 하나를 검사하려면 부하 뒤에,
또는 그 옆의 병렬 분기에 일반 [[ui:exp.node.http]] 노드를 두십시오.

부하가 설정된 노드에서 [[ui:exp.sendNow]] 버튼을 누르면 요청을 한 번 보냅니다.
