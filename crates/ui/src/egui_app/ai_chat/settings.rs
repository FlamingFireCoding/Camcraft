//! Provenance: issue #3781; transient credential controls over our existing provider APIs.

use std::sync::{Arc, mpsc};

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

    /// Keep one job in flight. Discard a stale result after a provider/model switch.
    pub fn sync(&mut self, request: SettingsRequest) {
        if self.request.as_ref() != Some(&request) {
            self.close_editor();
            self.request = Some(request);
            self.status = "Checking provider...".into();
            self.needs_check = true;
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
                self.status = result.unwrap_or_else(|message| message);
                self.pending = None;
            }
        }
        if !self.busy() && self.needs_check {
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
