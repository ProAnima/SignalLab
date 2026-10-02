<#
  Adds Signal Lab's folder to PATH, or takes it off again — for the setup
  (src-tauri/installer/hooks.nsh), so `signallab` works in every new terminal.

    path.ps1 -Add    "C:\Program Files\Signal Lab"
    path.ps1 -Remove "C:\Program Files\Signal Lab"

  The scope follows the folder: one under %LOCALAPPDATA% (a setup for one user)
  goes on that user's PATH, any other on the machine's (needs administrator
  rights, which that setup has). PATH is read and written as it is stored —
  REG_EXPAND_SZ, %VARIABLES% unexpanded — so nothing else on it changes, and
  every program is told that the environment changed.
  -Key names another key under HKEY_CURRENT_USER to work on, for tests.
#>
param(
  [string]$Add,
  [string]$Remove,
  [string]$Key
)
$ErrorActionPreference = 'Stop'

function Open-Environment([string]$folder) {
  if ($Key) { return [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey($Key) }
  $user = $folder.StartsWith($env:LOCALAPPDATA, [StringComparison]::OrdinalIgnoreCase)
  if ($user) { return [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true) }
  return [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey('SYSTEM\CurrentControlSet\Control\Session Manager\Environment', $true)
}

function Same([string]$a, [string]$b) {
  return [Environment]::ExpandEnvironmentVariables($a).TrimEnd('\') -ieq [Environment]::ExpandEnvironmentVariables($b).TrimEnd('\')
}

$folder = if ($Add) { $Add } else { $Remove }
if (-not $folder) { throw 'give -Add or -Remove with a folder' }
$folder = $folder.TrimEnd('\')
$environment = Open-Environment $folder
$path = [string]$environment.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
$parts = @($path -split ';' | Where-Object { $_ -ne '' })

if ($Add) {
  if ($parts | Where-Object { Same $_ $folder }) { exit 0 }
  $parts += $folder
} else {
  $kept = @($parts | Where-Object { -not (Same $_ $folder) })
  if ($kept.Count -eq $parts.Count) { exit 0 }
  $parts = $kept
}
$environment.SetValue('Path', ($parts -join ';'), [Microsoft.Win32.RegistryValueKind]::ExpandString)
$environment.Close()

if (-not $Key) {
  # Tell running programs (Explorer, so new terminals) that the environment changed.
  Add-Type -Namespace SignalLab -Name Native -MemberDefinition '[DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr SendMessageTimeout(IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam, uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);'
  $result = [UIntPtr]::Zero
  [void][SignalLab.Native]::SendMessageTimeout([IntPtr]0xffff, 0x1A, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result)
}
