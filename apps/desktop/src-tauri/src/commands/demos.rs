//! Embedded examples open as independent, unsaved projects.
use super::{CommandResult, snapshot};
use crate::state::AppState;
use serde_json::Value;
use soundbit_app::Session;
use tauri::State;

#[tauri::command]
pub(crate) fn open_demo(state: State<'_, AppState>, id: String) -> CommandResult<Value> {
    let source = super::demo_catalog::source(&id).ok_or("Unknown or audio-only demo song")?;
    let song = soundbit_formats::from_project_json(source).map_err(|error| error.to_string())?;
    let mut session = state.session();
    *session = Session::new(song);
    snapshot(&session)
}
