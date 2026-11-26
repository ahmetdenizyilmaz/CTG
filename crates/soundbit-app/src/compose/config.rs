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
    /// Sixteen sections, 64 bars.
    Long,
    /// Twenty-four sections, 96 bars.
    Epic,
    /// Thirty-two sections, 128 bars.
    Marathon,
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

/// High-level electronic/rave/world style preset driving default groove, contour and bass feel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GenreStyle {
    /// Let vibe and seed choose freely.
    #[default]
    Auto,
    /// 138 BPM soaring supersaw festival trance.
    EuphoricTrance,
    /// 303 diode-ladder warehouse acid rave.
    #[serde(rename = "acid_warehouse_303")]
    AcidWarehouse303,
    /// 174 BPM 2-step drum & bass with sustained Reese lead/bass.
    LiquidDnb,
    /// Skipped 2-step UK Garage shuffle with pitched vocal chops.
    #[serde(rename = "uk_garage_2step")]
    UkGarage2Step,
    /// French-touch electro-funk & talkbox disco bounce.
    FrenchTouchDisco,
    /// Offbeat Melbourne bounce / FM donk festival drop.
    MelbourneBounce,
    /// Heavy midtempo cyberpunk stomp with wide leaps.
    MidtempoCyberpunk,
    /// 90s Korg M1 piano house & organ bass.
    #[serde(rename = "piano_house_90s")]
    PianoHouse90s,
    /// Hypnotic melodic techno call-and-response.
    AfterlifeTechno,
    /// Balkan brass / Phrygian ornamental festival rave.
    BalkanFestivalBrass,
    /// High-octane 150-162 BPM Initial D Super Eurobeat with synth brass & solo guitar.
    #[serde(rename = "initial_d_eurobeat")]
    InitialDEurobeat,
    /// Fast 160-175 BPM anime opening techno with Royal Road sabi & sparkle arps.
    AnimeOpeningTechno,
    /// Uprising melodic techno with rising buildups & key-change finale.
    UprisingMelodicTechno,
    /// 90s Hyper Eurodance rave with M1 chords, cowbell & explosive choruses.
    HyperEurodanceRave,
}

impl GenreStyle {
    /// Whether this blueprint uses the multi-section semi-pro song engine.
    #[must_use]
    pub const fn is_semi_pro(self) -> bool {
        matches!(
            self,
            Self::InitialDEurobeat
                | Self::AnimeOpeningTechno
                | Self::UprisingMelodicTechno
                | Self::HyperEurodanceRave
        )
    }
}

/// Melodic contour & rhythmic phrasing archetype.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MelodyContour {
    /// Vary archetype deterministically from the seed and genre.
    #[default]
    Auto,
    /// Rapid alternation between a fixed anchor pedal note and upper chord tones.
    PedalPivot,
    /// Starts on a high 7th/9th peak and cascades downward in sequential terraces.
    DescendingCascade,
    /// Phrase begins with a late-bar pickup resolving into a long downbeat note.
    AnacrusisPickup,
    /// Deliberately rests on beat 1 and punches syncopated offbeats.
    OffbeatSyncopated,
    /// Bar 1 Question on Lead answered in Bar 2 by the Answer / Bridge instrument.
    DualCallResponse,
    /// Staccato vocal syllables grouped into bursts with generous rests.
    VocalChopSyllabic,
    /// Tight modal half-step / augmented-2nd turns with ornament pairs.
    ExoticOrnamental,
    /// Hypnotic 2-to-3 pitch Morse-code syncopated hook.
    RhythmicMorseHook,
    /// Cinematic octave, 6th and 7th interval leaps with sustained hold.
    WideIntervalLeap,
    /// Pocketed pentatonic/blues turnaround lick with grace anticipations.
    BluesFunkRiff,
}

impl MelodyContour {
    /// All explicit melodic contour generators (excluding Auto).
    pub const ALL: [Self; 10] = [
        Self::PedalPivot,
        Self::DescendingCascade,
        Self::AnacrusisPickup,
        Self::OffbeatSyncopated,
        Self::DualCallResponse,
        Self::VocalChopSyllabic,
        Self::ExoticOrnamental,
        Self::RhythmicMorseHook,
        Self::WideIntervalLeap,
        Self::BluesFunkRiff,
    ];

    /// Snake-case identifier for design reports and routing.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::PedalPivot => "pedal_pivot",
            Self::DescendingCascade => "descending_cascade",
            Self::AnacrusisPickup => "anacrusis_pickup",
            Self::OffbeatSyncopated => "offbeat_syncopated",
            Self::DualCallResponse => "dual_call_response",
            Self::VocalChopSyllabic => "vocal_chop_syllabic",
            Self::ExoticOrnamental => "exotic_ornamental",
            Self::RhythmicMorseHook => "rhythmic_morse_hook",
            Self::WideIntervalLeap => "wide_interval_leap",
            Self::BluesFunkRiff => "blues_funk_riff",
        }
    }

    /// Parse snake-case contour name back to enum variant.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "pedal_pivot" => Self::PedalPivot,
            "descending_cascade" => Self::DescendingCascade,
            "anacrusis_pickup" => Self::AnacrusisPickup,
            "offbeat_syncopated" => Self::OffbeatSyncopated,
            "dual_call_response" => Self::DualCallResponse,
            "vocal_chop_syllabic" => Self::VocalChopSyllabic,
            "exotic_ornamental" => Self::ExoticOrnamental,
            "rhythmic_morse_hook" => Self::RhythmicMorseHook,
            "wide_interval_leap" => Self::WideIntervalLeap,
            "blues_funk_riff" => Self::BluesFunkRiff,
            _ => Self::Auto,
        }
    }
}

/// 4-bar phrase structure grammar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MotifForm {
    /// Choose motif structure deterministically from seed or genre.
    #[default]
    Auto,
    /// Statement, repeat, contrasting B bar, resolved return.
    Aaba,
    /// Alternating A and B phrases ending on a C turnaround.
    Abac,
    /// Paired Question statements followed by paired elevated Answer bars.
    Aabb,
    /// Even bars call, odd bars answer and resolve.
    CallResponse,
}

impl MotifForm {
    /// All explicit motif forms (excluding Auto).
    pub const ALL: [Self; 4] = [Self::Aaba, Self::Abac, Self::Aabb, Self::CallResponse];

    /// Snake-case identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Aaba => "aaba",
            Self::Abac => "abac",
            Self::Aabb => "aabb",
            Self::CallResponse => "call_response",
        }
    }

    /// Parse snake-case form name back to enum variant.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "aaba" => Self::Aaba,
            "abac" => Self::Abac,
            "aabb" => Self::Aabb,
            "call_response" => Self::CallResponse,
            _ => Self::Auto,
        }
    }
}

/// Drum groove pattern template.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GrooveTemplate {
    /// Let vibe or genre_style select the drum groove.
    #[default]
    Auto,
    /// Four-on-the-floor kick with offbeat club hats.
    FourOnFloor,
    /// Syncopated funk pocket.
    Syncopated,
    /// Classic broken-beat pattern.
    Breakbeat,
    /// 3+3+2 Afro-Cuban/dancehall groove.
    Tresillo,
    /// Spacious half-time snare on beat 3.
    HalfTime,
    /// High-energy driving 16th-hat groove.
    Driving,
    /// 909 warehouse acid kick and crisp offbeat claps.
    AcidWarehouse,
    /// 174 BPM two-step Liquid DnB breakbeat.
    #[serde(rename = "liquid_dnb_174")]
    LiquidDnb174,
    /// Skipped UK Garage two-step shuffle hats.
    #[serde(rename = "uk_garage_2step")]
    UkGarage2Step,
    /// French Touch four-on-the-floor disco pocket.
    FrenchDisco,
    /// Melbourne Bounce heavy kick and clean offbeat groove.
    MelbourneOffbeat,
    /// Midtempo cyberpunk stomp with heavy halftime snare.
    MidtempoStomp,
    /// 90s M1 Piano House swinging club groove.
    HouseM1,
    /// Hypnotic Afterlife melodic techno rolling 16ths.
    MelodicTechno,
    /// High-energy Balkan festival brass groove.
    BalkanBrass,
    /// 150-162 BPM Super Eurobeat four-on-floor & driving 16th offbeats.
    #[serde(rename = "eurobeat_156")]
    Eurobeat156,
    /// Fast anime opening techno groove with accelerating risers.
    #[serde(rename = "anime_techno_166")]
    AnimeTechno166,
}

impl GrooveTemplate {
    /// Snake-case identifier used by percussion.rs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::FourOnFloor => "four_on_floor",
            Self::Syncopated => "syncopated",
            Self::Breakbeat => "breakbeat",
            Self::Tresillo => "tresillo",
            Self::HalfTime => "half_time",
            Self::Driving => "driving",
            Self::AcidWarehouse => "acid_warehouse",
            Self::LiquidDnb174 => "liquid_dnb_174",
            Self::UkGarage2Step => "uk_garage_2step",
            Self::FrenchDisco => "french_disco",
            Self::MelbourneOffbeat => "melbourne_offbeat",
            Self::MidtempoStomp => "midtempo_stomp",
            Self::HouseM1 => "house_m1",
            Self::MelodicTechno => "melodic_techno",
            Self::BalkanBrass => "balkan_brass",
            Self::Eurobeat156 => "eurobeat_156",
            Self::AnimeTechno166 => "anime_techno_166",
        }
    }
}

/// Chord progression vocabulary for multi-section semi-pro songwriting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChordProgressionStyle {
    /// Let section role and genre choose the optimal harmonic progression.
    #[default]
    Auto,
    /// Iconic Japanese anime & pop progression (`VI -> VII -> v -> i` / `IV -> V -> iii -> vi`).
    RoyalRoad,
    /// Relentless Initial D circle-of-fifths motion (`i -> iv -> VII -> III -> VI -> iv -> V7 -> i`).
    EurobeatCircle,
    /// Soaring 4-chord uprising anthem (`i -> VI -> III -> VII`).
    UprisingAnthem,
    /// J-pop / Eurodance driving lift (`VI -> VII -> i -> i` / `VI -> VII -> III -> v`).
    KomuroDrive,
    /// Dramatic minor climb into V7 dominant tension (`iv -> v -> VI -> VII -> V7`).
    DramaticMinor,
}

/// Musical role of a custom or generated arrangement section.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SectionKind {
    /// High-energy opening synth brass or hook riff.
    IntroRiff,
    /// First lyrical verse melody.
    VerseA,
    /// Answering higher verse continuation melody.
    VerseB,
    /// Climbing tension builder with snare rolls into Chorus.
    PreChorus,
    /// Main high-altitude soaring sabi/chorus hook.
    #[default]
    Chorus,
    /// Blazing guitar / synth solo showcase section.
    Solo,
    /// Stripped emotional breakdown before the final drop.
    Breakdown,
    /// Maximum energy finale chorus (optionally +2 st key change).
    FinalChorus,
    /// Punchy closing turnaround.
    Outro,
}

/// One customizable song section inside a multi-section arrangement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct SectionSpec {
    /// Musical function of this section.
    pub kind: SectionKind,
    /// Number of four-beat bars (2-16).
    pub bars: u8,
    /// Independent melodic theme slot (0-5) so different sections sing different melodies.
    pub melody_theme: u8,
    /// Chord progression blueprint (Auto adapts to `kind`).
    pub progression: ChordProgressionStyle,
    /// Section energy intensity (1-5).
    pub energy: u8,
    /// Section transposition shift in semitones (-12 to +12).
    pub transpose_semitones: i8,
    /// Per-section seed tweak allowing individual section melody rerolls.
    pub seed_offset: u32,
}

impl Default for SectionSpec {
    fn default() -> Self {
        Self {
            kind: SectionKind::Chorus,
            bars: 8,
            melody_theme: 4,
            progression: ChordProgressionStyle::Auto,
            energy: 4,
            transpose_semitones: 0,
            seed_offset: 0,
        }
    }
}

/// Accompaniment / chord texture behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccompanimentStyle {
    /// Let vibe or genre_style select accompaniment texture.
    #[default]
    Auto,
    /// Half-bar stereo chord swells.
    ChordPulses,
    /// Offbeat rave chord stabs.
    OffbeatStabs,
    /// Syncopated dyad comping.
    BrokenChords,
    /// Interlocking answer notes in melody rests.
    Pendulum,
    /// Polyrhythmic 3-over-4 rising pluck pattern.
    RisingArp,
    /// Phrase-end turnaround cascade.
    FallingArp,
    /// Full-bar stereo pad pumped by the kick sidechain bus.
    SidechainPad,
    /// 90s Korg M1 syncopated house piano stabs.
    PianoHouseStabs,
    /// Minimal gap chord stabs playing only where the lead rests.
    MinimalGapAnswer,
}

impl AccompanimentStyle {
    /// Snake-case identifier used by accompaniment.rs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::ChordPulses => "chord_pulses",
            Self::OffbeatStabs => "offbeat_stabs",
            Self::BrokenChords => "broken_chords",
            Self::Pendulum => "pendulum",
            Self::RisingArp => "rising_arp",
            Self::FallingArp => "falling_arp",
            Self::SidechainPad => "sidechain_pad",
            Self::PianoHouseStabs => "piano_house_stabs",
            Self::MinimalGapAnswer => "minimal_gap_answer",
        }
    }
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
    /// Upper-register sparkle arpeggio (10-track studio mode).
    pub arp: String,
    /// Offbeat Eurobeat / Initial D cowbell (10-track studio mode).
    pub cowbell: String,
    /// Section-drop crash cymbal (10-track studio mode).
    pub crash: String,
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
            arp: "anime_sparkle_arp".into(),
            cowbell: "eurobeat_cowbell".into(),
            crash: "club_crash_cymbal".into(),
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
    /// Electronic, rave or world style blueprint (auto preserves classic vibe behavior).
    pub genre_style: GenreStyle,
    /// Melodic contour and rhythmic phrasing archetype (auto varies across all 10 modes).
    pub melody_contour: MelodyContour,
    /// Four-bar motif development grammar (AABA, ABAC, AABB or Call/Response).
    pub motif_form: MotifForm,
    /// Explicit drum groove override.
    pub groove_template: GrooveTemplate,
    /// Explicit chord / accompaniment texture override.
    pub accompaniment_style: AccompanimentStyle,
    /// Target melody breathing space / silence percentage, 0-80.
    pub rest_density: u8,
    /// Preference for offbeat & upbeat rhythmic syncopation, 0-100.
    pub syncopation: u8,
    /// Probability of pitch glides/scoops on expressive notes, 0-100.
    pub slide_probability: u8,
    /// Semi-pro harmonic chord progression family.
    pub progression_style: ChordProgressionStyle,
    /// Modulate FinalChorus +2 semitones upward for an uprising climax.
    pub key_change_finale: bool,
    /// Add accelerating 8th->16th snare buildup rolls before choruses.
    pub riser_builds: bool,
    /// Enable 10-track studio arrangement layers (Sparkle Arp, Cowbell, Crash Cymbal).
    pub studio_layers: bool,
    /// Custom multi-section song timeline (overrides default Form sections when non-empty).
    pub custom_sections: Vec<SectionSpec>,
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
            genre_style: GenreStyle::Auto,
            melody_contour: MelodyContour::Auto,
            motif_form: MotifForm::Auto,
            groove_template: GrooveTemplate::Auto,
            accompaniment_style: AccompanimentStyle::Auto,
            rest_density: 35,
            syncopation: 40,
            slide_probability: 25,
            progression_style: ChordProgressionStyle::Auto,
            key_change_finale: false,
            riser_builds: false,
            studio_layers: false,
            custom_sections: Vec::new(),
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
        if self.rest_density > 80 {
            return Err("rest_density must be 0-80".into());
        }
        if self.syncopation > 100 {
            return Err("syncopation must be 0-100".into());
        }
        if self.slide_probability > 100 {
            return Err("slide_probability must be 0-100".into());
        }
        let mut total_bars = 0_usize;
        for sec in &self.custom_sections {
            if !(2..=16).contains(&sec.bars) {
                return Err("custom_sections[].bars must be 2-16".into());
            }
            if sec.melody_theme > 5 {
                return Err("custom_sections[].melody_theme must be 0-5".into());
            }
            if !(1..=5).contains(&sec.energy) {
                return Err("custom_sections[].energy must be 1-5".into());
            }
            if !(-12..=12).contains(&sec.transpose_semitones) {
                return Err("custom_sections[].transpose_semitones must be -12..=12".into());
            }
            total_bars += usize::from(sec.bars);
        }
        if total_bars > 192 {
            return Err("total custom_sections bars must be at most 192".into());
        }
        let root = self.key.parse::<PitchClass>().map_err(|e| e.to_string())?;
        let kind = match self.scale.as_str() {
            "major" => ScaleKind::Major,
            "natural_minor" => ScaleKind::NaturalMinor,
            "harmonic_minor" => ScaleKind::HarmonicMinor,
            "melodic_minor" => ScaleKind::MelodicMinor,
            "dorian" => ScaleKind::Dorian,
            "phrygian" => ScaleKind::Phrygian,
            "lydian" => ScaleKind::Lydian,
            "mixolydian" => ScaleKind::Mixolydian,
            "major_pentatonic" => ScaleKind::MajorPentatonic,
            "minor_pentatonic" => ScaleKind::MinorPentatonic,
            "blues" => ScaleKind::Blues,
            _ => {
                return Err(
                    "scale must be major, natural_minor, harmonic_minor, melodic_minor, dorian, phrygian, lydian, mixolydian, major_pentatonic, minor_pentatonic or blues"
                        .into(),
                )
            }
        };
        Ok(Scale::new(root, kind))
    }
}
