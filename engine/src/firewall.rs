//! Whether the system's firewall lets other machines reach Signal Lab, and
//! letting it do so when a person asks.
//!
//! Windows Defender Firewall decides per program: the first time a program
//! listens it asks the person at the screen, and a "Cancel" leaves a block
//! rule that silently drops everything — the classic "the monitor shows
//! nothing". `status` reads the rules for a program (through the firewall's
//! own COM interface, so it does not depend on the system's language) and the
//! profile of the network the machine is on; `allow` replaces that program's
//! inbound rules with one allow rule, after the system asks for administrator
//! rights (UAC) — never silently. Elsewhere there is nothing per program to
//! read (`applies: false`): ufw and firewalld work by port, which the server's
//! install script and docs/delivery.md cover.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::error::{EngineError, EngineResult};

/// What the firewall does with what other machines send to a program.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirewallStatus {
    /// There is a per-program firewall to read here (Windows).
    pub applies: bool,
    /// The program the rules are about.
    pub program: String,
    /// The firewall is on for the network the machine is on now.
    pub enabled: bool,
    /// The kinds of network the machine is on: "domain", "private", "public".
    pub networks: Vec<String>,
    /// An inbound rule lets UDP in for this program on the current network.
    pub allowed: bool,
    /// An inbound rule blocks this program on the current network (a "Cancel" at the prompt) — it wins over an allow.
    pub blocked: bool,
    /// Inbound rules for this program, of any kind.
    pub rules: usize,
}

impl FirewallStatus {
    /// Other machines' datagrams may not arrive: the firewall is on and nothing lets them in, or something blocks them.
    pub fn in_the_way(&self) -> bool {
        self.applies && self.enabled && (self.blocked || !self.allowed)
    }
}

/// The program that runs now.
pub fn this_program() -> PathBuf {
    std::env::current_exe().unwrap_or_default()
}

/// One inbound rule, as the firewall reports it.
#[derive(Debug, Deserialize)]
struct Rule {
    enabled: bool,
    allow: bool,
    /// Bitmask: 1 domain, 2 private, 4 public.
    profiles: i64,
    /// 6 TCP, 17 UDP, 256 any.
    protocol: i64,
}

#[derive(Debug, Deserialize)]
struct Report {
    current: i64,
    #[serde(default)]
    enabled: Vec<i64>,
    #[serde(default)]
    rules: Vec<Rule>,
}

const PROFILES: [(i64, &str); 3] = [(1, "domain"), (2, "private"), (4, "public")];

/// The status from what the firewall reported: the active profiles, and the rules for the program.
fn judge(program: &Path, report: Report) -> FirewallStatus {
    let current = report.current;
    let covers = |rule: &Rule| rule.enabled && rule.profiles & current != 0 && matches!(rule.protocol, 17 | 256);
    FirewallStatus {
        applies: true,
        program: program.display().to_string(),
        enabled: !report.enabled.is_empty(),
        networks: PROFILES.iter().filter(|(bit, _)| current & bit != 0).map(|(_, name)| name.to_string()).collect(),
        allowed: report.rules.iter().any(|rule| covers(rule) && rule.allow),
        blocked: report.rules.iter().any(|rule| covers(rule) && !rule.allow),
        rules: report.rules.len(),
    }
}

/// A path as PowerShell reads it inside single quotes.
#[cfg(windows)]
fn quoted(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "''"))
}

#[cfg(windows)]
fn powershell(script: &str) -> std::io::Result<std::process::Output> {
    use std::os::windows::process::CommandExt;
    // No console window flashing over the app.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
}

/// The firewall's view of `programs` (the first one is reported): Windows only.
pub async fn status(programs: Vec<PathBuf>) -> EngineResult<FirewallStatus> {
    let program = programs.first().cloned().unwrap_or_else(this_program);
    #[cfg(windows)]
    {
        let list = programs.iter().map(|path| quoted(path)).collect::<Vec<_>>().join(",");
        let script = format!(
            "$ErrorActionPreference='Stop'; $fw = New-Object -ComObject HNetCfg.FwPolicy2; $current = $fw.CurrentProfileTypes; \
             $programs = @({list}); \
             $enabled = @(@(1,2,4) | Where-Object {{ ($current -band $_) -and $fw.FirewallEnabled($_) }}); \
             $rules = @(foreach ($r in $fw.Rules) {{ if ($r.Direction -eq 1 -and $r.ApplicationName -and ($programs -contains [Environment]::ExpandEnvironmentVariables($r.ApplicationName))) \
               {{ [pscustomobject]@{{ enabled = [bool]$r.Enabled; allow = ($r.Action -eq 1); profiles = [int64]$r.Profiles; protocol = [int64]$r.Protocol }} }} }}); \
             [pscustomobject]@{{ current = [int64]$current; enabled = $enabled; rules = $rules }} | ConvertTo-Json -Depth 4 -Compress"
        );
        let output = tokio::task::spawn_blocking(move || powershell(&script))
            .await
            .map_err(|error| EngineError::new("firewall.failed").because(error))?
            .map_err(|error| EngineError::new("firewall.failed").because(error))?;
        if !output.status.success() {
            return Err(EngineError::new("firewall.failed").because(String::from_utf8_lossy(&output.stderr).trim()));
        }
        let report: Report = serde_json::from_slice(&output.stdout).map_err(|error| EngineError::new("firewall.failed").because(error))?;
        Ok(judge(&program, report))
    }
    #[cfg(not(windows))]
    {
        let _ = programs;
        Ok(FirewallStatus { program: program.display().to_string(), ..FirewallStatus::default() })
    }
}

/// Let other machines reach `programs`: their inbound rules (a "Cancel" at the
/// prompt included) are replaced by one allow rule each, on private and domain
/// networks — and public ones when `public`. Windows asks for administrator
/// rights first; a "No" there is `firewall.declined`.
pub async fn allow(programs: Vec<PathBuf>, public: bool) -> EngineResult<()> {
    #[cfg(windows)]
    {
        let profiles = if public { "private,domain,public" } else { "private,domain" };
        let mut commands = Vec::new();
        for program in &programs {
            let path = program.display().to_string();
            if path.contains('"') {
                return Err(EngineError::new("firewall.failed").with("program", &path));
            }
            let name = match program.file_stem().and_then(|stem| stem.to_str()) {
                Some("signallab") => "Signal Lab (command line)",
                Some("signal-lab-server") => "Signal Lab server",
                _ => "Signal Lab",
            };
            commands.push(format!("netsh advfirewall firewall delete rule name=all dir=in program=\"{path}\""));
            commands.push(format!("netsh advfirewall firewall add rule name=\"{name}\" dir=in action=allow program=\"{path}\" enable=yes profile={profiles}"));
        }
        let joined = commands.join(" & ").replace('\'', "''");
        // cmd runs the netsh lines elevated; the UAC prompt is the system's own.
        let script = format!(
            "$ErrorActionPreference='Stop'; try {{ $p = Start-Process -FilePath cmd.exe -ArgumentList '/c {joined}' -Verb RunAs -Wait -PassThru -WindowStyle Hidden; exit $p.ExitCode }} \
             catch {{ $e = $_.Exception; while ($e) {{ if ($e.NativeErrorCode -eq 1223) {{ exit 1223 }}; $e = $e.InnerException }}; [Console]::Error.WriteLine($_.Exception.Message); exit 2 }}"
        );
        let output = tokio::task::spawn_blocking(move || powershell(&script))
            .await
            .map_err(|error| EngineError::new("firewall.failed").because(error))?
            .map_err(|error| EngineError::new("firewall.failed").because(error))?;
        match output.status.code() {
            Some(0) => Ok(()),
            Some(1223) => Err(EngineError::new("firewall.declined")),
            _ => Err(EngineError::new("firewall.failed").because(String::from_utf8_lossy(&output.stderr).trim())),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (programs, public);
        Err(EngineError::new("firewall.unsupported"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(enabled: bool, allow: bool, profiles: i64, protocol: i64) -> Rule {
        Rule { enabled, allow, profiles, protocol }
    }

    #[test]
    fn the_rules_are_read_for_the_network_the_machine_is_on() {
        let program = Path::new(r"C:\Program Files\Signal Lab\signal-lab.exe");
        // On a private network, with the two rules the Windows prompt makes for "Allow".
        let private = judge(program, Report { current: 2, enabled: vec![2], rules: vec![rule(true, true, 2, 6), rule(true, true, 2, 17)] });
        assert_eq!((private.allowed, private.blocked, private.in_the_way()), (true, false, false));
        assert_eq!(private.networks, ["private"]);
        // The same rules on a public network (a venue's Wi-Fi) do not apply.
        let public = judge(program, Report { current: 4, enabled: vec![4], rules: vec![rule(true, true, 2, 17)] });
        assert!(!public.allowed && public.in_the_way());
        // "Cancel" at the prompt: a block rule, which wins.
        let cancelled = judge(program, Report { current: 2, enabled: vec![2], rules: vec![rule(true, true, 2, 17), rule(true, false, 0x7fff_ffff, 256)] });
        assert!(cancelled.blocked && cancelled.in_the_way());
        // TCP only, or a disabled rule, does not let datagrams in.
        assert!(!judge(program, Report { current: 2, enabled: vec![2], rules: vec![rule(true, true, 2, 6), rule(false, true, 2, 17)] }).allowed);
        // The firewall off: nothing is in the way, whatever the rules.
        let off = judge(program, Report { current: 2, enabled: vec![], rules: vec![] });
        assert!(!off.enabled && !off.in_the_way());
        assert!(!FirewallStatus::default().in_the_way(), "where it does not apply, it is never in the way");
    }

    #[tokio::test]
    async fn the_status_is_read_without_rights_or_side_effects() {
        let status = status(vec![this_program()]).await.unwrap();
        assert_eq!(status.applies, cfg!(windows));
        assert_eq!(status.program, this_program().display().to_string());
        assert!(status.networks.iter().all(|network| ["domain", "private", "public"].contains(&network.as_str())), "{status:?}");
    }
}
