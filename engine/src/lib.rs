//! The Signal Lab engine: protocol simulators, network tooling and the
//! experiment runner, independent of any window. The desktop app
//! (`src-tauri`) and the server (`server`) both run it through
//! [`service::Service`], the one command table, and give it a [`host::Host`]
//! to deliver events to their interface.

pub mod broadcast;
pub mod emulator;
pub mod emulator_files;
pub mod emulator_http;
pub mod emulator_net;
pub mod error;
pub mod experiment;
pub mod experiment_actions;
pub mod experiment_data;
pub mod experiment_files;
pub mod experiment_run;
pub mod experiment_steps;
pub mod experiment_validate;
pub mod firewall;
pub mod host;
pub mod http;
pub mod inspect;
pub mod jobs;
pub mod listen;
pub mod matching;
pub mod mqtt;
pub mod mqtt_codec;
pub mod net;
pub mod netsim;
pub mod osc;
pub mod osc_codec;
pub mod paths;
pub mod scan;
pub mod secrets;
pub mod service;
pub mod signals;
pub mod storm;
pub mod subscribe;
pub mod template;
pub mod transport;

pub use host::{EventSink, Host};
pub use inspect::Capture;
pub use jobs::{JobInfo, JobRegistry};
pub use mqtt::MqttHub;
pub use service::{Failure, Mode, Service};
