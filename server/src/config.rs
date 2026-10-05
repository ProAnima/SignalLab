//! How the server is told what to do: command-line options, each with an
//! environment variable for containers, checked once at startup so a server
//! that could be reached by others never starts without a token.

use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// A token shorter than this is refused: it would be guessable.
pub const MIN_TOKEN_LEN: usize = 24;

#[derive(Parser, Debug)]
#[command(
    name = "signal-lab-server",
    version,
    about = "Signal Lab without a window: the engine and its interface, served to a browser.",
    long_about = "Signal Lab without a window: the engine and its interface, served to a browser.\n\n\
        Listens on 127.0.0.1:1430 by default. Listening on any other address requires a token \
        (--token-file or SIGNALLAB_TOKEN; generate one with `signal-lab-server token`), or \
        --generate-token to have one made in the data folder on the first start.\n\n\
        Broadcast, multicast and discovery reach the physical network only with host networking \
        (docker run --network host, Linux hosts)."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    #[command(flatten)]
    pub options: Options,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Print a new random token for --token-file / SIGNALLAB_TOKEN.
    Token,
    /// Exit 0 when a server answers on --listen (for container health checks).
    Healthcheck,
}

#[derive(Args, Debug, Clone)]
pub struct Options {
    /// Address and port to listen on. Anything but loopback requires a token.
    #[arg(long, env = "SIGNALLAB_LISTEN", default_value = "127.0.0.1:1430")]
    pub listen: SocketAddr,

    /// Access token. Prefer --token-file: arguments are visible to other users of the machine.
    #[arg(long, env = "SIGNALLAB_TOKEN", hide_env_values = true)]
    pub token: Option<String>,

    /// File holding the access token (e.g. a Docker secret).
    #[arg(long, env = "SIGNALLAB_TOKEN_FILE")]
    pub token_file: Option<PathBuf>,

    /// Without --token or --token-file, on an address beyond loopback (where a token is
    /// needed): use the token in `<data folder>/token`, making it (readable by this user
    /// only) on the first start. A server that starts with nothing set up is still never
    /// open: the new token is printed once, to sign in with. On loopback it does nothing.
    #[arg(long, env = "SIGNALLAB_GENERATE_TOKEN")]
    pub generate_token: bool,

    /// Folder for the experiment, the signal library, run reports and exports.
    #[arg(long, env = "SIGNALLAB_DATA_DIR")]
    pub data_dir: Option<PathBuf>,

    /// Folder of read-only secrets, one file per name (the Docker secrets layout).
    #[arg(long, env = "SIGNALLAB_SECRETS_DIR", default_value = "/run/secrets/signallab")]
    pub secrets_dir: PathBuf,

    /// The built interface (the `dist` folder). Default: `ui` next to the executable, else `./dist`.
    #[arg(long, env = "SIGNALLAB_UI_DIR")]
    pub ui_dir: Option<PathBuf>,

    /// Host names the server may be reached by, comma-separated, as a guard against DNS
    /// rebinding. These and the loopback names are always allowed, with a token or without.
    /// Left empty, a server with a token answers to any name and one without a token to
    /// loopback names only.
    #[arg(long = "allowed-host", env = "SIGNALLAB_ALLOWED_HOSTS", value_delimiter = ',')]
    pub allowed_hosts: Vec<String>,

    /// Send the session cookie only over HTTPS (set this behind a TLS proxy).
    #[arg(long, env = "SIGNALLAB_SECURE_COOKIE")]
    pub secure_cookie: bool,

    /// Log filter: error, warn, info, debug, or per module (signal_lab_server=debug).
    #[arg(long, env = "SIGNALLAB_LOG", default_value = "info")]
    pub log: String,

    /// Log lines as text (coloured only on a terminal) or as one JSON object per line.
    #[arg(long, env = "SIGNALLAB_LOG_FORMAT", value_enum, default_value_t = LogFormat::Text)]
    pub log_format: LogFormat,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogFormat {
    Text,
    Json,
}

/// Where the token came from, when the server made or found it itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MadeToken {
    /// Made on this start and saved here: say so, and show it once.
    New(PathBuf),
    /// Made on an earlier start.
    Kept(PathBuf),
}

/// The checked configuration.
#[derive(Clone, Debug)]
pub struct Config {
    pub listen: SocketAddr,
    pub token: Option<String>,
    /// Set when the token is the one --generate-token keeps in the data folder.
    pub made_token: Option<MadeToken>,
    pub data_dir: Option<PathBuf>,
    pub secrets_dir: PathBuf,
    pub ui_dir: Option<PathBuf>,
    pub allowed_hosts: Vec<String>,
    pub secure_cookie: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    TokenTwice,
    TokenFile(PathBuf, String),
    TokenTooShort(usize),
    TokenWhitespace,
    NeedsToken(SocketAddr),
    GenerateNeedsDataDir,
    TokenSave(PathBuf, String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::TokenTwice => write!(f, "give the token once: --token/SIGNALLAB_TOKEN or --token-file/SIGNALLAB_TOKEN_FILE, not both"),
            ConfigError::TokenFile(path, error) => write!(f, "cannot read the token file {}: {error}", path.display()),
            ConfigError::TokenTooShort(length) => write!(
                f,
                "the token has {length} characters; it needs at least {MIN_TOKEN_LEN} — generate one with `signal-lab-server token`"
            ),
            ConfigError::TokenWhitespace => write!(f, "the token must not contain spaces or line breaks"),
            ConfigError::NeedsToken(listen) => write!(
                f,
                "refusing to listen on {listen} without a token: anyone who can reach it could send traffic from this machine. \
                 Set --token-file or SIGNALLAB_TOKEN (generate one with `signal-lab-server token`), use --generate-token, \
                 or listen on 127.0.0.1"
            ),
            ConfigError::GenerateNeedsDataDir => {
                write!(f, "--generate-token keeps the token in the data folder: set --data-dir or SIGNALLAB_DATA_DIR")
            }
            ConfigError::TokenSave(path, error) => write!(f, "cannot save the new token in {}: {error}", path.display()),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Options {
    pub fn resolve(self) -> Result<Config, ConfigError> {
        let token = match (self.token, self.token_file) {
            (Some(_), Some(_)) => return Err(ConfigError::TokenTwice),
            (Some(token), None) => Some(token),
            (None, Some(path)) => Some(std::fs::read_to_string(&path).map_err(|error| ConfigError::TokenFile(path.clone(), error.to_string()))?),
            (None, None) => None,
        };
        // A token file ends in a newline that is not part of the token.
        let mut token = token.map(|token| token.trim().to_string()).filter(|token| !token.is_empty());
        let mut made_token = None;
        if token.is_none() && self.generate_token && !self.listen.ip().is_loopback() {
            let dir = self.data_dir.as_ref().ok_or(ConfigError::GenerateNeedsDataDir)?;
            let (value, made) = kept_token(&dir.join("token"))?;
            token = Some(value);
            made_token = Some(made);
        }
        if let Some(token) = &token {
            if token.chars().any(char::is_whitespace) {
                return Err(ConfigError::TokenWhitespace);
            }
            if token.chars().count() < MIN_TOKEN_LEN {
                return Err(ConfigError::TokenTooShort(token.chars().count()));
            }
        }
        if token.is_none() && !self.listen.ip().is_loopback() {
            return Err(ConfigError::NeedsToken(self.listen));
        }
        let allowed_hosts = self.allowed_hosts.into_iter().map(|host| host.trim().to_ascii_lowercase()).filter(|host| !host.is_empty()).collect();
        Ok(Config {
            listen: self.listen,
            token,
            made_token,
            data_dir: self.data_dir,
            secrets_dir: self.secrets_dir,
            ui_dir: self.ui_dir.or_else(default_ui_dir),
            allowed_hosts,
            secure_cookie: self.secure_cookie,
        })
    }
}

/// `ui` next to the executable (an installed server), else `dist` (a checkout).
fn default_ui_dir() -> Option<PathBuf> {
    let beside = std::env::current_exe().ok().and_then(|exe| exe.parent().map(|dir| dir.join("ui")));
    beside.into_iter().chain([PathBuf::from("dist")]).find(|dir| dir.join("index.html").is_file())
}

/// The token kept in `path`, made there first when there is none. Written only
/// for this user (0600) and never overwritten: a second start reuses it, so
/// signed-in browsers and scripts keep working across restarts and updates.
fn kept_token(path: &std::path::Path) -> Result<(String, MadeToken), ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let value = text.trim().to_string();
            if value.chars().count() < MIN_TOKEN_LEN || value.chars().any(char::is_whitespace) {
                return Err(ConfigError::TokenFile(path.to_path_buf(), "it does not hold a valid token; remove it to have a new one made".into()));
            }
            Ok((value, MadeToken::Kept(path.to_path_buf())))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let value = new_token();
            let save = |error: std::io::Error| ConfigError::TokenSave(path.to_path_buf(), error.to_string());
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(save)?;
            }
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
            use std::io::Write;
            let mut file = options.open(path).map_err(save)?;
            file.write_all(format!("{value}\n").as_bytes()).map_err(save)?;
            Ok((value, MadeToken::New(path.to_path_buf())))
        }
        Err(error) => Err(ConfigError::TokenFile(path.to_path_buf(), error.to_string())),
    }
}

/// A new random token: 32 bytes as hex.
pub fn new_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(listen: &str) -> Options {
        Options {
            listen: listen.parse().unwrap(),
            token: None,
            token_file: None,
            generate_token: false,
            data_dir: None,
            secrets_dir: "/run/secrets/signallab".into(),
            ui_dir: None,
            allowed_hosts: vec![],
            secure_cookie: false,
            log: "info".into(),
            log_format: LogFormat::Text,
        }
    }

    #[test]
    fn only_loopback_runs_without_a_token() {
        assert!(options("127.0.0.1:1430").resolve().is_ok());
        assert!(options("[::1]:1430").resolve().is_ok());
        assert_eq!(options("0.0.0.0:1430").resolve().unwrap_err(), ConfigError::NeedsToken("0.0.0.0:1430".parse().unwrap()));
        assert!(matches!(options("192.168.1.20:1430").resolve(), Err(ConfigError::NeedsToken(_))));
        let mut with_token = options("0.0.0.0:1430");
        with_token.token = Some(new_token());
        assert!(with_token.resolve().unwrap().token.is_some());
    }

    #[test]
    fn tokens_must_be_strong_and_given_once() {
        let token = |value: &str| {
            let mut options = options("0.0.0.0:1430");
            options.token = Some(value.into());
            options.resolve()
        };
        assert_eq!(token("short").unwrap_err(), ConfigError::TokenTooShort(5));
        assert_eq!(token("has a space but is long enough!!").unwrap_err(), ConfigError::TokenWhitespace);
        assert!(matches!(token("   ").unwrap_err(), ConfigError::NeedsToken(_)), "a blank token is no token");
        assert_eq!(new_token().len(), 64);
        assert_ne!(new_token(), new_token());

        let dir = std::env::temp_dir().join(format!("signallab-token-{:016x}", rand::random::<u64>()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("token");
        let value = new_token();
        std::fs::write(&file, format!("{value}\n")).unwrap();
        let mut from_file = options("0.0.0.0:1430");
        from_file.token_file = Some(file.clone());
        assert_eq!(from_file.clone().resolve().unwrap().token.as_deref(), Some(value.as_str()), "the trailing newline is not part of it");
        from_file.token = Some(new_token());
        assert_eq!(from_file.resolve().unwrap_err(), ConfigError::TokenTwice);
        let mut missing = options("0.0.0.0:1430");
        missing.token_file = Some(dir.join("absent"));
        assert!(matches!(missing.resolve(), Err(ConfigError::TokenFile(..))));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_generated_token_is_made_once_and_kept() {
        let dir = std::env::temp_dir().join(format!("signallab-made-{:016x}", rand::random::<u64>()));
        let mut generate = options("0.0.0.0:1430");
        generate.generate_token = true;
        assert_eq!(generate.clone().resolve().unwrap_err(), ConfigError::GenerateNeedsDataDir);
        generate.data_dir = Some(dir.clone());

        let first = generate.clone().resolve().unwrap();
        let path = dir.join("token");
        assert_eq!(first.made_token, Some(MadeToken::New(path.clone())));
        let token = first.token.unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), format!("{token}\n"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600, "only this user reads it");
        }
        let second = generate.clone().resolve().unwrap();
        assert_eq!((second.token.as_deref(), second.made_token), (Some(token.as_str()), Some(MadeToken::Kept(path.clone()))), "a restart keeps it");

        // A token given explicitly wins, and nothing is made.
        let mut given = generate.clone();
        given.token = Some(new_token());
        assert_eq!(given.resolve().unwrap().made_token, None);

        // On loopback no token is needed, so none is made or used.
        let mut local = generate.clone();
        local.listen = "127.0.0.1:1430".parse().unwrap();
        let local = local.resolve().unwrap();
        assert_eq!((local.token, local.made_token), (None, None));

        std::fs::write(&path, "short\n").unwrap();
        assert!(matches!(generate.resolve(), Err(ConfigError::TokenFile(..))), "a damaged file is reported, never replaced");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn allowed_hosts_are_normalized() {
        let mut options = options("127.0.0.1:1430");
        options.allowed_hosts = vec![" Lab.Example ".into(), "".into()];
        assert_eq!(options.resolve().unwrap().allowed_hosts, ["lab.example"]);
    }

    /// The help says what `auth::Auth::host_allowed` does (its test pins the rules): listed
    /// names pass with a token or without; the empty list means any name only with a token.
    #[test]
    fn the_allowed_host_help_tells_what_the_server_does() {
        use clap::CommandFactory;
        let help = Cli::command().render_long_help().to_string();
        let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(help.contains("These and the loopback names are always allowed, with a token or without."), "{help}");
        assert!(help.contains("a server with a token answers to any name and one without a token to loopback names only"), "{help}");
        assert!(!help.contains("Without a token only loopback names are."), "the old wording left out the listed names");
    }
}
