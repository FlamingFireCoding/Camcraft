//! Provenance: our keychain/HTTP APIs and provider documentation; issue #3781.

use std::sync::atomic::AtomicBool;
use std::time::Duration;

use camcraft_app::ai::http::{Client, Timeouts, probe_ollama, send_chat};
use camcraft_app::ai::wire::{ChatMessage, ChatRequest};
use camcraft_app::ai::{ApiKey, Endpoint, KeySource, by_id, keychain, secret};

use super::settings::{SettingsAction, SettingsBackend, SettingsRequest};

pub(crate) struct NativeSettings;

impl SettingsBackend for NativeSettings {
    fn run(&self, request: &SettingsRequest, action: SettingsAction) -> Result<String, String> {
        let endpoint = by_id(&request.preset_id)
            .or_else(|| {
                request
                    .preset_id
                    .starts_with("local-")
                    .then(|| by_id("ollama"))
                    .flatten()
            })
            .ok_or("This provider has no configured endpoint.")?;
        let ep = Endpoint::parse(endpoint.base_url)?;
        let client = Client {
            offline_only: request.offline_only,
            timeouts: Timeouts {
                connect: Duration::from_secs(3),
                total: Duration::from_secs(15),
            },
        };
        let Some(name) = endpoint.key else {
            return match probe_ollama(&client, &ep, &request.model) {
                Ok(_) => Ok(format!("Ready (local model {})", request.model)),
                Err(e) if e.contains("HTTP 404") || e.contains("not found") => Err(format!(
                    "model {} not installed - install it with Ollama",
                    request.model
                )),
                Err(_) => Err(format!(
                    "Ollama not running at {} - start Ollama and check the model",
                    endpoint.base_url
                )),
            };
        };
        match action {
            SettingsAction::Save(ref key) => keychain::store(name, key)
                .map_err(|_| "The key could not be saved. Unlock the OS keychain and try again.")?,
            SettingsAction::Remove => keychain::forget(name).map_err(
                |_| "The key could not be removed. Unlock the OS keychain and try again.",
            )?,
            SettingsAction::Check | SettingsAction::Test => {}
        }
        let found = secret::lookup(name, |v| std::env::var(v).ok(), keychain::load)
            .map_err(|_| "The key could not be read. Check the environment variable or unlock the OS keychain.")?;
        let Some((key, source)) = found else {
            return Ok("No key - Set API key...".into());
        };
        let source = match source {
            KeySource::Environment => "environment; overrides the keychain",
            KeySource::Keychain => "keychain",
        };
        if matches!(action, SettingsAction::Test) {
            test_key(&client, &ep, endpoint.provider, &request.model, &key)?;
            return Ok(format!("Ready (key from {source}) - Test passed"));
        }
        Ok(format!("Ready (key from {source})"))
    }
}

fn test_key(
    client: &Client,
    ep: &Endpoint,
    provider: camcraft_app::mcp::provider::Provider,
    model: &str,
    key: &ApiKey,
) -> Result<(), String> {
    let messages = [ChatMessage {
        role: "user".into(),
        text: "Reply OK.".into(),
        image: None,
        raw: None,
    }];
    let request = ChatRequest {
        provider,
        model,
        system: Some("This is a connection test. Do not call tools.".into()),
        messages: &messages,
        tools: &[],
        max_tokens: 16,
    };
    // Do not forward remote error text to the UI: it can contain credentials.
    send_chat(client, ep, &request, Some(key), &AtomicBool::new(false))
        .map(|_| ())
        .map_err(|_| {
            "Test failed. Check the key, model, billing credits, and network, then try again."
                .into()
        })
}
