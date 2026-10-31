//! The preset library for the instrument panel.

use serde::Serialize;
use soundbit_app::PresetLibrary;
use soundbit_app::model::VoiceKind;

use super::CommandResult;
use crate::state::AppState;
use serde_json::{Value, json};
use soundbit_app::{
    model::{Instrument, InstrumentId},
    save_user_preset, user_preset_dir,
};
use tauri::State;

#[tauri::command]
pub(crate) fn preset_get(preset: String) -> CommandResult<Instrument> {
    PresetLibrary::available()?
        .find(&preset)
        .map(|p| p.instrument.clone())
        .ok_or_else(|| format!("Unknown preset {preset}"))
}

#[tauri::command]
pub(crate) fn preset_library_info() -> CommandResult<Value> {
    Ok(json!({"directory": user_preset_dir()?, "warnings": PresetLibrary::available()?.warnings()}))
}

#[tauri::command]
pub(crate) fn instrument_save_preset(state: State<'_, AppState>, id: u32) -> CommandResult<Value> {
    let instrument = state
        .session()
        .song()
        .instrument(InstrumentId::new(id))
        .cloned()
        .ok_or("Select an instrument first")?;
    let (preset, path) = save_user_preset(&user_preset_dir()?, &instrument)?;
    Ok(json!({"id":preset.id, "path":path, "instrument":instrument}))
}

/// A preset as the front end lists it. Mirrored by `PresetInfo` in `src/lib/ipc/types.ts`.
#[derive(Debug, Serialize)]
pub(crate) struct PresetInfo {
    id: String,
    name: String,
    category: String,
    description: String,
    voice: VoiceKind,
}

#[tauri::command]
pub(crate) fn list_presets() -> CommandResult<Vec<PresetInfo>> {
    let library = PresetLibrary::available()?;
    Ok(library
        .all()
        .iter()
        .map(|preset| PresetInfo {
            id: preset.id.clone(),
            name: preset.instrument.name.clone(),
            category: preset.category.clone(),
            description: preset.description.clone(),
            voice: preset.instrument.voice.kind(),
        })
        .collect())
}
