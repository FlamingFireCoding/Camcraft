//! The native HTTP client for AI providers (#1945). Every request passes
//! the endpoint laws (`endpoint::admit`) before anything resolves, then
//! the resolver re-checks the addresses: offline-only, or plain `http`,
//! reaches loopback addresses alone. No redirect is followed (a local
//! server could otherwise bounce a call off the machine), no proxy is
//! read from the environment, and no error message carries the key.
//!
//! Provenance: our own policy over the `ureq` crate's public API; the
//! auth headers are each provider's documented ones (`x-api-key`,
//! `Authorization: Bearer`, `x-goog-api-key`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::Value;

use crate::mcp::provider::Provider;

use super::endpoint::{self, Endpoint, Scheme};
use super::probe::{self, Capabilities};
use super::secret::ApiKey;
use super::wire::{self, ChatRequest};

/// How long one call may take: connect, then the whole answer.
#[derive(Clone, Copy, Debug)]
pub struct Timeouts {
    pub connect: Duration,
    pub total: Duration,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self { connect: Duration::from_secs(10), total: Duration::from_secs(300) }
    }
}

/// The client: the offline switch and the timeouts. An agent is built
/// per request, so each carries its own loopback rule.
#[derive(Clone, Copy, Debug)]
pub struct Client {
    pub offline_only: bool,
    pub timeouts: Timeouts,
}

/// The longest provider error text kept in a message.
const ERROR_TEXT: usize = 400;

impl Client {
    #[must_use]
    pub fn new(offline_only: bool) -> Self {
        Self { offline_only, timeouts: Timeouts::default() }
    }

    fn agent(&self, loopback_only: bool) -> ureq::Agent {
        ureq::AgentBuilder::new()
            .redirects(0)
            .try_proxy_from_env(false)
            .timeout_connect(self.timeouts.connect)
            .timeout(self.timeouts.total)
            .user_agent(concat!("camcraft/", env!("CARGO_PKG_VERSION")))
            .resolver(move |netloc: &str| -> std::io::Result<Vec<std::net::SocketAddr>> {
                let addrs = std::net::ToSocketAddrs::to_socket_addrs(netloc)?.collect();
                endpoint::admit_addrs(addrs, loopback_only)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))
            })
            .build()
    }

    /// POST `body` as JSON to `path` under `ep`, with `auth` as the auth
    /// header when given, and read a JSON answer. A non-2xx status is an
    /// error naming the status and the provider's message.
    pub fn post_json(
        &self,
        ep: &Endpoint,
        path: &str,
        headers: &[(String, String)],
        auth: Option<(&'static str, String)>,
        body: &Value,
    ) -> Result<Value, String> {
        endpoint::admit(ep, self.offline_only)?;
        let loopback_only = self.offline_only || ep.scheme == Scheme::Http;
        let url = ep.url(path);
        let mut req = self.agent(loopback_only).post(&url).set("content-type", "application/json");
        for (k, v) in headers {
            req = req.set(k, v);
        }
        if let Some((name, value)) = &auth {
            req = req.set(name, value);
        }
        let resp = match req.send_string(&body.to_string()) {
            Ok(r) => r,
            Err(ureq::Error::Status(code, r)) => {
                let text = r.into_string().unwrap_or_default();
                return Err(format!("{} answered HTTP {code}: {}", ep.host, provider_message(&text)));
            }
            Err(ureq::Error::Transport(t)) => {
                return Err(with_detail(format!("{} could not be reached: {}", ep.host, t.kind()), &t));
            }
        };
        let status = resp.status();
        if !(200..300).contains(&status) {
            return Err(format!("{} answered HTTP {status} (redirects are not followed)", ep.host));
        }
        let text = resp.into_string().map_err(|e| format!("{} answer not read: {e}", ep.host))?;
        serde_json::from_str(&text).map_err(|e| format!("{} answer is not JSON ({e})", ep.host))
    }
}

/// A transport error's own message, minus anything after the URL (ureq
/// prints the request URL, which never holds a key: keys ride headers).
fn with_detail(base: String, t: &ureq::Transport) -> String {
    match t.message() {
        Some(m) => format!("{base} ({m})"),
        None => base,
    }
}

/// The provider's error message if it sent one, else the start of the body.
fn provider_message(text: &str) -> String {
    let parsed: Option<Value> = serde_json::from_str(text).ok();
    let msg = parsed
        .as_ref()
        .and_then(|v| v["error"]["message"].as_str().or_else(|| v["error"].as_str()).map(str::to_owned))
        .unwrap_or_else(|| text.to_owned());
    msg.chars().take(ERROR_TEXT).collect()
}

/// The auth header for `provider`, per its documentation.
#[must_use]
pub fn auth_header(provider: Provider, key: &ApiKey) -> (&'static str, String) {
    match provider {
        Provider::Anthropic => ("x-api-key", key.expose().to_owned()),
        Provider::OpenAiChat | Provider::OpenAiResponses => ("authorization", format!("Bearer {}", key.expose())),
        Provider::Gemini => ("x-goog-api-key", key.expose().to_owned()),
    }
}

/// Send one chat step and return the provider's response envelope.
pub fn send_chat(
    client: &Client,
    ep: &Endpoint,
    req: &ChatRequest<'_>,
    key: Option<&ApiKey>,
    cancel: &AtomicBool,
) -> Result<Value, String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("cancelled".into());
    }
    let w = wire::build(req)?;
    let auth = key.map(|k| auth_header(req.provider, k));
    let out = client.post_json(ep, &w.path, &w.headers, auth, &w.body)?;
    if cancel.load(Ordering::Relaxed) {
        return Err("cancelled".into());
    }
    Ok(out)
}

/// Ask an Ollama server what `model` can do. `ep` is the chat base
/// (`.../v1`); the probe goes to `/api/show` on the same host and port.
pub fn probe_ollama(client: &Client, ep: &Endpoint, model: &str) -> Result<Capabilities, String> {
    wire::check_model(model)?;
    let root = ep.with_base_path("");
    let answer = client.post_json(&root, "/api/show", &[], None, &probe::show_request(model))?;
    probe::parse_show(&answer)
}
