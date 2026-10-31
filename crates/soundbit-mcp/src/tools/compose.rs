//! Native music-theory composition tools shared with the desktop.
#![allow(
    clippy::unused_self,
    reason = "MCP tool router requires instance methods"
)]
use super::{ToolResult, attempt, refuse, reply};
use crate::{SoundBitServer, view::song_view};
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use soundbit_app::{
    Command, Origin,
    command::PatternCommand,
    compose::{ComposeConfig, generate, randomize, transform_notes},
    model::PatternId,
};

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct RandomInput {
    /// Discovery seed. Same seed and input produce the same recipe.
    seed: u32,
    /// Optional current recipe: change only its instruments, preserving its musical seed.
    /// Omit to discover a completely new song recipe with harmonic guardrails.
    instruments_for: Option<ComposeConfig>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ComposeInput {
    /// Composition recipe. Use {} for defaults; `composition_defaults` documents all controls.
    config: ComposeConfig,
    /// Editor id from `song_get`. Prevents replacing a different newly opened project.
    expected_editor_id: u64,
    /// Revision from `song_get`. Prevents overwriting concurrent human edits.
    expected_revision: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct DevelopInput {
    /// Existing pattern id; changes all arrangement uses, with undo available.
    pattern_id: u32,
    /// Shift in scale degrees, -14 to 14. Zero leaves the register unchanged.
    #[serde(default)]
    degrees: i16,
    /// Reflect the contour about the first note before shifting.
    #[serde(default)]
    invert: bool,
}

#[tool_router(router = compose_tools, vis = "pub(crate)")]
impl SoundBitServer {
    #[tool(
        description = "Discover a random song recipe with style-compatible scale, tempo, density and role-appropriate native sounds. Keeps motifs, voice leading and cadences enabled. Supply instruments_for to change only a recipe's sounds. Does not change the open song; apply the returned recipe with song_compose.",
        annotations(read_only_hint = true)
    )]
    fn composition_random(&self, Parameters(input): Parameters<RandomInput>) -> ToolResult {
        reply(&attempt!(serde_json::to_value(randomize(
            input.seed,
            input.instruments_for.as_ref()
        ))))
    }

    #[tool(
        description = "Returns CTG Studio's default composition recipe and guidance. All native presets are available through preset_list. Composition is seeded and editable; preview with composition_preview, apply with song_compose.",
        annotations(read_only_hint = true)
    )]
    fn composition_defaults(&self) -> ToolResult {
        reply(
            &json!({"config":ComposeConfig::default(),"engine":"CTG Studio native synthesis",
            "composer_version":3,
            "features":["weighted progressions","rhythmic cells and answering phrases","melodic look-ahead","phrase breathing and developed returns","recurring bass/drum pocket","Euclidean bass","arpeggios","section contrast","cadences","optional extensions, mixture and secondary dominants","native swing"],
            "workflow":"song_get -> composition_preview -> song_compose (editor_id + revision) -> song_save / export_wav; history_undo restores the previous song"}),
        )
    }

    #[tool(
        description = "Plans a full composition and returns its bar-by-bar chords, voicings and attack counts without changing the open song. Same config and seed produce the same project in desktop, CLI and MCP.",
        annotations(read_only_hint = true)
    )]
    fn composition_preview(&self, Parameters(config): Parameters<ComposeConfig>) -> ToolResult {
        let composition = attempt!(generate(&config));
        reply(&attempt!(serde_json::to_value(composition.report)))
    }

    #[tool(
        description = "Replaces the open song with a fully editable CTG composition in one undo step. Uses CTG's native and ZzFX preset instruments with phrase development and melodic look-ahead. Supply editor_id and revision from `song_get`. Does not save or overwrite a file; history_undo restores the previous complete song."
    )]
    fn song_compose(&self, Parameters(input): Parameters<ComposeInput>) -> ToolResult {
        let composition = attempt!(generate(&input.config));
        let mut session = self.session();
        if session.editor().id() != input.expected_editor_id
            || session.editor().revision() != input.expected_revision
        {
            return refuse(
                "song changed while composing; call song_get and review before retrying with its editor_id and revision",
            );
        }
        attempt!(session.editor_mut().execute(
            Command::ReplaceSong(Box::new(composition.song)),
            Origin::Agent
        ));
        reply(
            &json!({"song":song_view(&session,self.workspace()),"composition":composition.report}),
        )
    }

    #[tool(
        description = "Develops an existing motif with scale-degree transposition and/or contour inversion about its first note. Preserves timing, velocity, detuning and glides; one undo step. For borrowed/chromatic phrases use pattern_transpose to preserve exact semitone intervals."
    )]
    fn pattern_develop(&self, Parameters(input): Parameters<DevelopInput>) -> ToolResult {
        if !(-14..=14).contains(&input.degrees) {
            return refuse("degrees must be -14 to 14");
        }
        let mut session = self.session();
        let id = PatternId::new(input.pattern_id);
        let Some(pattern) = session.song().pattern(id) else {
            return refuse("unknown pattern_id");
        };
        let notes = attempt!(transform_notes(
            pattern.notes(),
            session.song().scale(),
            input.degrees,
            input.invert
        ));
        attempt!(
            session
                .editor_mut()
                .execute(PatternCommand::SetNotes(id, notes), Origin::Agent)
        );
        reply(&song_view(&session, self.workspace()))
    }
}
