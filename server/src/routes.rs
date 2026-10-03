//! The server's HTTP surface:
//!
//! | Route | |
//! | --- | --- |
//! | `GET /api/health` | liveness and version, open to all (container health checks) |
//! | `POST /api/invoke/<command>` | one engine command, JSON arguments in, JSON result out |
//! | `POST /api/run` | an experiment run to its end: the result, or its steps as lines (`run.rs`) |
//! | `GET /api/openapi.json` | this API, described (`docs/api/openapi.json`) |
//! | `GET /api/events` | WebSocket of engine events |
//! | `GET /api/files?path=` | a file from the data folder (exports, reports), as a download |
//! | `GET/POST /login`, `POST /logout` | the session cookie for browsers |
//! | everything else | the interface |
//!
//! Every request passes `guard` (host, origin, authentication) and leaves with
//! the same security headers.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{ConnectInfo, DefaultBodyLimit, Form, Path as UrlPath, Query, Request, State, WebSocketUpgrade};
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use signal_lab_engine::error::EngineError;
use signal_lab_engine::service::{Failure, JOB_COMMANDS, VERSION};
use signal_lab_engine::{paths, Service};
use tokio::sync::{broadcast, watch};
use tower_http::services::{ServeDir, ServeFile};

use crate::auth::Auth;
use crate::events;
use crate::run;

/// What every handler can reach.
pub struct Inner {
    pub service: Service,
    pub events: broadcast::Sender<Arc<str>>,
    pub auth: Auth,
    pub ui_dir: Option<PathBuf>,
    pub stopping: watch::Receiver<bool>,
}

pub type AppState = Arc<Inner>;

/// An experiment document is at most 4 MiB; the largest arguments are the
/// feedback form's (`feedback_send`): 15 MB of screenshots and logs, base64.
const BODY_LIMIT: usize = 24 * 1024 * 1024;
const DOWNLOAD_LIMIT: u64 = 256 * 1024 * 1024;
/// A wrong token costs this long, which makes guessing slow.
const WRONG_TOKEN_DELAY: Duration = Duration::from_secs(1);

/// The interface may load only what this server serves; it calls only this server.
const CSP: &str = "default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; \
    script-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'";

pub(crate) fn failure(status: StatusCode, error: EngineError) -> Response {
    (status, Json(Failure::Engine(error))).into_response()
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route("/invoke/{command}", post(invoke))
        .route("/run", post(run::run))
        .route("/openapi.json", get(openapi))
        .route("/events", get(events))
        .route("/files", get(download))
        .fallback(api_not_found);
    let router = Router::new()
        .nest("/api", api)
        .route("/login", get(login_page).post(login))
        .route("/logout", post(logout));
    let router = match state.ui_dir.clone() {
        Some(dir) => router.fallback_service(ServeDir::new(&dir).fallback(ServeFile::new(dir.join("index.html")))),
        None => router.fallback(no_interface),
    };
    router
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .layer(middleware::from_fn(security_headers))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(state)
}

/// Host, origin and authentication, before anything else runs.
async fn guard(State(app): State<AppState>, request: Request, next: Next) -> Response {
    let headers = request.headers();
    if !app.auth.host_allowed(headers) {
        return failure(StatusCode::FORBIDDEN, EngineError::new("auth.host"));
    }
    let path = request.uri().path();
    let changes = !matches!(*request.method(), Method::GET | Method::HEAD);
    let upgrade = headers.contains_key(header::UPGRADE);
    if (changes || upgrade) && !app.auth.origin_allowed(headers) {
        return failure(StatusCode::FORBIDDEN, EngineError::new("auth.origin"));
    }
    let open = path == "/api/health" || path == "/login";
    if !open && !app.auth.authenticated(headers) {
        if path.starts_with("/api/") {
            return failure(StatusCode::UNAUTHORIZED, EngineError::new("auth.required"));
        }
        return Redirect::to("/login").into_response();
    }
    next.run(request).await
}

async fn security_headers(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    // Not "no-referrer": under it browsers send `Origin: null` with this site's own
    // form posts (sign-in), which the origin check rightly refuses.
    headers.insert(header::REFERRER_POLICY, HeaderValue::from_static("same-origin"));
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(CSP));
    // Built assets carry a content hash in their name and never change.
    let cache = if path.starts_with("/assets/") {
        "public, max-age=31536000, immutable"
    } else if path.starts_with("/api/") || path == "/login" {
        "no-store"
    } else {
        "no-cache"
    };
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
    response
}

/// Open to all: whether the server answers, its version, and whether it asks
/// for a token (the interface shows *Sign out* only then).
async fn health(State(app): State<AppState>) -> Json<Value> {
    Json(json!({ "status": "ok", "version": VERSION, "auth": app.auth.required() }))
}

/// The API described for tools and people: the stable endpoints, the
/// commands, the run request and result (OpenAPI 3.1, written by hand).
pub const OPENAPI: &str = include_str!("../../docs/api/openapi.json");

async fn openapi() -> Response {
    ([(header::CONTENT_TYPE, HeaderValue::from_static("application/json"))], OPENAPI).into_response()
}

async fn api_not_found() -> Response {
    failure(StatusCode::NOT_FOUND, EngineError::new("api.not_found"))
}

async fn no_interface() -> Response {
    (StatusCode::NOT_FOUND, "This Signal Lab server has no interface folder: start it with --ui-dir pointing at the built `dist`.").into_response()
}

pub(crate) fn is_json(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|kind| kind.trim().eq_ignore_ascii_case("application/json"))
}

/// One engine command. Only JSON is accepted: a page elsewhere cannot send it
/// without the browser asking this server first, and this server never agrees.
async fn invoke(State(app): State<AppState>, UrlPath(command): UrlPath<String>, ConnectInfo(peer): ConnectInfo<SocketAddr>, headers: HeaderMap, body: Bytes) -> Response {
    if !is_json(&headers) {
        return failure(StatusCode::UNSUPPORTED_MEDIA_TYPE, EngineError::new("command.json_required"));
    }
    let args: Value = if body.is_empty() {
        Value::Null
    } else {
        match serde_json::from_slice(&body) {
            Ok(args) => args,
            Err(error) => return failure(StatusCode::BAD_REQUEST, EngineError::new("command.args_invalid").with("name", &command).because(error)),
        }
    };
    match app.service.invoke(&command, args).await {
        Ok(value) => {
            if JOB_COMMANDS.contains(&command.as_str()) {
                tracing::info!(client = %peer, command, job = value["id"].as_u64().unwrap_or(0), "job started");
            } else {
                tracing::debug!(client = %peer, command, "command");
            }
            Json(value).into_response()
        }
        Err(failed) => {
            tracing::debug!(client = %peer, command, "command failed");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(failed)).into_response()
        }
    }
}

async fn events(State(app): State<AppState>, socket: WebSocketUpgrade) -> Response {
    // Subscribed before the upgrade completes, so nothing in between is missed.
    let receiver = app.events.subscribe();
    let stopping = app.stopping.clone();
    socket.max_message_size(64 * 1024).on_upgrade(move |socket| events::stream(socket, receiver, stopping))
}

#[derive(Deserialize)]
struct FileQuery {
    path: String,
}

/// A file the engine wrote — only from the data folder, never anything else on the machine.
async fn download(Query(query): Query<FileQuery>) -> Response {
    let outside = || failure(StatusCode::NOT_FOUND, EngineError::new("file.not_found").with("path", &query.path));
    let (Ok(root), Ok(file)) = (std::fs::canonicalize(paths::data_dir()), std::fs::canonicalize(&query.path)) else {
        return outside();
    };
    let Ok(meta) = std::fs::metadata(&file) else { return outside() };
    if !file.starts_with(&root) || !meta.is_file() {
        return outside();
    }
    if meta.len() > DOWNLOAD_LIMIT {
        return failure(StatusCode::PAYLOAD_TOO_LARGE, EngineError::new("file.too_large").with("max", DOWNLOAD_LIMIT / (1024 * 1024)));
    }
    let bytes = match tokio::fs::read(&file).await {
        Ok(bytes) => bytes,
        Err(error) => return failure(StatusCode::INTERNAL_SERVER_ERROR, EngineError::new("file.io").with("path", file.display()).because(error)),
    };
    let name: String = file
        .file_name()
        .map(|name| name.to_string_lossy().chars().map(|c| if c.is_ascii_alphanumeric() || "._-".contains(c) { c } else { '_' }).collect())
        .unwrap_or_else(|| "download".into());
    let disposition = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")).unwrap_or(HeaderValue::from_static("attachment"));
    ([(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream")), (header::CONTENT_DISPOSITION, disposition)], bytes).into_response()
}

// ---- signing in --------------------------------------------------------------

struct LoginText {
    lang: &'static str,
    title: &'static str,
    label: &'static str,
    button: &'static str,
    note: &'static str,
    wrong: &'static str,
}

const ENGLISH: LoginText = LoginText {
    lang: "en",
    title: "Sign in",
    label: "Access token",
    button: "Sign in",
    note: "The token is set where the server runs (SIGNALLAB_TOKEN or --token-file). This browser stays signed in for 7 days.",
    wrong: "That token is not right.",
};

const RUSSIAN: LoginText = LoginText {
    lang: "ru",
    title: "Вход",
    label: "Токен доступа",
    button: "Войти",
    note: "Токен задаётся там, где запущен сервер (SIGNALLAB_TOKEN или --token-file). Этот браузер будет помнить вход 7 дней.",
    wrong: "Токен не подходит.",
};

/// The sign-in page in every language the interface has (src/lib/locales/index.ts).
const LOGIN_TEXTS: [&LoginText; 2] = [&ENGLISH, &RUSSIAN];

/// The language of the sign-in page: the most preferred one of
/// `Accept-Language` (by its `q` weights, then order) that the page has, by
/// base language (`ru-RU` is `ru`); English when none matches.
fn login_text(headers: &HeaderMap) -> &'static LoginText {
    let accept = headers.get(header::ACCEPT_LANGUAGE).and_then(|value| value.to_str().ok()).unwrap_or("");
    preferred_languages(accept)
        .iter()
        .find_map(|tag| {
            let base = tag.split('-').next().unwrap_or("");
            LOGIN_TEXTS.into_iter().find(|text| text.lang == base)
        })
        .unwrap_or(&ENGLISH)
}

/// The tags of an `Accept-Language` header, most preferred first, without `q=0`.
fn preferred_languages(accept: &str) -> Vec<String> {
    let mut tags: Vec<(f32, usize, String)> = accept
        .split(',')
        .enumerate()
        .filter_map(|(order, part)| {
            let mut pieces = part.split(';');
            let tag = pieces.next()?.trim().to_ascii_lowercase();
            let weight = pieces
                .find_map(|piece| piece.trim().strip_prefix("q=").map(|q| q.trim().parse::<f32>().unwrap_or(0.0)))
                .unwrap_or(1.0);
            (!tag.is_empty() && tag != "*" && weight > 0.0).then_some((weight, order, tag))
        })
        .collect();
    tags.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    tags.into_iter().map(|(_, _, tag)| tag).collect()
}

fn login_html(text: &LoginText, wrong: bool) -> String {
    let error = if wrong { format!(r#"<p class="error" role="alert">{}</p>"#, text.wrong) } else { String::new() };
    include_str!("login.html")
        .replace("{{lang}}", text.lang)
        .replace("{{title}}", text.title)
        .replace("{{label}}", text.label)
        .replace("{{button}}", text.button)
        .replace("{{note}}", text.note)
        .replace("{{error}}", &error)
}

async fn login_page(State(app): State<AppState>, headers: HeaderMap) -> Response {
    if app.auth.authenticated(&headers) {
        return Redirect::to("/").into_response();
    }
    Html(login_html(login_text(&headers), false)).into_response()
}

#[derive(Deserialize)]
struct LoginForm {
    token: String,
}

async fn login(State(app): State<AppState>, ConnectInfo(peer): ConnectInfo<SocketAddr>, headers: HeaderMap, Form(form): Form<LoginForm>) -> Response {
    if !app.auth.required() {
        return Redirect::to("/").into_response();
    }
    if !app.auth.token_matches(&form.token) {
        tracing::warn!(client = %peer, "sign-in with a wrong token");
        tokio::time::sleep(WRONG_TOKEN_DELAY).await;
        return (StatusCode::UNAUTHORIZED, Html(login_html(login_text(&headers), true))).into_response();
    }
    tracing::info!(client = %peer, "signed in");
    let session = app.auth.new_session();
    ([(header::SET_COOKIE, app.auth.session_cookie(&session))], Redirect::to("/")).into_response()
}

async fn logout(State(app): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(session) = app.auth.session_from(&headers) {
        app.auth.end_session(&session);
    }
    ([(header::SET_COOKIE, app.auth.cleared_cookie())], Redirect::to("/login")).into_response()
}
