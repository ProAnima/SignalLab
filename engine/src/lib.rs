//! The Signal Lab engine: protocol simulators, network tooling and the
//! experiment runner, independent of any window. The desktop app
//! (`src-tauri`) and the server (`server`) both run it through
//! [`service::Service`], the one command table, and give it a [`host::Host`]
//! to deliver events to their interface.

pub mod broadcast;
pub mod cookies;
pub mod discovery;
pub mod emulator;
pub mod emulator_files;
pub mod emulator_http;
pub mod emulator_job;
pub mod emulator_match;
pub mod emulator_mqtt;
pub mod emulator_net;
pub mod emulator_rules;
pub mod emulator_run;
pub mod emulator_state;
pub mod error;
pub mod experiment;
pub mod experiment_actions;
pub mod experiment_compare;
pub mod experiment_data;
pub mod experiment_fields;
pub mod experiment_files;
pub mod experiment_flow;
pub mod experiment_report;
pub mod experiment_run;
pub mod experiment_steps;
pub mod experiment_validate;
pub mod feedback;
pub mod firewall;
pub mod host;
pub mod http;
pub mod hub;
pub mod http_auth;
pub mod inspect;
pub mod jobs;
pub mod latency;
pub mod listen;
pub mod load;
pub mod matching;
pub mod mqtt;
pub mod mqtt_codec;
pub mod mqtt_dial;
pub mod net;
pub mod netsim;
pub mod netsim_run;
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
pub mod ws;

pub use host::{EventSink, Host};
pub use inspect::Capture;
pub use jobs::{JobInfo, JobRegistry};
pub use mqtt::MqttHub;
pub use service::{Failure, Mode, Service};
