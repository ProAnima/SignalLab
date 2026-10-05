//! HTTP client: single request inspector + concurrent load ("burst") runner.
//! A run's load on a node (`load`) sends through the same client and exchange.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use crate::host::Host;

use super::cookies::CookieJar;
use super::error::{EngineError, EngineResult, Field};
use super::http_auth::{self, Answering, Auth, DigestMemory};
use super::latency::{LatencyHistogram, Percentiles};
use super::inspect::{self, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry, TaskGuard};
use super::transport::{self, Cause};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: Option<String>,
    /// Per-request timeout in milliseconds.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Basic, Bearer or Digest; its `Authorization` header is never reported.
    #[serde(default, skip_serializing_if = "Auth::is_none")]
    pub auth: Auth,
}

fn default_timeout() -> u64 {
    10_000
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HttpResponse {
    pub ok: bool,
    pub status: u16,
    pub status_text: String,
    pub latency_ms: f64,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub body_bytes: usize,
    /// True if the body was truncated for display.
    pub truncated: bool,
    /// The failure with every layer of its cause, when there was no response.
    pub error: Option<String>,
    /// What kind of failure `error` is, for a localized message.
    #[serde(default)]
    pub cause: Option<Cause>,
    /// How a Digest request went: a 401's challenge answered, or why it could not be.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<DigestOutcome>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DigestOutcome {
    /// The server asked (401) and was answered: the response is to the second request.
    pub challenged: bool,
    /// The 401 could not be answered: no Digest asked for, or one Signal Lab does not speak.
    pub error: Option<EngineError>,
}

const MAX_BODY_PREVIEW: usize = 256 * 1024;

/// Redirects followed, as reqwest's own default.
const MAX_REDIRECTS: usize = 10;

/// Answers to one request's Digest challenges after the first, while the server
/// says the nonce ran out or asks with another: under load a new nonce can wear
/// out before a request that met it is answered.
const DIGEST_ROUNDS: usize = 3;

/// A client for `req`: a Digest request follows its redirects itself, so the
/// URL that asks is the one answered.
pub(crate) fn build_client(req: &HttpRequest, jar: Option<Arc<CookieJar>>) -> EngineResult<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_millis(req.timeout_ms.max(1)))
        .danger_accept_invalid_certs(false)
        .user_agent("SignalLab/0.1");
    if matches!(req.auth, Auth::Digest { .. }) {
        builder = builder.redirect(reqwest::redirect::Policy::none());
    }
    if let Some(jar) = jar {
        builder = builder.cookie_provider(jar);
    }
    builder.build().map_err(|e| EngineError::new("http.client_failed").because(transport::chain(&e)))
}

/// What one exchange sends: `req` as written, or a hop of it after a redirect.
struct Outgoing<'a> {
    method: String,
    url: &'a str,
    headers: &'a [(String, String)],
    body: Option<&'a str>,
}

impl<'a> Outgoing<'a> {
    fn of(req: &'a HttpRequest) -> Self {
        Outgoing { method: req.method.to_uppercase(), url: &req.url, headers: &req.headers, body: req.body.as_deref() }
    }
}

/// The request as reqwest sends it, with `authorization` when there is one.
fn builder(client: &reqwest::Client, out: &Outgoing<'_>, authorization: Option<&str>) -> reqwest::RequestBuilder {
    let method = reqwest::Method::from_bytes(out.method.as_bytes()).unwrap_or(reqwest::Method::GET);
    let mut builder = client.request(method, out.url);
    for (k, v) in out.headers {
        if !k.trim().is_empty() {
            builder = builder.header(k, v);
        }
    }
    if let Some(authorization) = authorization {
        builder = builder.header(reqwest::header::AUTHORIZATION, authorization);
    }
    if let Some(b) = out.body {
        if !b.is_empty() {
            builder = builder.body(b.to_string());
        }
    }
    builder
}

/// One exchange: sent, the answer read (or the failure classified).
async fn exchange(client: &reqwest::Client, out: &Outgoing<'_>, authorization: Option<&str>) -> HttpResponse {
    let start = Instant::now();
    match builder(client, out, authorization).send().await {
        Ok(resp) => {
            let status = resp.status();
            // A value that is not ASCII (a filename in UTF-8, Latin-1) is shown as near as it reads.
            let headers: Vec<(String, String)> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).into_owned()))
                .collect();
            let full = resp.bytes().await.unwrap_or_default();
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            let body_bytes = full.len();
            let truncated = body_bytes > MAX_BODY_PREVIEW;
            let slice = &full[..body_bytes.min(MAX_BODY_PREVIEW)];
            HttpResponse {
                ok: status.is_success(),
                status: status.as_u16(),
                status_text: status.canonical_reason().unwrap_or("").to_string(),
                latency_ms,
                headers,
                body: String::from_utf8_lossy(slice).into_owned(),
                body_bytes,
                truncated,
                error: None,
                cause: None,
                digest: None,
            }
        }
        Err(e) => HttpResponse {
            ok: false,
            status: 0,
            status_text: String::new(),
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
            headers: Vec::new(),
            body: String::new(),
            body_bytes: 0,
            truncated: false,
            error: Some(transport::chain(&e)),
            cause: Some(transport::of_reqwest(&e)),
            digest: None,
        },
    }
}

/// The `WWW-Authenticate` values of a response, for `http_auth::digest_challenge`.
fn challenges(response: &HttpResponse) -> impl Iterator<Item = &str> {
    response.headers.iter().filter(|(name, _)| name.eq_ignore_ascii_case("www-authenticate")).map(|(_, value)| value.as_str())
}

/// The request target a Digest answer hashes: the URL's path and query.
fn request_target(url: &reqwest::Url) -> String {
    match url.query() {
        Some(query) => format!("{}?{query}", url.path()),
        None => url.path().to_string(),
    }
}

/// Send `req` with its authentication. The latency is all of it: what a client waits.
pub(crate) async fn execute(client: &reqwest::Client, req: &HttpRequest, memory: &DigestMemory) -> HttpResponse {
    let out = Outgoing::of(req);
    match &req.auth {
        Auth::None => exchange(client, &out, None).await,
        Auth::Basic { username, password } => exchange(client, &out, Some(&http_auth::basic(username, password))).await,
        Auth::Bearer { token } => exchange(client, &out, Some(&format!("Bearer {token}"))).await,
        Auth::Digest { username, password } => digest(client, req, username, password, memory).await,
    }
}

/// A Digest request. The 401's challenge is answered and the request sent
/// again — or, when `memory` holds what this origin asked an earlier request,
/// answered at once; a stale or new challenge is answered again (`DIGEST_ROUNDS`). Redirects
/// are followed here, as reqwest follows them, so the URL that asks is the one
/// answered, with its own path and the method that reached it; a challenge
/// from another origin than the request's is not answered (`answerable`).
async fn digest(client: &reqwest::Client, req: &HttpRequest, username: &str, password: &str, memory: &DigestMemory) -> HttpResponse {
    let Ok(origin) = reqwest::Url::parse(&req.url) else {
        // reqwest says what is wrong with the URL.
        return exchange(client, &Outgoing::of(req), None).await;
    };
    let mut url = origin.clone();
    let mut method = req.method.to_uppercase();
    let mut headers = req.headers.clone();
    let mut body = req.body.clone();
    let mut latency_ms = 0.0;
    let mut outcome: Option<DigestOutcome> = None;
    for _ in 0..=MAX_REDIRECTS {
        let here = url.origin().ascii_serialization();
        let answerable_here = answerable(&origin, &url);
        let uri = request_target(&url);
        let target = url.to_string();
        let out = Outgoing { method: method.clone(), url: &target, headers: &headers, body: body.as_deref() };
        let answer = |answering: &Answering| answering.header(username, password, &method, &uri, body.as_deref().unwrap_or_default().as_bytes());
        let early = if answerable_here { memory.next(&here) } else { None };
        let mut response = exchange(client, &out, early.as_ref().map(answer).as_deref()).await;
        latency_ms += response.latency_ms;
        if early.is_some() {
            outcome.get_or_insert(DigestOutcome { challenged: false, error: None });
        }
        if response.status == 401 {
            let refused = |error: EngineError| Some(DigestOutcome { challenged: false, error: Some(error) });
            match http_auth::digest_challenge(challenges(&response)) {
                Ok(Some(_)) if !answerable_here => {
                    return HttpResponse { latency_ms, digest: refused(EngineError::new("http.digest_other_origin").with("origin", &here)), ..response };
                }
                Ok(Some(challenge)) => {
                    let mut asked = challenge;
                    let mut rounds = 0;
                    loop {
                        let answering = memory.remember(&here, asked);
                        response = exchange(client, &out, Some(&answer(&answering))).await;
                        latency_ms += response.latency_ms;
                        if response.status != 401 {
                            break;
                        }
                        // Asked again because the nonce ran out, or because the server has
                        // another by now: answered again, a few times at most. Refused with
                        // the nonce just answered, it is the credentials that are wrong.
                        match http_auth::digest_challenge(challenges(&response)) {
                            Ok(Some(next)) if rounds < DIGEST_ROUNDS && (next.stale || next.nonce != answering.challenge.nonce) => {
                                asked = next;
                                rounds += 1;
                            }
                            _ => {
                                memory.forget(&here, &answering.challenge.nonce);
                                break;
                            }
                        }
                    }
                    outcome = Some(DigestOutcome { challenged: true, error: None });
                }
                Ok(None) => return HttpResponse { latency_ms, digest: refused(EngineError::new("http.digest_not_offered")), ..response },
                Err(error) => return HttpResponse { latency_ms, digest: refused(error), ..response },
            }
        }
        let Some(next) = redirect(&response, &url) else {
            return HttpResponse { latency_ms, digest: outcome, ..response };
        };
        // 301, 302 and 303 go on as GET without a body (HEAD stays HEAD); 307 and 308 as they were.
        if matches!(response.status, 301..=303) {
            if method != "HEAD" {
                method = "GET".into();
            }
            body = None;
            headers.retain(|(name, _)| !["content-type", "content-length", "content-encoding", "transfer-encoding"].iter().any(|kept| name.trim().eq_ignore_ascii_case(kept)));
        }
        // Credentials and cookies typed for one host never go on to another.
        if next.host_str() != url.host_str() || next.port_or_known_default() != url.port_or_known_default() {
            headers.retain(|(name, _)| !["authorization", "cookie", "cookie2", "proxy-authorization", "www-authenticate"].iter().any(|kept| name.trim().eq_ignore_ascii_case(kept)));
        }
        // Where it came from, as reqwest says it — never from HTTPS to plain HTTP.
        headers.retain(|(name, _)| !name.trim().eq_ignore_ascii_case("referer"));
        if !(url.scheme() == "https" && next.scheme() == "http") {
            let mut referer = url.clone();
            let _ = referer.set_username("");
            let _ = referer.set_password(None);
            referer.set_fragment(None);
            headers.push(("Referer".into(), referer.to_string()));
        }
        url = next;
    }
    HttpResponse {
        ok: false,
        status: 0,
        status_text: String::new(),
        latency_ms,
        headers: Vec::new(),
        body: String::new(),
        body_bytes: 0,
        truncated: false,
        error: Some(format!("error following redirect for url ({url}): too many redirects")),
        cause: Some(Cause::Failed),
        digest: outcome,
    }
}

/// Whether a challenge from `here` is answered for a request to `requested`:
/// the same origin, or the same host moved from http:// to https:// on the
/// default ports — the credentials then go to the host they were typed for,
/// over TLS.
fn answerable(requested: &reqwest::Url, here: &reqwest::Url) -> bool {
    requested.origin() == here.origin()
        || (requested.scheme() == "http" && here.scheme() == "https" && requested.host_str() == here.host_str() && requested.port().is_none() && here.port().is_none())
}

/// Where a redirect response sends the request next, if it is one with a `Location`.
fn redirect(response: &HttpResponse, from: &reqwest::Url) -> Option<reqwest::Url> {
    if !matches!(response.status, 301 | 302 | 303 | 307 | 308) {
        return None;
    }
    let location = response.headers.iter().find(|(name, _)| name.eq_ignore_ascii_case("location"))?;
    let next = from.join(&location.1).ok()?;
    matches!(next.scheme(), "http" | "https").then_some(next)
}

/// Response body kept in a capture frame's detail pane.
const FRAME_BODY_PREVIEW: usize = 2_000;

/// Render one request/response exchange as a capture frame.
pub(crate) fn exchange_frame(req: &HttpRequest, resp: &HttpResponse, job_id: Option<u64>) -> Frame {
    let summary = match &resp.error {
        Some(e) => format!("{} {} → {e}", req.method, req.url),
        None => format!(
            "{} {} → {} in {:.0}ms",
            req.method, req.url, resp.status, resp.latency_ms
        ),
    };

    let mut detail = String::new();
    for (k, v) in &resp.headers {
        detail.push_str(&format!("{k}: {v}\n"));
    }
    if !resp.body.is_empty() {
        detail.push('\n');
        detail.extend(resp.body.chars().take(FRAME_BODY_PREVIEW));
        if resp.body.len() > FRAME_BODY_PREVIEW {
            detail.push_str("\n… (body truncated)");
        }
    }

    let mut frame = Frame::tx("http", "http")
        .remote(&req.url)
        .size(resp.body_bytes)
        .summary(summary)
        .detail(detail)
        .verdict(match (&resp.error, &resp.digest) {
            (Some(_), _) => "failed".to_string(),
            (None, Some(DigestOutcome { challenged: true, .. })) => format!("{} {} · digest after 401", resp.status, resp.status_text),
            (None, _) => format!("{} {}", resp.status, resp.status_text),
        });
    if let Some(id) = job_id {
        frame = frame.job(id);
    }
    frame
}

/// Fire a single request and return the full response for the inspector.
/// A failure to connect or to get an answer is in the response (`error`,
/// `cause`); the error is only for a client that could not be built.
pub async fn request_once(host: Host, req: HttpRequest, jar: Option<Arc<CookieJar>>) -> EngineResult<HttpResponse> {
    let client = build_client(&req, jar)?;
    let resp = execute(&client, &req, &DigestMemory::default()).await;
    if inspect::armed(&host) {
        inspect::publish(&host, exchange_frame(&req, &resp, None));
    }
    Ok(resp)
}

// ---------------------------------------------------------------------------
// Burst / load runner
// ---------------------------------------------------------------------------

#[derive(Clone, Deserialize)]
pub struct BurstConfig {
    #[serde(flatten)]
    pub request: HttpRequest,
    /// At most this many requests in flight at once.
    pub concurrency: u32,
    /// Total requests to send (0 = run for duration_s).
    #[serde(default)]
    pub total: u64,
    /// Optional time limit in seconds (0 = until total reached / stopped).
    #[serde(default)]
    pub duration_s: f64,
    /// Requests started per second, on a fixed schedule (an open model): the
    /// n-th is due at n / rate, however slow the answers. 0 = a closed model,
    /// each worker sending again as soon as it has an answer.
    #[serde(default)]
    pub rate: f64,
    /// Use the HTTP screen's cookie jar, as its single requests do.
    #[serde(default)]
    pub cookies: bool,
}

/// A paced burst slower than this is a probe, not a load; faster, one pacer is
/// not a measurement any more.
pub const MIN_BURST_RATE: f64 = 0.1;
pub const MAX_BURST_RATE: f64 = 100_000.0;
const MAX_CONCURRENCY: u32 = 512;
/// A request still waiting for a free worker this long after its moment is
/// skipped and counted as missed. Sending it late would bunch the load up and
/// hide that the workers could not keep the rate; the margin is wider than a
/// timer tick on any system, so the pacer's own wake-ups never count.
pub(crate) const MISS_AFTER: Duration = Duration::from_millis(50);

#[derive(Clone, Serialize)]
struct BurstProgress {
    job_id: u64,
    ts: u64,
    /// Answered or failed so far.
    sent: u64,
    ok: u64,
    failed: u64,
    /// Due while every worker was busy, and skipped (a paced burst only).
    missed: u64,
    /// Requests per second over the last report's window; in the last report,
    /// over the whole burst.
    rps: f64,
    last_latency_ms: f64,
    min_latency_ms: f64,
    max_latency_ms: f64,
    avg_latency_ms: f64,
    /// Over every request so far, failures included.
    #[serde(flatten)]
    percentiles: Percentiles,
    /// The last report: the burst has ended.
    done: bool,
}

/// What every request of a burst adds to; read by the reporter.
#[derive(Default)]
struct BurstStats {
    sent: AtomicU64,
    ok: AtomicU64,
    failed: AtomicU64,
    missed: AtomicU64,
    last_us: AtomicU64,
    latency: LatencyHistogram,
}

impl BurstStats {
    fn record(&self, resp: &HttpResponse) {
        let micros = (resp.latency_ms * 1000.0) as u64;
        self.latency.record(micros);
        self.last_us.store(micros, Ordering::Relaxed);
        if resp.error.is_none() && resp.ok {
            self.ok.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed.fetch_add(1, Ordering::Relaxed);
        }
        self.sent.fetch_add(1, Ordering::Relaxed);
    }

    fn report(&self, job_id: u64, rps: f64, done: bool) -> BurstProgress {
        BurstProgress {
            job_id,
            ts: now_ms(),
            sent: self.sent.load(Ordering::Relaxed),
            ok: self.ok.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            missed: self.missed.load(Ordering::Relaxed),
            rps,
            last_latency_ms: self.last_us.load(Ordering::Relaxed) as f64 / 1000.0,
            min_latency_ms: self.latency.min_ms(),
            max_latency_ms: self.latency.max_ms(),
            avg_latency_ms: self.latency.mean_ms(),
            percentiles: self.latency.percentiles(),
            done,
        }
    }
}

/// Refuses a burst that cannot run as asked, before anything is sent.
fn check_burst(cfg: &BurstConfig) -> EngineResult<()> {
    let rate = cfg.rate;
    if !rate.is_finite() || rate < 0.0 || (rate > 0.0 && rate < MIN_BURST_RATE) || rate > MAX_BURST_RATE {
        return Err(EngineError::new("http.rate_invalid")
            .with("min", MIN_BURST_RATE)
            .with("max", MAX_BURST_RATE)
            .in_field(Field::new("rate")));
    }
    if !cfg.duration_s.is_finite() || cfg.duration_s < 0.0 {
        return Err(EngineError::new("http.duration_invalid").in_field(Field::new("duration_s")));
    }
    Ok(())
}

pub async fn start_burst(
    host: Host,
    jobs: JobRegistry,
    cfg: BurstConfig,
    jar: Option<Arc<CookieJar>>,
) -> EngineResult<JobInfo> {
    check_burst(&cfg)?;
    let client = build_client(&cfg.request, jar.filter(|_| cfg.cookies))?;
    let concurrency = cfg.concurrency.clamp(1, MAX_CONCURRENCY);

    let id = jobs.next_id();
    let mut info = JobInfo::new(id, "http-burst", format!("HTTP burst {} {}", cfg.request.method, cfg.request.url))
        .with("method", &cfg.request.method)
        .with("url", &cfg.request.url);
    if cfg.rate > 0.0 {
        info = info.with("rate", cfg.rate);
    }

    let host_cl = host.clone();
    let jobs_cl = jobs.clone();
    let handle = tokio::spawn(async move {
        let stats = Arc::new(BurstStats::default());
        // One Digest challenge for every request: answered once, then sent with each.
        let memory = Arc::new(DigestMemory::default());
        let request = Arc::new(cfg.request.clone());
        let client = Arc::new(client);
        let start = Instant::now();

        // Reporter: progress ~10 Hz while the burst runs. Stopping the burst
        // drops this task's future, and the guard aborts the reporter with it.
        let mut guard = TaskGuard::new();
        let reporter = {
            let (host_r, stats_r) = (host_cl.clone(), stats.clone());
            tokio::spawn(async move {
                let (mut prev_sent, mut prev) = (0u64, Instant::now());
                loop {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let sent = stats_r.sent.load(Ordering::Relaxed);
                    let now = Instant::now();
                    let rps = (sent - prev_sent) as f64 / now.duration_since(prev).as_secs_f64().max(1e-6);
                    (prev_sent, prev) = (sent, now);
                    host_r.emit("http://burst-progress", stats_r.report(id, rps, false));
                }
            })
        };
        guard.watch(reporter.abort_handle());

        // One pacer hands out the requests, each its own task holding one of
        // `concurrency` permits; dropping the set (a stop) aborts those in
        // flight. A burst can push thousands of requests per second, so they
        // share one sampling gate into the Inspector.
        let gate = Arc::new(Gate::new(100));
        let permits = Arc::new(Semaphore::new(concurrency as usize));
        let mut flights = JoinSet::new();
        let pace = Pace::new(cfg.rate, cfg.total, cfg.duration_s);
        let mut n = 0u64;
        while let Some(due) = pace.due(n, start) {
            if due > Instant::now() {
                tokio::time::sleep_until(due.into()).await;
            }
            let Ok(permit) = permits.clone().acquire_owned().await else { break };
            if pace.paced() {
                // Waited past the margin for a worker: skip to the first request
                // whose moment is still within it.
                if Instant::now().saturating_duration_since(due) > MISS_AFTER {
                    let caught_up = pace.catch_up(start);
                    stats.missed.fetch_add(caught_up - n, Ordering::Relaxed);
                    n = caught_up;
                    let Some(due) = pace.due(n, start) else { break };
                    if due > Instant::now() {
                        tokio::time::sleep_until(due.into()).await;
                    }
                }
            } else if cfg.duration_s > 0.0 && start.elapsed().as_secs_f64() >= cfg.duration_s {
                break;
            }
            let (client, request, stats, gate, host_w, memory) = (client.clone(), request.clone(), stats.clone(), gate.clone(), host_cl.clone(), memory.clone());
            flights.spawn(async move {
                let resp = execute(&client, &request, &memory).await;
                if inspect::armed(&host_w) && gate.allow() {
                    inspect::publish(&host_w, gate.mark(exchange_frame(&request, &resp, Some(id))));
                }
                stats.record(&resp);
                drop(permit);
            });
            n += 1;
            while flights.try_join_next().is_some() {}
        }
        while flights.join_next().await.is_some() {}

        // The last report waits for no tick: the reporter is gone before it is
        // sent, so nothing older arrives after it, and it rates the whole burst.
        reporter.abort();
        let _ = reporter.await;
        let elapsed = start.elapsed().as_secs_f64().max(1e-6);
        host_cl.emit("http://burst-progress", stats.report(id, stats.sent.load(Ordering::Relaxed) as f64 / elapsed, true));
        drop(guard);

        host_cl.emit(
            "job://ended",
            serde_json::json!({
                "job_id": id,
                "kind": "http-burst",
                "error": serde_json::Value::Null,
            }),
        );
        jobs_cl.finish(id);
    });

    jobs.insert(info.clone(), handle);
    Ok(info)
}

/// When each request of a burst is due. Paced: the n-th at n / rate from the
/// start, so a late wake-up never shifts the ones after it. Closed: now — the
/// permits are what holds it back.
struct Pace {
    rate: f64,
    total: u64,
    duration_s: f64,
}

impl Pace {
    fn new(rate: f64, total: u64, duration_s: f64) -> Self {
        Pace { rate, total, duration_s }
    }

    fn paced(&self) -> bool {
        self.rate > 0.0
    }

    /// The n-th request's moment, or None past the total or the duration.
    fn due(&self, n: u64, start: Instant) -> Option<Instant> {
        if self.total > 0 && n >= self.total {
            return None;
        }
        if !self.paced() {
            return Some(Instant::now());
        }
        let at = n as f64 / self.rate;
        (self.duration_s <= 0.0 || at < self.duration_s).then(|| start + Duration::from_secs_f64(at))
    }

    /// The first request whose moment is less than `MISS_AFTER` ago — every
    /// one before it is missed. Bounded by the total and the duration.
    fn catch_up(&self, start: Instant) -> u64 {
        let behind = Instant::now().saturating_duration_since(start).saturating_sub(MISS_AFTER).as_secs_f64();
        let mut first = (behind * self.rate).ceil() as u64;
        if self.total > 0 {
            first = first.min(self.total);
        }
        if self.duration_s > 0.0 {
            first = first.min((self.duration_s * self.rate).ceil() as u64);
        }
        first
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn burst(rate: f64, duration_s: f64) -> BurstConfig {
        BurstConfig {
            request: HttpRequest { method: "GET".into(), url: "http://127.0.0.1:9/".into(), headers: Vec::new(), body: None, timeout_ms: 1000, auth: Auth::None },
            concurrency: 1,
            total: 1,
            duration_s,
            rate,
            cookies: false,
        }
    }

    #[test]
    fn a_digest_challenge_is_answered_only_where_the_credentials_belong() {
        let url = |text: &str| reqwest::Url::parse(text).unwrap();
        assert!(answerable(&url("http://lab.test/a"), &url("http://lab.test/b?x=1")));
        assert!(answerable(&url("http://lab.test/a"), &url("https://lab.test/a")), "the same host, moved to TLS");
        assert!(!answerable(&url("https://lab.test/a"), &url("http://lab.test/a")), "never down to plain HTTP");
        assert!(!answerable(&url("http://lab.test:8080/"), &url("https://lab.test:8443/")), "other ports: another service");
        assert!(!answerable(&url("http://lab.test/"), &url("http://api.lab.test/")), "another host");
    }

    #[test]
    fn a_rate_is_zero_or_within_the_limits() {
        for rate in [0.0, MIN_BURST_RATE, 250.0, MAX_BURST_RATE] {
            assert!(check_burst(&burst(rate, 0.0)).is_ok(), "{rate}");
        }
        for rate in [-1.0, 0.05, MAX_BURST_RATE + 1.0, f64::NAN, f64::INFINITY] {
            let refused = check_burst(&burst(rate, 0.0)).unwrap_err();
            assert_eq!((refused.code.as_str(), refused.params["max"].as_str()), ("http.rate_invalid", "100000"), "{rate}");
        }
        assert!(check_burst(&burst(0.0, -1.0)).unwrap_err().is("http.duration_invalid"));
    }

    #[test]
    fn a_paced_burst_is_due_on_its_own_schedule() {
        let start = Instant::now();
        let pace = Pace::new(100.0, 0, 1.0);
        assert_eq!(pace.due(0, start), Some(start));
        assert_eq!(pace.due(50, start), Some(start + Duration::from_millis(500)));
        assert_eq!(pace.due(99, start).map(|due| due - start), Some(Duration::from_millis(990)));
        assert_eq!(pace.due(100, start), None, "a second at 100/s is 100 requests");
        assert_eq!(Pace::new(100.0, 10, 0.0).due(10, start), None, "the total ends it as well");
        assert!(Pace::new(0.0, 10, 0.0).due(9, start).is_some() && !Pace::new(0.0, 10, 0.0).paced());
        // Caught up a second in: what was due before 950 ms is missed.
        let earlier = Instant::now() - Duration::from_secs(1);
        let first = Pace::new(100.0, 0, 0.0).catch_up(earlier);
        assert!((95..=97).contains(&first), "{first}");
        assert_eq!(Pace::new(100.0, 40, 0.0).catch_up(earlier), 40, "never past the total");
        assert_eq!(Pace::new(100.0, 0, 0.5).catch_up(earlier), 50, "nor past the duration");
    }
}
