//! `signallab doctor` and `signallab firewall allow`: what stands between
//! Signal Lab and the gear — the firewall (per program on Windows, per port on
//! Linux), the network, the data folder, a lab server and its token — and the
//! fix for the firewall, with the system's own administrator prompt.

use std::path::PathBuf;

use serde_json::{json, Value};
use signal_lab_engine::firewall::{self, FirewallStatus};
use signal_lab_engine::{net, paths};

use crate::fail::{Exit, Failure};
use crate::remote::Remote;
use crate::{Ctx, DoctorArgs, FirewallArgs};

/// The programs of an installation the firewall should let through: this one,
/// and the desktop app — next to it, or where the installers put it.
pub fn programs() -> Vec<(String, PathBuf)> {
    let this = firewall::this_program();
    let mut found = vec![("signallab".to_string(), this.clone())];
    let app = if cfg!(windows) { "signal-lab.exe" } else { "signal-lab" };
    let mut places = vec![this.with_file_name(app)];
    if cfg!(windows) {
        for base in ["LOCALAPPDATA", "ProgramFiles"] {
            if let Ok(dir) = std::env::var(base) {
                places.push(PathBuf::from(dir).join("Signal Lab").join(app));
            }
        }
    } else {
        places.push(PathBuf::from("/usr/bin").join(app));
    }
    if let Some(path) = places.into_iter().find(|path| path.is_file() && *path != this) {
        found.push(("app".to_string(), path));
    }
    found
}

/// A Linux firewall that works by port, if one is on: its name and how to open a port.
fn port_firewall() -> Option<(&'static str, &'static str)> {
    let active = |command: &str, args: &[&str], expect: &str| {
        std::process::Command::new(command).args(args).output().ok().is_some_and(|output| String::from_utf8_lossy(&output.stdout).trim() == expect)
    };
    if cfg!(target_os = "linux") {
        if active("systemctl", &["is-active", "ufw"], "active") {
            return Some(("ufw", "sudo ufw allow <port>/udp"));
        }
        if active("firewall-cmd", &["--state"], "running") {
            return Some(("firewalld", "sudo firewall-cmd --permanent --add-port=<port>/udp && sudo firewall-cmd --reload"));
        }
    }
    None
}

fn status_line(status: &FirewallStatus) -> &'static str {
    match (status.enabled, status.blocked, status.allowed) {
        (false, _, _) => "the firewall is off on this network",
        (true, true, _) => "BLOCKED by a rule (a Cancel at the system's prompt) — signallab firewall allow",
        (true, false, true) => "allowed",
        (true, false, false) => "not allowed yet — signallab firewall allow",
    }
}

pub async fn doctor(ctx: &Ctx, args: DoctorArgs) -> Exit {
    let mut problems = 0;
    let mut report = json!({ "version": env!("CARGO_PKG_VERSION") });
    let say = |line: String| ctx.say(&line);
    say(format!("signallab {}", env!("CARGO_PKG_VERSION")));

    // The network this machine is on.
    let host = net::host_info().await;
    say(format!("Network: {} · {}", host.hostname, host.local_ip));
    report["network"] = json!({ "hostname": host.hostname, "address": host.local_ip });

    // The data folder: the app's documents, where fire and the app keep their files. Looked at, never made here.
    let data = paths::data_dir();
    let (writable, state) = if data.is_dir() {
        let writable = tempfile_in(&data);
        (writable, if writable { "writable" } else { "NOT writable" })
    } else {
        (true, "not there yet — the app makes it on first use")
    };
    say(format!("Data folder: {} — {state}", data.display()));
    report["data_dir"] = json!({ "path": data.display().to_string(), "writable": writable, "exists": data.is_dir() });
    if !writable {
        problems += 1;
    }

    // The firewall.
    let mut firewalls = Vec::new();
    for (name, program) in programs() {
        match firewall::status(vec![program.clone()]).await {
            Ok(status) if status.applies => {
                let line = status_line(&status);
                if status.in_the_way() {
                    problems += 1;
                }
                say(format!("Firewall · {name} ({}) · {} network: {line}", program.display(), status.networks.join(", ")));
                firewalls.push(json!({ "program": name, "status": status }));
            }
            Ok(_) => {}
            Err(error) => {
                let failure = Failure::environment(error);
                say(format!("Firewall · {name}: {}", failure.lines(ctx, "").join(" ")));
            }
        }
    }
    if firewalls.is_empty() {
        match port_firewall() {
            Some((name, how)) => {
                say(format!("Firewall: {name} is on — it lets in only the ports it lists: {how} (and the server's port, e.g. 1430/tcp)"));
                firewalls.push(json!({ "system": name, "open_with": how }));
            }
            None => say("Firewall: no per-program firewall here, and neither ufw nor firewalld is on".into()),
        }
    }
    report["firewall"] = Value::Array(firewalls);

    // A lab server, when one is named.
    if let Some(url) = &args.server {
        match Remote::new(url, args.token_file.as_deref()) {
            Ok(remote) => match remote.invoke("app_info", &Value::Null).await {
                Ok(info) => {
                    say(format!("Server {url}: answers · Signal Lab {} · token accepted", info["version"].as_str().unwrap_or("?")));
                    report["server"] = json!({ "url": url, "ok": true, "info": info });
                }
                Err(failure) => {
                    problems += 1;
                    say(format!("Server {url}: {}", failure.lines(ctx, "").join(" ")));
                    report["server"] = json!({ "url": url, "ok": false, "error": failure.error });
                }
            },
            Err(failure) => {
                problems += 1;
                say(format!("Server {url}: {}", failure.lines(ctx, "").join(" ")));
            }
        }
    }

    report["problems"] = problems.into();
    if ctx.json {
        println!("{report}");
    } else {
        println!("{}", if problems == 0 { "✔ nothing in the way".to_string() } else { format!("✖ {problems} thing(s) in the way") });
    }
    if problems == 0 { Exit::Passed } else { Exit::Failed }
}

/// The folder takes a file (and lets it go again).
fn tempfile_in(dir: &std::path::Path) -> bool {
    let probe = dir.join(format!(".signallab-doctor-{}", std::process::id()));
    let ok = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

pub async fn firewall(ctx: &Ctx, args: FirewallArgs) -> Exit {
    let programs: Vec<PathBuf> = programs().into_iter().map(|(_, path)| path).collect();
    if !cfg!(windows) {
        match port_firewall() {
            Some((name, how)) => ctx.say(&format!("{name} lets in only the ports it lists; open the ones you listen on: {how}")),
            None => ctx.say("No firewall here filters by program, and neither ufw nor firewalld is on: nothing to change."),
        }
        return Exit::Passed;
    }
    ctx.say("Windows asks for administrator rights now…");
    match firewall::allow(programs.clone(), args.public).await {
        Ok(()) => {
            for program in programs {
                if let Ok(status) = firewall::status(vec![program.clone()]).await {
                    ctx.say(&format!("{}: {}", program.display(), status_line(&status)));
                }
            }
            Exit::Passed
        }
        Err(error) => Failure::environment(error).report(ctx, "✖ "),
    }
}
