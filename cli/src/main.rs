//! `signallab`: Signal Lab without a window, for scripts and CI. Runs
//! experiments headless — with the engine in this process, or on a lab server
//! through its `/api/run` — and exits with a code a pipeline understands;
//! sends one OSC message, datagram, HTTP request or MQTT publish; fires a
//! signal from a library. Everything goes through the engine's own commands
//! (`engine::Service`), so it behaves as the app does. See docs/automation.md.

mod catalog;
mod doctor;
mod emulate;
// build.rs reads the dictionaries with it; here only its tests run.
#[cfg(test)]
mod extract;
mod fail;
mod i18n;
mod junit;
mod mcp;
mod mcp_library;
mod mcp_send;
mod mcp_tools;
mod remote;
mod run;
mod send;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use fail::Exit;
use i18n::{Lang, Texts};

#[derive(Parser, Debug)]
#[command(
    name = "signallab",
    version,
    about = "Signal Lab on the command line: run experiments headless and send single messages.",
    long_about = "Signal Lab on the command line: run experiments headless — in this process, or on a Signal Lab \
        server with --server — and send single OSC messages, datagrams, HTTP requests and MQTT publishes.\n\n\
        Exit codes: 0 every experiment passed (or the send succeeded); 1 an experiment ran and failed, or a send failed; \
        2 the invocation or a document is invalid; 3 nothing could run for a reason outside the experiment \
        (server unreachable, token refused, a port that could not be opened). See docs/automation.md."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Language of messages [default: SIGNALLAB_LANG, else the locale (LC_ALL, LC_MESSAGES, LANG), else en]
    #[arg(long, global = true, value_enum)]
    lang: Option<Lang>,

    /// Machine-readable output on stdout instead of text: one JSON object per line for `run`
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run experiments, one after another, and exit 0 only when every one passed.
    Run(RunArgs),
    /// Check experiments the way the editor does before a run; exit 0 when every one would start.
    Validate(ValidateArgs),
    /// Send one message, the way the app's screens send it.
    #[command(subcommand)]
    Send(SendCommand),
    /// Fire a signal from a signal library by its id or name.
    Fire(FireArgs),
    /// Play the other side — an HTTP API, an OSC, UDP or TCP device, an MQTT broker — until Ctrl+C or --for,
    /// printing every request and what it got.
    Emulate(EmulateArgs),
    /// List the emulators of the app's library (`signallab emulate <id>` starts one).
    Emulators(EmulatorsArgs),
    /// List the bundled templates (`signallab run <name>` runs one).
    Templates,
    /// What an experiment is made of: every kind of node with its fields, outputs and an example (JSON).
    Nodes,
    /// Serve Signal Lab to an LLM over the Model Context Protocol (stdio): experiments, sends, listening.
    Mcp(McpArgs),
    /// What stands between Signal Lab and the gear: the firewall, the network, the data folder, a server and its token.
    Doctor(DoctorArgs),
    /// The firewall: `allow` lets other machines reach signallab and the desktop app (Windows asks for administrator rights).
    #[command(subcommand)]
    Firewall(FirewallCommand),
    /// Print the version.
    Version,
}

/// Which experiments: files as the app saves and exports them, or bundled template names.
#[derive(Args, Debug)]
struct Experiments {
    /// Experiment files (.json, any version the app reads) or names of bundled templates.
    #[arg(required = true, value_name = "FILE")]
    files: Vec<String>,

    /// A parameter value for this run, NAME=VALUE; repeat for more. Each must be a parameter
    /// of at least one of the experiments, and applies to the ones that have it.
    #[arg(long = "param", short = 'p', value_name = "NAME=VALUE")]
    params: Vec<String>,

    /// Run with this profile (every experiment must have it); "" runs with the defaults.
    #[arg(long, value_name = "NAME")]
    profile: Option<String>,
}

/// Where the experiments run: here (the default) or on a server.
#[derive(Args, Debug)]
struct Place {
    /// Run on this Signal Lab server (http://host:1430) instead of in this process.
    #[arg(long, value_name = "URL", env = "SIGNALLAB_SERVER")]
    server: Option<String>,

    /// File holding the server's token [default: SIGNALLAB_TOKEN]
    #[arg(long, value_name = "PATH", env = "SIGNALLAB_TOKEN_FILE")]
    token_file: Option<PathBuf>,

    /// Where secrets come from in this process: files (SIGNALLAB_SECRET_<NAME> and --secrets-dir) or
    /// the system credential store (Windows Credential Manager, macOS Keychain).
    #[arg(long, value_enum, default_value_t = SecretSource::Files, conflicts_with = "server")]
    secrets: SecretSource,

    /// Folder of secrets, one file per name [default: /run/secrets/signallab when it exists]
    #[arg(long, value_name = "PATH", conflicts_with = "server")]
    secrets_dir: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecretSource {
    Files,
    System,
}

#[derive(Args, Debug)]
struct RunArgs {
    #[command(flatten)]
    experiments: Experiments,

    #[command(flatten)]
    place: Place,

    /// Seed for the random values (the summary of a failed run names the one it used).
    #[arg(long)]
    seed: Option<u64>,

    /// Fail a run that takes longer than this, 1–300 seconds [default: 300].
    #[arg(long, value_name = "SECONDS")]
    timeout: Option<u64>,

    /// Write a JUnit XML report here: a test suite per experiment, a test case per node.
    #[arg(long, value_name = "PATH")]
    junit: Option<PathBuf>,

    /// Copy the run report here: the file for one experiment, a folder for several.
    #[arg(long, value_name = "PATH")]
    report: Option<PathBuf>,

    /// Data folder for this process's runs [default: a temporary folder, removed at exit].
    #[arg(long, value_name = "PATH", conflicts_with = "server")]
    data_dir: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct ValidateArgs {
    #[command(flatten)]
    experiments: Experiments,

    #[command(flatten)]
    place: Place,
}

#[derive(Subcommand, Debug)]
enum SendCommand {
    /// One OSC message. Arguments are typed: i:3 f:0.5 d:1.5 h:64 s:text b:hex T F N;
    /// a plain integer is i, a plain decimal is f, anything else is s.
    Osc {
        /// host:port
        target: String,
        /// /address/of/it
        address: String,
        #[arg(value_name = "ARG", allow_hyphen_values = true, trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// One UDP datagram.
    Udp {
        /// host:port
        target: String,
        /// The payload as text.
        #[arg(long, required_unless_present = "hex", conflicts_with = "hex")]
        text: Option<String>,
        /// The payload as hex bytes: "de ad be ef".
        #[arg(long)]
        hex: Option<String>,
    },
    /// One HTTP request: prints the status, the time and the body.
    Http {
        /// GET, POST, PUT, …
        method: String,
        url: String,
        /// A request header, "Name: value"; repeat for more.
        #[arg(short = 'H', long = "header", value_name = "NAME: VALUE")]
        headers: Vec<String>,
        /// The request body, or @file to send a file's contents.
        #[arg(long, value_name = "TEXT|@FILE")]
        body: Option<String>,
        /// Exit 1 unless the response has this status.
        #[arg(long, value_name = "STATUS")]
        expect_status: Option<u16>,
        /// Milliseconds to wait for the response.
        #[arg(long, value_name = "MS", default_value_t = 10_000)]
        timeout: u64,
    },
    /// One MQTT publish (3.1.1, plain TCP, no credentials).
    Mqtt {
        /// The broker, host:port
        broker: String,
        topic: String,
        /// The payload; empty with --retain clears a retained value.
        #[arg(default_value = "")]
        payload: String,
        #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u8).range(0..=2))]
        qos: u8,
        #[arg(long)]
        retain: bool,
    },
}

#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Also check this Signal Lab server and its token.
    #[arg(long, value_name = "URL", env = "SIGNALLAB_SERVER")]
    server: Option<String>,
    /// File holding the server's token [default: SIGNALLAB_TOKEN]
    #[arg(long, value_name = "PATH", env = "SIGNALLAB_TOKEN_FILE")]
    token_file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum FirewallCommand {
    /// Replace this installation's inbound firewall rules with one that allows it, on private and domain networks.
    Allow(FirewallArgs),
}

#[derive(Args, Debug)]
pub struct FirewallArgs {
    /// On public networks too (a venue's Wi-Fi is often one).
    #[arg(long)]
    public: bool,
}

#[derive(Args, Debug)]
pub struct McpArgs {
    #[command(flatten)]
    place: Place,

    /// Data folder for the runs and their reports [default: the app's, Documents/SignalLab].
    #[arg(long, value_name = "PATH", conflicts_with = "server")]
    data_dir: Option<PathBuf>,

    /// The signal library for list_signals and fire_signal [default: the app's signals.json]
    #[arg(long, value_name = "PATH")]
    library: Option<PathBuf>,

    /// The emulator library for list_emulators and start_emulator [default: the app's emulators.json]
    #[arg(long, value_name = "PATH")]
    emulators: Option<PathBuf>,

    /// Print what a client needs to start this server, and exit.
    #[arg(long, value_name = "CLIENT", value_parser = ["claude-code", "claude-desktop", "cursor", "vscode"])]
    print_config: Option<String>,
}

#[derive(Args, Debug)]
pub struct EmulateArgs {
    /// Emulator files (one emulator, a list, or a library as the app writes it), or the id or
    /// name of one in the app's library.
    #[arg(required = true, value_name = "FILE|NAME")]
    emulators: Vec<String>,

    /// A parameter its templates read, NAME=VALUE; repeat for more.
    #[arg(long = "param", short = 'p', value_name = "NAME=VALUE")]
    params: Vec<String>,

    /// Listen here instead (IP:port), when one emulator is given.
    #[arg(long, value_name = "IP:PORT")]
    bind: Option<String>,

    /// Stop after this many seconds [default: at Ctrl+C].
    #[arg(long = "for", value_name = "SECONDS")]
    duration: Option<u64>,

    /// Seed for its random choices: a random order, jitter, generators.
    #[arg(long)]
    seed: Option<u64>,

    /// Check the emulators and exit, without opening a port.
    #[arg(long)]
    check: bool,

    /// The library names are looked up in [default: emulators.json in the app's data folder]
    #[arg(long, value_name = "PATH")]
    library: Option<PathBuf>,

    #[command(flatten)]
    place: Place,
}

#[derive(Args, Debug)]
pub struct EmulatorsArgs {
    /// The library [default: emulators.json in the app's data folder]
    #[arg(long, value_name = "PATH")]
    library: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct FireArgs {
    /// The signal's id or name.
    signal: String,
    /// The library file [default: signals.json in the app's data folder]
    #[arg(long, value_name = "PATH")]
    library: Option<PathBuf>,
}

/// What every command needs: the language and the output mode.
pub struct Ctx {
    pub texts: Texts,
    pub json: bool,
}

impl Ctx {
    /// A line for a person, on stderr (stdout is for results); nothing in --json mode.
    pub fn say(&self, line: &str) {
        if !self.json {
            eprintln!("{line}");
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let lang = Lang::choose(cli.lang, |name| std::env::var(name).ok());
    let ctx = Ctx { texts: Texts::new(lang), json: cli.json };
    let exit = match cli.command {
        Command::Run(args) => run::run(&ctx, args).await,
        Command::Validate(args) => run::validate(&ctx, args).await,
        Command::Send(command) => send::send(&ctx, command).await,
        Command::Fire(args) => send::fire(&ctx, args).await,
        Command::Emulate(args) => emulate::emulate(&ctx, args).await,
        Command::Emulators(args) => emulate::emulators(&ctx, args),
        Command::Templates => run::templates(&ctx),
        Command::Nodes => {
            println!("{}", serde_json::to_string_pretty(&catalog::describe(&ctx.texts)).unwrap_or_default());
            Exit::Passed
        }
        Command::Mcp(args) => mcp::serve(ctx, args).await,
        Command::Doctor(args) => doctor::doctor(&ctx, args).await,
        Command::Firewall(FirewallCommand::Allow(args)) => doctor::firewall(&ctx, args).await,
        Command::Version => {
            if ctx.json {
                println!("{}", serde_json::json!({ "version": env!("CARGO_PKG_VERSION") }));
            } else {
                println!("signallab {}", env!("CARGO_PKG_VERSION"));
            }
            Exit::Passed
        }
    };
    ExitCode::from(exit.code())
}
