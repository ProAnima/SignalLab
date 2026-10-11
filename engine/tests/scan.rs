//! The port scanner against listeners of its own on loopback: every open port
//! is found — by address and by host name — and only those.

use std::collections::BTreeSet;
use std::time::Duration;

use signal_lab_engine::host::Recorder;
use signal_lab_engine::scan::{start_scan, ScanConfig};
use signal_lab_engine::{Capture, Host, JobRegistry};
use tokio::net::TcpListener;

/// Listeners on as many ports of `from..from+count` as are free, each taking
/// and closing what connects, as a service would; the ports taken. They stop
/// with the returned handles.
async fn listeners(from: u16, count: u16) -> (Vec<AbortOnDrop>, BTreeSet<u16>) {
    let mut held = Vec::new();
    let mut ports = BTreeSet::new();
    for port in from..from + count {
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)).await {
            ports.insert(port);
            held.push(AbortOnDrop(tokio::spawn(async move {
                while let Ok((stream, _)) = listener.accept().await {
                    drop(stream);
                }
            })));
        }
    }
    (held, ports)
}

struct AbortOnDrop(tokio::task::JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Scans `host` over `from..=to`; the open ports it reported and its last progress.
/// The ports are below 32 768, outside the range Linux picks outgoing ports from
/// (Windows picks above 49 152), and the timeout is generous: a loaded test
/// machine may take a while to answer, and only what is missed is in question.
async fn scan(host_name: &str, from: u16, to: u16) -> (BTreeSet<u16>, serde_json::Value) {
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    let config = ScanConfig { host: host_name.into(), port_start: from, port_end: to, concurrency: 400, timeout_ms: 2_000, grab_banner: false };
    let job = start_scan(host, JobRegistry::new(), config).await.unwrap();
    let waiting = recorder.clone();
    tokio::task::spawn_blocking(move || waiting.wait_for("job://ended", Duration::from_secs(60), |ended| ended["job_id"] == job.id))
        .await
        .unwrap()
        .expect("the scan ends");
    let open = recorder.payloads("scan://open").iter().map(|found| found["port"].as_u64().unwrap() as u16).collect();
    let last = recorder.payloads("scan://progress").last().cloned().expect("progress");
    (open, last)
}

#[tokio::test(flavor = "multi_thread")]
async fn every_listener_is_found_and_counted() {
    let (_held, ports) = listeners(21_000, 300).await;
    assert!(ports.len() > 250, "enough free ports to scan");
    let (open, last) = scan("127.0.0.1", 21_000, 21_399).await;
    assert!(ports.is_subset(&open), "missed: {:?}", ports.difference(&open).collect::<Vec<_>>());
    assert!(open.iter().all(|port| (21_000..=21_399).contains(port)));
    assert_eq!(last["done"], 400);
    assert_eq!(last["open"].as_u64().unwrap() as usize, open.len());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_host_name_finds_what_its_address_finds() {
    let (_held, ports) = listeners(22_000, 100).await;
    let (open, _) = scan("localhost", 22_000, 22_099).await;
    assert!(ports.is_subset(&open), "missed by name: {:?}", ports.difference(&open).collect::<Vec<_>>());
}

#[tokio::test]
async fn a_name_that_does_not_resolve_is_the_scans_error() {
    let host = Host::new(Recorder::new(), Capture::new());
    let config = ScanConfig { host: "no-such-host.invalid".into(), port_start: 1, port_end: 10, concurrency: 4, timeout_ms: 100, grab_banner: false };
    let Err(error) = start_scan(host, JobRegistry::new(), config).await else { panic!("no job for a name that is not there") };
    assert!(error.is("transport.dns"), "{error:?}");
}

/// More connections than this machine has ports to make them from (16 384 on
/// Windows): if each one held its port for minutes after the scan let it go,
/// the later scans would find nothing and call it closed.
#[tokio::test(flavor = "multi_thread")]
async fn scanning_many_open_ports_again_and_again_misses_none() {
    let (_held, ports) = listeners(23_000, 2_500).await;
    assert!(ports.len() > 2_000, "enough free ports to scan");
    for round in 0..8 {
        let (open, last) = scan("127.0.0.1", 23_000, 25_499).await;
        assert!(ports.is_subset(&open), "round {round} missed {} ports", ports.difference(&open).count());
        assert_eq!(last["unchecked"], 0, "round {round}");
    }
}
