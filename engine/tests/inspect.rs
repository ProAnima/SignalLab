//! The Inspector's whole payloads through the command table: a datagram larger
//! than the preview is kept to its last byte, handed out by number
//! (`inspect_payload`) and written by both exports — to a scratch folder,
//! never to the user's Documents.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use base64::Engine as _;
use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::inspect::{self, Frame};
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};

fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-inspect-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        paths::data_dir()
    })
    .clone()
}

#[tokio::test]
async fn a_large_frame_is_kept_whole_handed_out_and_exported() {
    let dir = scratch_data_dir();
    let host = Host::new(Recorder::new(), Capture::new());
    let service = Service::new(host.clone(), Mode::Desktop, Arc::new(FileStore::new(None).with_env(|_| None)));
    service.invoke("inspect_set_enabled", json!({ "enabled": true })).await.unwrap();

    let datagram: Vec<u8> = (0..5000u32).map(|i| (i * 7 % 256) as u8).collect();
    let seq = inspect::publish(&host, Frame::rx("udp", "test").remote("127.0.0.1:9000").payload(&datagram).summary("big")).unwrap();

    let listed = service.invoke("inspect_snapshot", json!({ "limit": 5 })).await.unwrap();
    let frame = &listed[0];
    assert_eq!((frame["bytes"].as_u64(), frame["kept"].as_u64()), (Some(5000), Some(5000)));
    assert!(frame.get("data").is_none(), "a listing carries the preview only");
    assert!(frame["hex"].as_str().unwrap().contains("more bytes"));

    let payload = service.invoke("inspect_payload", json!({ "seq": seq })).await.unwrap();
    let hex: Vec<u8> = payload["hex"].as_str().unwrap().split(' ').map(|byte| u8::from_str_radix(byte, 16).unwrap()).collect();
    assert_eq!(hex, datagram, "every byte, in order");
    assert!(payload["dump"].as_str().unwrap().contains("\n1380  "), "the last row is there");

    let gone = serde_json::to_value(service.invoke("inspect_payload", json!({ "seq": seq + 100 })).await.unwrap_err()).unwrap();
    assert_eq!((gone["code"].as_str(), gone["params"]["seq"].as_str()), (Some("inspect.frame_gone"), Some((seq + 100).to_string().as_str())));
    let unknown = serde_json::to_value(service.invoke("inspect_payload", json!({ "seq": seq, "whole": true })).await.unwrap_err()).unwrap();
    assert_eq!(unknown["code"], "command.args_invalid");

    let jsonl = service.invoke("inspect_export", json!({ "format": "jsonl" })).await.unwrap();
    let path = PathBuf::from(jsonl.as_str().unwrap());
    assert!(path.starts_with(&dir), "{}", path.display());
    let line: Value = serde_json::from_str(std::fs::read_to_string(&path).unwrap().lines().next().unwrap()).unwrap();
    assert_eq!(base64::engine::general_purpose::STANDARD.decode(line["data"].as_str().unwrap()).unwrap(), datagram);
    let txt = service.invoke("inspect_export", json!({ "format": "txt" })).await.unwrap();
    let text = std::fs::read_to_string(txt.as_str().unwrap()).unwrap();
    assert!(text.contains("\n1380  ") && !text.contains("more bytes"), "every row in the text export too");
}
