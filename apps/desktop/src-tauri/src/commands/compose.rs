//! Human access to the same composition engine that MCP and CLI use.
use super::{CommandResult, snapshot};
use crate::state::AppState;
use serde_json::{Value, json};
use soundbit_app::{
    Command, Origin,
    command::PatternCommand,
    compose::{ComposeConfig, generate, randomize, transform_notes},
    model::PatternId,
};
use tauri::State;

#[tauri::command]
pub(crate) fn composition_defaults() -> ComposeConfig {
    ComposeConfig::default()
}

#[tauri::command]
pub(crate) fn composition_random(
    seed: u32,
    instruments_for: Option<ComposeConfig>,
) -> ComposeConfig {
    randomize(seed, instruments_for.as_ref())
}

#[tauri::command]
pub(crate) fn composition_preview(config: ComposeConfig) -> CommandResult<Value> {
    serde_json::to_value(generate(&config)?.report).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn compose_song(
    state: State<'_, AppState>,
    config: ComposeConfig,
    editor_id: u64,
    revision: u64,
) -> CommandResult<Value> {
    let composition = generate(&config)?;
    let mut session = state.session();
    if session.editor().id() != editor_id || session.editor().revision() != revision {
        return Err("The song changed. Review the current song, then generate again.".into());
    }
    session
        .editor_mut()
        .execute(
            Command::ReplaceSong(Box::new(composition.song)),
            Origin::User,
        )
        .map_err(|e| e.to_string())?;
    Ok(json!({"state":snapshot(&session)?,"report":composition.report}))
}

#[tauri::command]
pub(crate) fn develop_pattern(
    state: State<'_, AppState>,
    pattern_id: u32,
    degrees: i16,
    invert: bool,
) -> CommandResult<Value> {
    if !(-14..=14).contains(&degrees) {
        return Err("Scale shift must be -14 to 14.".into());
    }
    let mut session = state.session();
    let id = PatternId::new(pattern_id);
    let pattern = session
        .song()
        .pattern(id)
        .ok_or("Select a pattern first.")?;
    let notes = transform_notes(pattern.notes(), session.song().scale(), degrees, invert)?;
    session
        .editor_mut()
        .execute(PatternCommand::SetNotes(id, notes), Origin::User)
        .map_err(|e| e.to_string())?;
    snapshot(&session)
}
