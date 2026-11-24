//! Provenance: issue #3781; transient credential controls over our existing provider APIs.

use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use camcraft_app::ai::ApiKey;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SettingsRequest {
    pub preset_id: String,
    pub model: String,
    pub offline_only: bool,
}

pub(crate) enum SettingsAction {
    Check,
    Save(ApiKey),
    Remove,
    Test,
}

pub(crate) trait SettingsBackend: Send + Sync {
    fn run(&self, request: &SettingsRequest, action: SettingsAction) -> Result<String, String>;
}

pub(crate) struct Settings {
    pub backend: Arc<dyn SettingsBackend>,
    pub status: String,
    pub editing: bool,
    pub draft: String,
    request: Option<SettingsRequest>,
    pending: Option<mpsc::Receiver<Result<String, String>>>,
    needs_check: bool,
    check_after: Option<Instant>,
    tested_model: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            status: "Checking provider...".into(),
            editing: false,
            draft: String::new(),
            request: None,
            pending: None,
            needs_check: false,
            check_after: None,
            tested_model: None,
        }
    }
}

impl Settings {
    /// Never retain a password draft when the editor closes.
    pub fn close_editor(&mut self) {
        self.draft.clear();
        self.editing = false;
    }

    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }

    pub fn has_work(&self) -> bool {
        self.busy() || self.needs_check
    }

    /// Keep one job in flight. Cloud readiness depends on credentials, not the model.
    pub fn sync(&mut self, request: SettingsRequest) {
        self.sync_at(request, Instant::now());
    }

    pub(crate) fn sync_at(&mut self, request: SettingsRequest, now: Instant) {
        if self.request.as_ref() != Some(&request) {
            let same_provider = self.request.as_ref().is_some_and(|previous| {
                previous.preset_id == request.preset_id
                    && previous.offline_only == request.offline_only
            });
            let cloud = camcraft_app::ai::by_id(&request.preset_id)
                .is_some_and(|preset| preset.key.is_some());
            self.close_editor();
            self.request = Some(request);
            if !same_provider || !cloud {
                self.status = "Checking provider...".into();
                self.needs_check = true;
                // Coalesce local Model keystrokes. Provider switches check immediately.
                self.check_after = same_provider.then(|| now + Duration::from_millis(500));
            }
        }
        if let Some(rx) = &self.pending {
            let result = match rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Disconnected) => Some(Err(
                    "The provider check stopped. Click Test to try again.".into(),
                )),
                Err(mpsc::TryRecvError::Empty) => None,
            };
            if let Some(result) = result {
                if !self.needs_check {
                    let current_model = self.request.as_ref().map(|request| &request.model);
                    if self
                        .tested_model
                        .as_ref()
                        .is_none_or(|model| Some(model) == current_model)
                    {
                        self.status = result.unwrap_or_else(|message| message);
                    } else {
                        self.status = "Model changed. Click Test to test this model.".into();
                    }
                }
                self.tested_model = None;
                self.pending = None;
            }
        }
        if !self.busy()
            && self.needs_check
            && self.check_after.is_none_or(|deadline| now >= deadline)
        {
            // Starting the queued check replaces a stale status before the UI draws it.
            self.needs_check = false;
            self.start(SettingsAction::Check);
        }
    }

    pub fn save(&mut self) {
        match ApiKey::new(&self.draft) {
            Ok(key) => {
                self.close_editor();
                self.start(SettingsAction::Save(key));
            }
            Err(message) => self.status = message,
        }
    }

    pub fn start(&mut self, action: SettingsAction) {
        if self.busy() {
            return;
        }
        let Some(request) = self.request.clone() else {
            return;
        };
        if matches!(action, SettingsAction::Test)
            && request.offline_only
            && camcraft_app::ai::by_id(&request.preset_id).is_some_and(|p| p.key.is_some())
        {
            self.status =
                "Offline only blocks the cloud test. Clear Offline only to test this key.".into();
            return;
        }
        let tested_model = matches!(&action, SettingsAction::Test).then(|| request.model.clone());
        let backend = Arc::clone(&self.backend);
        let (tx, rx) = mpsc::channel();
        let started = request.clone();
        match std::thread::Builder::new()
            .name("ai-provider-settings".into())
            .spawn(move || {
                let result = backend.run(&started, action);
                let _ = tx.send(result);
            }) {
            Ok(_) => {
                self.status = "Checking provider...".into();
                self.pending = Some(rx);
                self.tested_model = tested_model;
                self.needs_check = false;
                self.check_after = None;
            }
            Err(_) => {
                self.status = "The provider check could not start. Click Test to retry.".into()
            }
        }
    }
}

fn default_backend() -> Arc<dyn SettingsBackend> {
    #[cfg(all(not(test), not(target_arch = "wasm32")))]
    {
        Arc::new(super::settings_native::NativeSettings)
    }
    #[cfg(any(test, target_arch = "wasm32"))]
    {
        Arc::new(UnavailableSettings)
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
struct UnavailableSettings;

#[cfg(any(test, target_arch = "wasm32"))]
impl SettingsBackend for UnavailableSettings {
    fn run(&self, _: &SettingsRequest, _: SettingsAction) -> Result<String, String> {
        #[cfg(test)]
        {
            Ok("No key - Set API key...".into())
        }
        #[cfg(not(test))]
        {
            Err("Provider setup needs the desktop app.".into())
        }
    }
}
