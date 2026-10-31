//! The preset library.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use soundbit_app::{PresetLibrary, model::InstrumentId, save_user_preset, user_preset_dir};

use super::{ToolResult, attempt, refuse, reply};
use crate::SoundBitServer;

/// Which presets to list.
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub(crate) struct PresetListInput {
    /// Only list this category, e.g. "strings", "brass", "drums", "lead", "bass". Lists
    /// everything when unset.
    category: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct PresetSaveInput {
    /// Instrument in the current project to save as a reusable sound.
    instrument_id: u32,
}

#[tool_router(router = preset_tools, vis = "pub(crate)")]
impl SoundBitServer {
    #[tool(
        description = "Saves a new reusable copy of an instrument to CTG Studio's user preset library (Music/CTG Studio/presets, or CTG_PRESET_DIR). Never overwrites existing presets or changes the song. The saved id is available to preset_list, instrument_add_preset and composition recipes in desktop, CLI and AI."
    )]
    fn preset_save(&self, Parameters(input): Parameters<PresetSaveInput>) -> ToolResult {
        let Some(instrument) = self
            .session()
            .song()
            .instrument(InstrumentId::new(input.instrument_id))
            .cloned()
        else {
            return refuse("Unknown instrument_id");
        };
        let directory = attempt!(user_preset_dir());
        let (preset, path) = attempt!(save_user_preset(&directory, &instrument));
        reply(&json!({"preset":preset, "path":path}))
    }

    #[tool(
        description = "Lists the instrument presets: id, name, category, voice type and what \
                       each sounds like and suits (including which notes to play drums on). Add \
                       one with instrument_add_preset.",
        annotations(read_only_hint = true)
    )]
    #[allow(clippy::unused_self, reason = "rmcp tools are methods")]
    fn preset_list(&self, Parameters(input): Parameters<PresetListInput>) -> ToolResult {
        let library = attempt!(PresetLibrary::available());
        let mut categories: Vec<&str> = library
            .all()
            .iter()
            .map(|preset| preset.category.as_str())
            .collect();
        categories.sort_unstable();
        categories.dedup();
        let presets: Vec<_> = library
            .all()
            .iter()
            .filter(|preset| {
                input
                    .category
                    .as_deref()
                    .is_none_or(|category| preset.category == category)
            })
            .map(|preset| {
                json!({
                    "id": preset.id,
                    "name": preset.instrument.name,
                    "category": preset.category,
                    "voice": preset.instrument.voice.kind(),
                    "description": preset.description,
                })
            })
            .collect();
        reply(&json!({"categories": categories, "presets": presets, "warnings":library.warnings()}))
    }
}
