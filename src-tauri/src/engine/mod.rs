//! The Signal Lab engine: protocol simulators and network tooling, each exposed
//! to the UI as Tauri commands (see `crate::commands`).

pub mod http;
pub mod jobs;
pub mod net;
pub mod netsim;
pub mod osc;
pub mod osc_codec;
pub mod scan;
pub mod storm;

pub use jobs::{JobInfo, JobRegistry};
