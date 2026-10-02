; Signal Lab's additions to the Windows setup (Tauri's NSIS hooks):
;
;   - signallab.exe, the command line (docs/automation.md), next to the app,
;     and that folder on PATH — the user's for a setup "for me", the machine's
;     for "for everyone" — so `signallab` works in every new terminal;
;   - when the setup has administrator rights ("for everyone"), an inbound
;     firewall rule for each of the two programs on private and domain
;     networks, so a monitor or a wait hears other machines without the
;     system's prompt. A setup for one user cannot change the firewall; the app
;     offers it, with the system's own administrator prompt, when it first
;     listens (src/components/FirewallBanner.tsx);
;   - the uninstaller takes all of it away again.
;
; On the setup's command line, /NOPATH leaves PATH alone and /NOFIREWALL the firewall.

!include FileFunc.nsh
!include LogicLib.nsh

; This file's folder, captured here: inside a macro ${__FILEDIR__} would be installer.nsi's.
!define SIGNALLAB_INSTALLER_DIR "${__FILEDIR__}"

!macro SIGNALLAB_PATH_SCRIPT
  InitPluginsDir
  File "/oname=$PLUGINSDIR\signallab-path.ps1" "${SIGNALLAB_INSTALLER_DIR}\path.ps1"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Built by scripts/cli-bundle.mjs (the bundle's beforeBundleCommand).
  File "/oname=$INSTDIR\signallab.exe" "${SIGNALLAB_INSTALLER_DIR}\..\..\target\release\signallab.exe"

  ClearErrors
  ${GetOptions} $CMDLINE "/NOPATH" $0
  ${If} ${Errors}
    !insertmacro SIGNALLAB_PATH_SCRIPT
    nsExec::ExecToLog 'powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\signallab-path.ps1" -Add "$INSTDIR"'
    Pop $0
  ${EndIf}

  ClearErrors
  ${GetOptions} $CMDLINE "/NOFIREWALL" $0
  ${If} ${Errors}
    ; Without administrator rights netsh refuses, and nothing changes.
    nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="Signal Lab"'
    Pop $0
    nsExec::ExecToLog 'netsh advfirewall firewall add rule name="Signal Lab" dir=in action=allow program="$INSTDIR\signal-lab.exe" enable=yes profile=private,domain'
    Pop $0
    nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="Signal Lab (command line)"'
    Pop $0
    nsExec::ExecToLog 'netsh advfirewall firewall add rule name="Signal Lab (command line)" dir=in action=allow program="$INSTDIR\signallab.exe" enable=yes profile=private,domain'
    Pop $0
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro SIGNALLAB_PATH_SCRIPT
  nsExec::ExecToLog 'powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\signallab-path.ps1" -Remove "$INSTDIR"'
  Pop $0
  ; Every inbound rule for the two programs: ours, and the ones the system's prompt made.
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name=all program="$INSTDIR\signal-lab.exe"'
  Pop $0
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name=all program="$INSTDIR\signallab.exe"'
  Pop $0
  Delete "$INSTDIR\signallab.exe"
!macroend
