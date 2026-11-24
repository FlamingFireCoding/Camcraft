## #3781 - Set up AI provider keys inside the app

**What was decided:** Add a password entry field, save, remove, and test controls to AI Chat. Use the existing app keychain functions and the existing environment-first lookup. Saving replaces the provider's stored key. Remove deletes only the keychain entry. Clear the entry field after save, cancel, and provider changes. Do not return a stored secret to the UI.

Run credential operations and readiness checks in background work. Show the key source as soon as the provider is selected. A ready status means that a key was found. An explicit Test sends a small request to the selected model and can incur a provider charge. Check local Ollama readiness and model availability as well.

Explain API keys, provider setup, billing, environment precedence, and OS permission prompts beside the controls and in the user guide. Keep the PR a draft until the reporter confirms the macOS hand-test.

**Why:** A desktop user must be able to set up a cloud provider without Terminal or an app restart. Reusing the existing storage preserves its service and account names. Background operations keep keychain prompts and network requests from blocking the UI. Distinguishing a found key from a successful test avoids a false claim about account access or credit.

**Considered and dropped:** Environment-only setup does not work well for an app started from the Dock. Storing a key in preferences or the project would expose it. Displaying a saved key would create an unnecessary disclosure. Sending cloud requests automatically on provider selection could cause charges without an explicit action. A separate credential store would duplicate the existing platform support.

**Cost to change later:** Provider instructions and links can change without a storage migration. Changing the keychain service or account names requires a migration for existing keys. Changing lookup precedence requires updates to the transport, readiness checks, tests, and guide.
