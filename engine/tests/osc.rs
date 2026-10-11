//! The OSC monitor and generator at their limits, on loopback: a flood reaches
//! the interface as a bounded sample with every packet counted, hostile
//! datagrams do not end the monitor, the generator keeps the rate it was given.

use std::time::{Duration, Instant};

use signal_lab_engine::host::Recorder;
use signal_lab_engine::osc::{start_generator, start_monitor, GenConfig, Waveform};
use signal_lab_engine::osc_codec::{encode_message, OscArg};
use signal_lab_engine::{Capture, Host, JobRegistry};

fn recorder_host() -> (std::sync::Arc<Recorder>, Host) {
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    (recorder, host)
}

/// 100 000 packets as fast as loopback takes them: what the interface is sent.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_flood_reaches_the_interface_as_a_sample_with_everything_counted() {
    let (recorder, host) = recorder_host();
    let jobs = JobRegistry::new();
    let job = start_monitor(host, jobs.clone(), "127.0.0.1:0".into()).await.unwrap();
    let target = job.params["bind"].clone();
    let sent = tokio::task::spawn_blocking(move || {
        let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let packet = encode_message("/flood", &[OscArg::Int(1), OscArg::Float(0.5)]);
        let started = Instant::now();
        let mut sent = 0u64;
        while sent < 100_000 {
            if socket.send_to(&packet, target.as_str()).is_ok() {
                sent += 1;
            }
        }
        (sent, started.elapsed())
    })
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let messages = recorder.payloads("osc://message");
    let stats = recorder.payloads("osc://monitor-stat");
    eprintln!("sent {} in {:?}; osc://message events: {}; last stat: {:?}", sent.0, sent.1, messages.len(), stats.last());
    jobs.stop(job.id);
    let last = stats.last().expect("the monitor reports its totals");
    let received = last["packets"].as_u64().unwrap();
    assert!(received > 10_000, "{last}");
    assert_eq!(received, messages.len() as u64 + last["not_shown"].as_u64().unwrap(), "every packet is shown or counted as not shown: {last}");
    // Two seconds of flood at most: a few hundred events, not one per packet.
    assert!(messages.len() <= 3 * 200 + 10, "{} events", messages.len());
}

/// Datagrams no OSC sender should make: each is a decode error on screen, the
/// monitor keeps listening.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hostile_datagrams_do_not_end_the_monitor() {
    let (recorder, host) = recorder_host();
    let jobs = JobRegistry::new();
    let job = start_monitor(host, jobs.clone(), "127.0.0.1:0".into()).await.unwrap();
    let target = job.params["bind"].clone();
    let mut nested = encode_message("/deep", &[]);
    while nested.len() + 20 <= 65_507 {
        let mut outer = b"#bundle\0\0\0\0\0\0\0\0\x01".to_vec();
        outer.extend_from_slice(&(nested.len() as i32).to_be_bytes());
        outer.extend_from_slice(&nested);
        nested = outer;
    }
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    for packet in [&b"/x\0"[..], b"/x\0\0,s", b"/x\0\0,b\0\0\xff\xff\xff\xff", &nested, &vec![b'/'; 65_507]] {
        socket.send_to(packet, target.as_str()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    // A monitor that is still listening forwards this; loopback may drop a
    // datagram after the big ones, so it is sent until it arrives or time runs out.
    let alive = loop {
        socket.send_to(&encode_message("/still/here", &[]), target.as_str()).await.unwrap();
        let waiting = recorder.clone();
        if let Some(found) = tokio::task::spawn_blocking(move || {
            waiting.wait_for("osc://message", Duration::from_millis(400), |message| message["messages"][0]["address"] == "/still/here")
        })
        .await
        .unwrap()
        {
            break Some(found);
        }
        if recorder.payloads("osc://monitor-stat").is_empty() {
            break None;
        }
    };
    assert!(alive.is_some(), "the monitor stopped: {:?}", recorder.events());
    let errors = recorder.payloads("osc://message").iter().filter(|message| message["error"]["code"] == "osc.packet_malformed").count();
    assert!((4..=5).contains(&errors), "malformed forwarded: {errors}");
    jobs.stop(job.id);
}

/// The generator at its top rate for a second: what reached the device.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_generator_keeps_its_top_rate() {
    let (recorder, host) = recorder_host();
    let device = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    device.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
    let jobs = JobRegistry::new();
    let cfg = GenConfig { target: device.local_addr().unwrap().to_string(), address: "/lfo".into(), rate: 5_000.0, waveform: Waveform::Sine, freq: 1.0, min: 0.0, max: 1.0, as_int: false, duration_s: 1.0 };
    start_generator(host, jobs, cfg).await.unwrap();
    let received = tokio::task::spawn_blocking(move || {
        let mut buf = [0u8; 64];
        let mut n = 0u64;
        while device.recv_from(&mut buf).is_ok() {
            n += 1;
        }
        n
    })
    .await
    .unwrap();
    let ticks = recorder.payloads("osc://gen-tick");
    eprintln!("generator at 5000/s for 1 s: {received} received, {} ticks", ticks.len());
    assert!((4_500..=5_100).contains(&received), "{received}");
    assert!((15..=40).contains(&ticks.len()), "{} ticks", ticks.len());
}

/// Nothing listens where the generator sends: Windows reports the ICMP
/// "port unreachable" on the next send, and the generator carries on.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_generator_outlives_port_unreachable() {
    let (recorder, host) = recorder_host();
    let closed = std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    let jobs = JobRegistry::new();
    let cfg = GenConfig { target: closed.to_string(), address: "/lfo".into(), rate: 100.0, waveform: Waveform::Sine, freq: 1.0, min: 0.0, max: 1.0, as_int: false, duration_s: 0.6 };
    let job = start_generator(host, jobs, cfg).await.unwrap();
    let waiting = recorder.clone();
    let ended = tokio::task::spawn_blocking(move || waiting.wait_for("job://ended", Duration::from_secs(3), |ended| ended["job_id"] == job.id)).await.unwrap().expect("it ends with its duration");
    eprintln!("ended: {ended}");
    assert!(ended["error"].is_null(), "{ended}");
    let sent = recorder.payloads("osc://gen-tick").last().map(|tick| tick["sent"].as_u64().unwrap()).unwrap_or(0);
    assert!(sent >= 40, "sent {sent}");
}
