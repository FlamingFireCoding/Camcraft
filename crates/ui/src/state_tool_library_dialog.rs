//! #3892 (U1 of #3663): the tool library window's view state. The window
//! is open while `State::tool_library` is `Some`. Like the tool shape
//! window, it is ephemeral view state: never a command, never saved,
//! never undoable. The edits it starts ("Add", Import) are commands.
//! The Tool type and Name filters stay in `State::tool_catalog_kind` and
//! `State::tool_catalog_name`, so they hold between two openings.
//! Provenance: our own view state, after the tool shape window's
//! `Option` pattern; no HSMWorks material involved.

use super::State;

/// Which tools the list shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolSource {
    /// The tools of the open document.
    Document,
    /// The built-in catalog; each row can be added to the document.
    #[default]
    Catalog,
}

/// The open tool library window.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToolLibraryView {
    pub source: ToolSource,
}

impl State {
    /// Open the tool library window. An open window keeps its view; a new
    /// one starts on the document's tools, or on the catalog when the
    /// document has no tool yet.
    pub(crate) fn open_tool_library(&mut self) {
        if self.tool_library.is_some() {
            return;
        }
        let source = if self.tool_names().is_empty() {
            ToolSource::Catalog
        } else {
            ToolSource::Document
        };
        self.tool_library = Some(ToolLibraryView { source });
    }

    pub(crate) fn close_tool_library(&mut self) {
        self.tool_library = None;
    }
}
