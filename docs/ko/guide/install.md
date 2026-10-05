---
title: 설치와 업데이트
description: Windows나 Linux에 Signal Lab 데스크톱 앱을 설치하고, 다운로드한 파일을 확인하고, 최신 상태로 유지하고, 다시 제거하는 방법을 설명합니다.
---

# 설치와 업데이트

Signal Lab은 GitHub에서 릴리스됩니다. 모든 릴리스에는 Windows와 Linux용 데스크톱 앱, 단독으로
쓰는 `signallab` 명령줄, 체크섬 목록이 들어 있습니다. 서버에서 실행하고 브라우저로 사용하려면
[서버](../server/index.md)를 참고하십시오.

## 무엇을 다운로드할까 {#downloads}

[최신 릴리스](https://github.com/ProAnima/SignalLab/releases/latest)를 열고 사용하는 시스템에 맞는
파일을 고르십시오. `<version>`은 `[[version]]`과 같은 릴리스 버전입니다.

| 파일 | 용도 |
| --- | --- |
| `Signal.Lab_<version>_x64-setup.exe` | Windows 10 및 11, x64 — 대부분의 사용자에게 맞는 설치 프로그램 |
| `Signal.Lab_<version>_x64_en-US.msi` | Windows, IT 부서의 배포용(Intune, 그룹 정책): 모든 사용자용으로 설치 |
| `Signal.Lab_<version>_amd64.deb` | Linux x86_64: Debian, Ubuntu 및 그 계열 |
| `Signal.Lab-<version>-1.x86_64.rpm` | Linux x86_64: Fedora, RHEL, openSUSE 및 그 계열 |
| `Signal.Lab_<version>_amd64.AppImage` | Linux x86_64, 모든 배포판, 설치 없이 실행 |
| `signallab-<version>-windows-x64.zip` | Windows용 명령줄 단독 |
| `signallab-<version>-linux-x64.tar.gz` | Linux용 명령줄 단독 |
| `SHA256SUMS.txt` | 위의 모든 파일의 SHA-256 체크섬 |

옆에 있는 `.sig` 파일과 `latest.json`은 앱 자체의 업데이트 기능이 사용하는 파일이므로 받을
필요가 없습니다.

## Windows {#windows}

Signal Lab은 Windows 10 및 11(x64)에서 실행됩니다. 두 버전 모두에 포함된 WebView2 런타임을
사용합니다.

### 설치 프로그램으로 설치하기 {#windows-setup}

1. `Signal.Lab_<version>_x64-setup.exe`를 실행합니다.
2. Windows SmartScreen이 PC를 보호했다고 표시하면 **추가 정보**를 선택한 다음
   **실행**을 선택합니다(아래 [SmartScreen](#smartscreen) 참고).
3. 설치 프로그램의 언어를 고릅니다. 앱과 같은 11개 언어를 제공합니다.
4. 누구를 위해 설치할지 선택합니다.
   - 나만 사용: `%LOCALAPPDATA%\Signal Lab`에 설치되며 관리자 권한이 필요하지
     않습니다.
   - 이 컴퓨터를 사용하는 모든 사람: `C:\Program Files\Signal Lab`에 설치되며
     관리자 권한을 요청합니다.
5. 폴더를 확인하고 설치합니다.
6. 마지막 페이지에서 Signal Lab을 바로 시작하고 바탕 화면 바로 가기를 만들 수 있습니다.

이미 설치된 버전 위에 새 버전의 설치 프로그램을 실행하면 그 자리에서 업그레이드됩니다.

### 설치 프로그램이 추가하는 것 {#windows-setup-adds}

설치 프로그램은 앱 외에 두 가지를 설치하며, 제거 프로그램은 둘 다 제거합니다.

- **명령줄.** `signallab.exe`가 앱 옆에 설치되고 그 폴더가 `PATH`에 추가됩니다.
  나만 사용하는 설치는 사용자 본인의 `PATH`에, 모든 사람용 설치는 컴퓨터 전체 설정에
  추가되므로, 이후에 여는 모든 터미널에서 `signallab`을 사용할 수 있습니다.
  [명령줄](../automation/cli.md)을 참고하십시오.
- **방화벽 규칙**(모든 사람용 설치에만 해당). Windows Defender 방화벽은 규칙이 허용할 때만
  다른 컴퓨터가 프로그램에 접근하게 하며, 프로그램이 처음 수신을 시작할 때 화면 앞의 사용자에게
  묻습니다. 여기서 *취소*를 누르면 다른 컴퓨터가 보내는 데이터가 아무 표시 없이 버려집니다.
  관리자 권한이 있는 모든 사람용 설치는 개인 네트워크와 도메인 네트워크에서
  `signal-lab.exe`(앱)와 `signallab.exe`(명령줄)에 대한 인바운드 허용 규칙을 하나씩 추가합니다.
  나만 사용하는 설치는 방화벽을 변경할 수 없습니다. 대신 무언가가 처음 수신을 시작할 때 앱이
  Windows 자체의 관리자 권한 요청 창으로 규칙 추가를 제안합니다.
  [방화벽 알림](interface.md#firewall-notice)을 참고하십시오.

이 컴퓨터 안의 트래픽(`127.0.0.1`)은 절대 필터링되지 않으므로, 루프백에서 하는 작업은
규칙 없이도 모두 동작합니다.

### 무인 설치 {#windows-unattended}

설치 프로그램은 명령줄에서 다음 스위치를 받습니다.

| 스위치 | 동작 |
| --- | --- |
| `/S` | 아무것도 묻지 않고 자동으로 설치합니다. |
| `/P` | 진행률 표시줄만 보여 주고 질문 없이 설치합니다. |
| `/ALLUSERS` | 모든 사람용으로 설치합니다(관리자 권한 프롬프트 필요). |
| `/CURRENTUSER` | 현재 사용자용으로만 설치합니다. |
| `/NS` | 바로 가기를 만들지 않습니다. |
| `/D=C:\Tools\Signal Lab` | 이 폴더에 설치합니다. 맨 마지막에 와야 하며 따옴표로 묶지 않습니다. |
| `/NOPATH` | `PATH`를 건드리지 않습니다. `signallab`은 설치되지만 `PATH`에는 없습니다. |
| `/NOFIREWALL` | 방화벽 규칙을 추가하지 않습니다. |

예를 들어, 관리자 권한 프롬프트에서 방화벽 규칙 없이 모든 사람용으로 자동 설치하려면 다음과
같이 합니다.

```powershell
.\Signal.Lab_[[version]]_x64-setup.exe /S /ALLUSERS /NOFIREWALL
```

### MSI {#windows-msi}

`Signal.Lab_<version>_x64_en-US.msi`는 IT 부서의 배포용입니다. 모든 사용자용으로 설치되고
`signallab.exe`를 앱 옆에 두며 그 폴더를 컴퓨터의 `PATH`에 추가하지만, 방화벽 규칙은 추가하지
않습니다.
이는 배포하는 쪽(예: 그룹 정책)에 맡깁니다. 앱은 처음 수신을 시작할 때 여전히 규칙 추가를
제안합니다. 자동으로 설치하려면 다음과 같이 합니다.

```powershell
msiexec /i "Signal.Lab_[[version]]_x64_en-US.msi" /qn
```

### SmartScreen {#smartscreen}

설치 프로그램은 아직 코드 서명이 되어 있지 않으므로, Windows SmartScreen이 게시자를 알지 못해
경고와 함께 설치를 막을 수 있습니다. **추가 정보**를 선택하고, 파일이 릴리스 페이지에서 받은
바로 그 파일인지 확인한 다음 **실행**을 선택하십시오. 파일이 변경되지 않았음을 확실히 하려면
먼저 [`SHA256SUMS.txt`와 대조해 확인](#checksums)하십시오.

앱 자체의 업데이트는 서명되어 있으며 따로 검사합니다. [업데이트](#updates)를 참고하십시오.

## 다운로드한 파일 확인하기 {#checksums}

`SHA256SUMS.txt`에는 릴리스의 모든 파일에 대한 SHA-256 체크섬이 한 줄에 하나씩, 파일 이름과
함께 나열되어 있습니다. 이 파일을 확인할 파일 옆에 다운로드하고 비교하십시오.

Windows에서는 PowerShell에서 다음을 실행합니다.

```powershell
Get-FileHash .\Signal.Lab_[[version]]_x64-setup.exe -Algorithm SHA256
Select-String "Signal.Lab_[[version]]_x64-setup.exe" .\SHA256SUMS.txt
```

두 해시가 같아야 합니다(`Get-FileHash`는 대문자로 출력하지만 대소문자는 상관없습니다).

Linux에서는 두 파일이 있는 폴더에서 다음을 실행합니다.

```bash
sha256sum --check --ignore-missing SHA256SUMS.txt
```

찾은 파일마다 뒤에 `OK`를 출력합니다. 다른 결과가 나오면 다운로드한 파일이 릴리스된 파일과
다르다는 뜻이므로 다시 다운로드하십시오.

## Linux {#linux}

Linux 앱은 x86_64용으로(Ubuntu 22.04에서) 빌드되며, 창을 그리는 웹 뷰인 WebKitGTK 4.1이
필요합니다. Arm 프로세서용 빌드는 없으므로, Arm 컴퓨터에서는 대신 [서버](../server/index.md)를
실행하십시오.

### .deb 패키지 {#linux-deb}

```bash
sudo apt install ./Signal.Lab_[[version]]_amd64.deb
```

`apt`가 패키지에 필요한 것을 함께 설치합니다. 패키지는 명령줄도 `/usr/bin/signallab`에
설치합니다.

### .rpm 패키지 {#linux-rpm}

```bash
sudo dnf install ./Signal.Lab-[[version]]-1.x86_64.rpm
```

openSUSE에서는 같은 파일로 `sudo zypper install`을 사용하십시오. `.deb`와 마찬가지로 이
패키지도 명령줄을 `/usr/bin/signallab`에 설치합니다.

### AppImage {#linux-appimage}

AppImage는 설치 없이 실행됩니다.

```bash
chmod +x Signal.Lab_[[version]]_amd64.AppImage
./Signal.Lab_[[version]]_amd64.AppImage
```

AppImage에는 `signallab` 명령줄이 포함되어 있지 않습니다. 필요하면
[명령줄 압축 파일](#cli-archives)에서 받으십시오.

### Linux의 방화벽 {#linux-firewall}

Linux에서는 앱이 방화벽 알림을 표시하지 않습니다. `ufw`와 `firewalld`는 프로그램이 아니라
포트 단위로 동작하기 때문입니다. 둘 중 하나가 켜져 있고 다른 컴퓨터가 모니터, 리스너,
에뮬레이터, 릴레이에 접근해야 한다면 거기서 해당 포트를 여십시오. 예를 들면 다음과 같습니다.

```bash
sudo ufw allow 9000/udp
```

루프백 트래픽은 영향을 받지 않습니다.

## 명령줄 단독 설치 {#cli-archives}

데스크톱 설치 프로그램은 `signallab`을 함께 설치합니다. 앱이 없는 컴퓨터(빌드 에이전트, 서버,
랩 PC)를 위해 각 릴리스에는 명령줄만 따로 들어 있는 파일도 있습니다.

- `signallab-<version>-windows-x64.zip`에는 `signallab.exe`와 라이선스가 들어 있습니다.
- `signallab-<version>-linux-x64.tar.gz`에는 `signallab`과 라이선스가 들어 있습니다.

`PATH`에 있는 폴더에 압축을 푸십시오. Linux에서는 다음과 같습니다.

```bash
tar -xzf signallab-[[version]]-linux-x64.tar.gz signallab
sudo install signallab /usr/local/bin/
signallab version
```

서버의 Docker 이미지에도 들어 있습니다. 사용 방법은 [명령줄](../automation/cli.md)을
참고하십시오.

## 업데이트 {#updates}

### 데스크톱 앱 {#updates-desktop}

데스크톱 앱은 게시된 릴리스에서만, 그리고 사용자가 원할 때만 스스로 업데이트합니다.

- **하루에 한 번 확인합니다.** [[ui:update.auto]] 옵션이 선택되어 있으면(기본값) 앱은 마지막
  확인 후 하루 이상 지났을 때 시작 직후 새 릴리스를 찾고, 실행 중에 하루가 더 지날 때마다 다시
  찾습니다. 자동으로 확인하지 않으려면 Signal Lab 정보에서 이 옵션을 해제하십시오.
- **지금 확인할 수도 있습니다.** 헤더의 **?**를 눌러 Signal Lab 정보를 열고
  [[ui:update.title]] 아래의 [[ui:update.check]] 버튼을 누르십시오.
- **확인할 때 보내는 내용.** 스튜디오의 업데이트 서비스(`hub.proanima.net`)에 이 설치가 받아야
  할 릴리스를 묻고, 그 서비스에 연결할 수 없을 때만 GitHub의 최신 릴리스로 갑니다. 요청에는
  앱 버전, 시스템 및 프로세서 종류, 이 설치를 위해 만든 임의의 번호가 담기며, 이 번호 덕분에
  새 릴리스를 일부 설치에 먼저 배포할 수 있습니다. 사용자나 이 컴퓨터를 식별하는 정보는 보내지
  않습니다.
- **릴리스를 찾으면** 콘솔에 알림이 표시되고, 헤더에 Signal Lab 정보를 여는 업데이트 버튼이
  나타납니다. 거기서 [[ui:update.notes]]에 릴리스 노트가 표시됩니다.
- **클릭하기 전에는 아무것도 설치되지 않습니다.** [[ui:update.install]] 버튼을 누르십시오
  (실행 중인 작업이 있으면 버튼에 작업을 중지한다고 표시됩니다). Signal Lab은 아직 저장되지 않은
  모든 것을 저장하고, 실행 중인 모든 작업을 중지하고, 업데이트를 다운로드하고, 앱에 내장된 키로
  서명을 검사한 뒤(릴리스 과정에서 서명되지 않은 업데이트는 거부됩니다) 설치하고 새 버전으로 다시
  시작합니다.

Windows에서는 업데이트가 질문 없이 설치 프로그램을 실행하며, 설치 폴더, `PATH`, 방화벽 규칙을
그대로 유지합니다. Linux에서는 AppImage를 교체하거나 새 `.deb` 또는 `.rpm` 패키지를 설치합니다.
패키지의 경우 시스템이 먼저 암호를 묻습니다.

시험판과 초안은 제공되지 않습니다. 예를 들어 IT 부서가 직접 업데이트를 배포하는 경우처럼
컴퓨터의 모든 설치가 스스로 확인하지 않게 하려면 환경 변수 `SIGNALLAB_NO_UPDATE_CHECK`를 아무
값으로나 설정하십시오. 그래도 [[ui:update.check]] 버튼을 누르면 확인합니다.
개발 빌드는 스스로 확인하지 않습니다.

### 서버 {#updates-server}

서버와 브라우저가 서버에서 받아 표시하는 페이지는 Docker 이미지와 함께 업데이트되며, 스스로
업데이트하지 않습니다. Signal Lab 정보에도 그렇게 표시됩니다. 업데이트 방법은
[서버](../server/index.md)를 참고하십시오.

## 제거하기 {#uninstall}

**Windows.** *설정 → 앱 → 설치된 앱*을 열고 Signal Lab을 찾아 *제거*를 선택하십시오. 설치
프로그램의 제거 프로그램은 `PATH`에서 `signallab.exe`를 빼고, 관리자 권한으로 실행되면(모든
사람용 설치처럼) Windows 프롬프트가 만든 규칙을 포함해 앱과 명령줄에 대한 모든 인바운드 방화벽
규칙을 제거합니다. 애플리케이션 데이터도 삭제할지 묻는 확인란이 있습니다. 이 데이터는 언어, 창
영역 크기, 마지막으로 보던 화면, 화면에 마지막으로 입력한 값 같은 앱 자체의 설정이며, 사용자의
파일이 아닙니다. MSI 설치도 같은 방법으로 제거합니다. MSI는 `PATH` 항목을 함께 제거하고
방화벽은 배포한 쪽에 맡깁니다.

**Linux.** 설치할 때 사용한 패키지 관리자로 패키지를 제거하거나 AppImage 파일을 삭제하십시오.

어느 쪽도 **데이터 폴더**는 건드리지 않습니다. 실험, 신호 및 에뮬레이터 라이브러리, 실행
보고서, 내보낸 파일은 홈 폴더의 `Documents/SignalLab`에 남아 있습니다. 이것들을 없애려면 그
폴더를 직접 삭제하십시오.

## 데이터 저장 위치 {#data}

만든 모든 것은 Windows와 Linux 모두 홈 폴더의 `Documents/SignalLab`이라는 한 폴더에 일반
파일로 보관됩니다. 현재 실험, 신호 라이브러리, 에뮬레이터 라이브러리, 실행 보고서, 내보낸 파일,
인스펙터 캡처가 여기에 있습니다. 각 파일과 그 형식은 [파일과 폴더](../reference/files.md)에
설명되어 있습니다.

## 서버로 실행하기 {#server}

장비 옆의 랩 PC, Linux 장치, Docker에서 Signal Lab을 브라우저로 사용하려면
[서버](../server/index.md)를 참고하십시오. Docker가 있는 Linux 컴퓨터에서는 명령 하나로 설치하고
시작할 수 있습니다.
