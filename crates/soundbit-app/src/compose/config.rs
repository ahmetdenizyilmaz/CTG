//! Settings shared by the desktop composer, CLI and MCP.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use soundbit_core::{PitchClass, Scale, ScaleKind};

/// Arrangement and groove character.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Vibe {
    /// Tuneful, medium energy.
    Adventure,
    /// Fast, driving rhythm.
    Battle,
    /// Sparse atmosphere.
    Dungeon,
    /// Welcoming melodic theme.
    TitleScreen,
    /// Dense dramatic arrangement.
    Boss,
    /// Syncopated groove with a reflective bridge.
    #[default]
    Funk,
}

/// Number of four-bar sections.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Form {
    /// Four sections, 16 bars.
    Short,
    /// Eight sections, 32 bars.
    #[default]
    Standard,
    /// Twelve sections, 48 bars.
    Extended,
}

/// Bass onset selection, always respecting the section's attack budget.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BassRhythm {
    /// Choose a template or Euclidean rhythm per section.
    #[default]
    Mixed,
    /// Preserve a syncopated or driving template.
    Template,
    /// Distribute attacks evenly and rotate toward the kick.
    Euclidean,
}

/// Native preset ids for the seven lanes. Every built-in preset is selectable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Palette {
    /// Main hook instrument.
    pub lead: String,
    /// Chords and arpeggios.
    pub harmony: String,
    /// Bass instrument.
    pub bass: String,
    /// Slower bridge melody.
    pub emotional: String,
    /// Kick drum.
    pub kick: String,
    /// Snare drum.
    pub snare: String,
    /// Closed hi-hat.
    pub hat: String,
}
impl Default for Palette {
    fn default() -> Self {
        Self {
            lead: "aloboi_warm_lead".into(),
            harmony: "trig_velvet_chord_pluck".into(),
            bass: "pc98_opna_slap_bass".into(),
            emotional: "aloboi_cello_solo".into(),
            kick: "kick".into(),
            snare: "snare".into(),
            hat: "hihat_closed".into(),
        }
    }
}

/// Repeatable composition recipe. Omitted fields use the documented defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent user-selectable musical features, not state-machine flags"
)]
pub struct ComposeConfig {
    /// Project title, at most 160 characters.
    pub title: String,
    /// Deterministic random seed. Identical settings yield identical notes.
    pub seed: u32,
    /// Groove and arrangement character.
    pub vibe: Vibe,
    /// Root note name, e.g. C, F#, Bb.
    pub key: String,
    /// major, `natural_minor`, `harmonic_minor`, dorian, mixolydian, `major_pentatonic` or `minor_pentatonic`.
    pub scale: String,
    /// Tempo in beats per minute, 40-240.
    pub bpm: u16,
    /// Short (16), standard (32), or extended (48) bars.
    pub form: Form,
    /// Native swing percentage, 0-60. Zero is straight.
    pub swing: u8,
    /// Bass rhythm algorithm.
    pub bass_rhythm: BassRhythm,
    /// Bass attacks per 16-step verse bar, 0-16. Sections scale this budget.
    pub bass_hits: u8,
    /// Reuse and develop one melodic motif across sections.
    pub motif_development: bool,
    /// Prefer small melodic steps and smoothly connected chord voicings.
    pub voice_leading: bool,
    /// Coordinate phrase endings, dominant preparation and final tonic.
    pub cadences: bool,
    /// Add sevenths or ninths to suitable chords.
    pub chord_colors: bool,
    /// Borrow minor iv in a major-key bridge.
    pub modal_mixture: bool,
    /// Use a dominant of the next chord at selected transitions.
    pub secondary_dominants: bool,
    /// Prepare and resolve a suspension at suitable bridge chord changes.
    pub suspensions: bool,
    /// Lock in syncopated octave bass and offbeat stabs for neon chiptune funk.
    pub neon_funk: bool,
    /// Add deliberate pickups, slides and glissando to selected phrases.
    pub expressive_slides: bool,
    /// Native and `ZzFX` instrument preset IDs.
    pub instruments: Palette,
}
impl Default for ComposeConfig {
    fn default() -> Self {
        Self {
            title: "New CTG composition".into(),
            seed: 42,
            vibe: Vibe::Funk,
            key: "D".into(),
            scale: "dorian".into(),
            bpm: 124,
            form: Form::Standard,
            swing: 12,
            bass_rhythm: BassRhythm::Mixed,
            bass_hits: 7,
            motif_development: true,
            voice_leading: true,
            cadences: true,
            chord_colors: true,
            modal_mixture: false,
            secondary_dominants: false,
            suspensions: true,
            instruments: Palette::default(),
            neon_funk: false,
            expressive_slides: false,
        }
    }
}
impl ComposeConfig {
    pub(super) fn validate(&self) -> Result<Scale, String> {
        if self.title.chars().count() > 160 {
            return Err("title must be at most 160 characters".into());
        }
        if !(40..=240).contains(&self.bpm) {
            return Err("bpm must be 40-240".into());
        }
        if self.swing > 60 {
            return Err("swing must be 0-60".into());
        }
        if self.bass_hits > 16 {
            return Err("bass_hits must be 0-16".into());
        }
        let root = self.key.parse::<PitchClass>().map_err(|e| e.to_string())?;
        let kind = match self.scale.as_str() {
            "major" => ScaleKind::Major, "natural_minor" => ScaleKind::NaturalMinor,
            "harmonic_minor" => ScaleKind::HarmonicMinor, "dorian" => ScaleKind::Dorian,
            "mixolydian" => ScaleKind::Mixolydian, "major_pentatonic" => ScaleKind::MajorPentatonic,
            "minor_pentatonic" => ScaleKind::MinorPentatonic,
            _ => return Err("scale must be major, natural_minor, harmonic_minor, dorian, mixolydian, major_pentatonic or minor_pentatonic".into()),
        };
        Ok(Scale::new(root, kind))
    }
}
