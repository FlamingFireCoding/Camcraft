//! Provenance: issue #3781 setup workflow and public provider account pages.

use std::time::Duration;

use camcraft_app::ai::{KeyName, by_id};
use egui::Ui;

use super::settings::{Settings, SettingsAction, SettingsRequest};

pub(crate) fn draw(ui: &mut Ui, settings: &mut Settings, preset: &str, model: &str, offline: bool) {
    let _span = crate::frame_cost::span("ui/ai_chat.settings");
    settings.sync(SettingsRequest {
        preset_id: preset.into(),
        model: model.into(),
        offline_only: offline,
    });
    if settings.busy() {
        ui.ctx().request_repaint_after(Duration::from_millis(50));
    }
    ui.label(&settings.status);
    let name = by_id(preset).and_then(|p| p.key);
    ui.horizontal_wrapped(|ui| {
        if name.is_some() && ui.button("Set API key...").clicked() {
            settings.close_editor();
            settings.editing = true;
        }
        if ui
            .add_enabled(!settings.busy(), egui::Button::new("Test"))
            .clicked()
        {
            settings.start(SettingsAction::Test);
        }
        if name.is_some()
            && ui
                .add_enabled(!settings.busy(), egui::Button::new("Remove"))
                .clicked()
        {
            settings.close_editor();
            settings.start(SettingsAction::Remove);
        }
    });
    if let Some(name) = name {
        egui::CollapsingHeader::new("API key help").show(ui, |ui| {
            ui.label(
                "An API key is a secret password that lets CamCraft use your provider account.",
            );
            ui.label(
                "Ready means a key was found. Test sends a small request and can cost credits.",
            );
            provider_help(ui, name);
            keychain_help(ui);
        });
        draw_editor(ui, settings, name);
    } else {
        egui::CollapsingHeader::new("Ollama setup").show(ui, |ui| {
            ui.hyperlink_to("Download Ollama", "https://ollama.com/download");
            ui.label("Install and start Ollama. In Terminal, run ollama pull followed by the Model name above. Then click Test.");
            ui.hyperlink_to("Install a model", "https://docs.ollama.com/quickstart");
        });
    }
}

fn draw_editor(ui: &mut Ui, settings: &mut Settings, name: KeyName) {
    if !settings.editing {
        return;
    }
    let mut open = true;
    egui::Window::new("Set API key...")
        .id(egui::Id::new("ai_key_editor"))
        .open(&mut open)
        .resizable(true)
        .default_width(440.0)
        .vscroll(true)
        .show(ui.ctx(), |ui| {
            ui.label(format!("{} API key", name.label()));
            ui.label(
                "Paste a new key. Save replaces the stored key. CamCraft never shows a saved key.",
            );
            let label = ui.label("API key");
            ui.add(egui::TextEdit::singleline(&mut settings.draft).password(true))
                .labelled_by(label.id);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!settings.busy(), egui::Button::new("Save key"))
                    .clicked()
                {
                    settings.save();
                }
                if ui.button("Cancel key entry").clicked() {
                    settings.close_editor();
                }
            });
            ui.label(&settings.status);
            provider_help(ui, name);
            keychain_help(ui);
        });
    if !open {
        settings.close_editor();
    }
}

fn provider_help(ui: &mut Ui, name: KeyName) {
    let (keys, billing, limits) = match name {
        KeyName::Anthropic => (
            "https://platform.claude.com/settings/keys",
            "https://platform.claude.com/settings/billing",
            "https://platform.claude.com/settings/spend-limits",
        ),
        KeyName::OpenAi => (
            "https://platform.openai.com/api-keys",
            "https://platform.openai.com/settings/organization/billing/overview",
            "https://platform.openai.com/settings/organization/limits",
        ),
        KeyName::Gemini => (
            "https://aistudio.google.com/api-keys",
            "https://ai.google.dev/gemini-api/docs/billing",
            "https://aistudio.google.com/usage",
        ),
    };
    ui.hyperlink_to("Get or manage an API key", keys);
    match name {
        KeyName::Anthropic => {
            ui.label("1. Sign in to the Anthropic console. Open Settings > API keys.");
            ui.label("2. Click Create key. Name it, for example CamCraft on my Mac.");
            ui.label("3. Copy the key now. It starts with sk-ant- and appears only once.");
        }
        KeyName::OpenAi => {
            ui.label("1. Sign in to the OpenAI platform. Open API keys.");
            ui.label("2. Create a new secret key. Name it CamCraft on my Mac.");
            ui.label("3. Copy the secret key now. You cannot view it again.");
        }
        KeyName::Gemini => {
            ui.label("1. Sign in to Google AI Studio. Open API keys.");
            ui.label("2. Create an API key and choose a project. Name the key for this computer.");
            ui.label("3. Copy the key from AI Studio.");
        }
    }
    ui.label("4. Paste the key above and click Save key.");
    ui.label(
        "5. Keep it secret, like a password. To revoke it, delete it on the provider's key page.",
    );
    ui.label("6. Cloud use can cost credits. Check billing and spending controls before Send.");
    ui.horizontal_wrapped(|ui| {
        ui.hyperlink_to("Billing / credits", billing);
        ui.hyperlink_to("Usage / spending controls", limits);
    });
}

fn keychain_help(ui: &mut Ui) {
    ui.label("The key stays in the OS keychain, outside the project. An environment key takes priority. Remove deletes only the stored key.");
    ui.label("On macOS, the first key read can ask for the login keychain password. This is normally your Mac login password.");
    ui.label("Choose Always Allow for CamCraft. Allow permits one read and asks again on the next Send. Never enter this password in CamCraft.");
    ui.label("CamCraft is not code-signed. macOS can ask again after an update; this is expected.");
    ui.label("Windows uses Credential Manager and may ask for access. Linux uses Secret Service and may ask you to unlock the keyring. These systems can also work without a prompt.");
}
