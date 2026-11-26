//! A song-level identity: coherent choices reused across its changing sections.
use super::{
    ComposeConfig, Form, Role, Vibe,
    config::{
        AccompanimentStyle, ChordProgressionStyle, GenreStyle, GrooveTemplate, MelodyContour,
        MotifForm, SectionKind,
    },
    rhythm::Rng,
};
use serde::Serialize;

/// The arrangement decisions that distinguish one generated song from another.
#[derive(Debug, Clone, Serialize)]
pub struct SongDesign {
    /// The recurring drum feel; sections develop this rather than rerolling every bar.
    pub groove: String,
    /// Main harmony texture.
    pub accompaniment: String,
    /// Bass contour shared by related phrases.
    pub bass_motion: String,
    /// Selected melodic contour archetype.
    pub melody_contour: String,
    /// Selected 4-bar motif grammar form.
    pub motif_form: String,
    /// Length of the hook before its answering phrase, in bars.
    pub phrase_bars: usize,
    /// Normal bars per chord; cadences and transitions can move faster.
    pub bars_per_chord: usize,
    /// Ordered section roles and their durations.
    pub sections: Vec<Section>,
    pub(super) answer: usize,
    pub(super) register: i16,
    pub(super) articulation: usize,
}

/// One section, with a duration chosen as part of the whole form.
#[derive(Debug, Clone, Serialize)]
pub struct Section {
    /// What the section contributes to the song.
    pub role: Role,
    /// Length in four-beat bars.
    pub bars: usize,
    /// Specific song section role for semi-pro multi-theme arrangement.
    pub kind: SectionKind,
    /// Melodic theme index (0-5) so different sections play distinct melodies.
    pub melody_theme: u8,
    /// Harmonic progression blueprint for this section.
    pub progression: ChordProgressionStyle,
    /// Energy level 1-5 driving arrangement density.
    pub energy: u8,
    /// Section transposition shift in semitones.
    pub transpose_semitones: i8,
    /// Independent per-section melody seed tweak.
    pub seed_offset: u32,
}

impl SongDesign {
    pub(super) fn bars(&self) -> Vec<PlannedBar> {
        self.sections
            .iter()
            .enumerate()
            .flat_map(|(section, part)| {
                (0..part.bars).map(move |local_bar| PlannedBar {
                    section_bars: part.bars,
                    role: part.role,
                    local_bar,
                    section_end: local_bar + 1 == part.bars,
                    final_phrase: section + 1 == self.sections.len() && local_bar + 4 >= part.bars,
                    kind: part.kind,
                    melody_theme: part.melody_theme,
                    progression: part.progression,
                    energy: part.energy,
                    transpose_semitones: part.transpose_semitones,
                    seed_offset: part.seed_offset,
                })
            })
            .collect()
    }

    pub(super) fn new(config: &ComposeConfig) -> Self {
        let mut rng = Rng::new(config.seed ^ 0x51ec_7102);
        let grooves: &[&str] = match config.vibe {
            Vibe::Funk => &["syncopated", "four_on_floor", "breakbeat", "tresillo"],
            Vibe::Dungeon => &["half_time", "tresillo", "breakbeat"],
            Vibe::Battle | Vibe::Boss => &["driving", "breakbeat", "four_on_floor"],
            _ => &["syncopated", "four_on_floor", "half_time", "tresillo"],
        };
        let default_groove = grooves[rng.pick(grooves.len())];
        let default_acc = [
            "rising_arp",
            "falling_arp",
            "pendulum",
            "broken_chords",
            "offbeat_stabs",
            "chord_pulses",
        ][rng.pick(6)];
        let default_bass = ["root_fifth", "octave", "walking", "pedal"][rng.pick(4)];
        let default_contour = MelodyContour::ALL[rng.pick(MelodyContour::ALL.len())];
        let default_form = MotifForm::ALL[rng.pick(MotifForm::ALL.len())];

        let (g_groove, g_acc, g_bass, g_contour, g_form): (
            &str,
            &str,
            &str,
            MelodyContour,
            MotifForm,
        ) = match config.genre_style {
            GenreStyle::Auto => (
                default_groove,
                default_acc,
                default_bass,
                default_contour,
                default_form,
            ),
            GenreStyle::EuphoricTrance => (
                "four_on_floor",
                "sidechain_pad",
                "octave",
                MelodyContour::PedalPivot,
                MotifForm::Aaba,
            ),
            GenreStyle::AcidWarehouse303 => (
                "acid_warehouse",
                "minimal_gap_answer",
                "acid_303",
                MelodyContour::OffbeatSyncopated,
                MotifForm::Abac,
            ),
            GenreStyle::LiquidDnb => (
                "liquid_dnb_174",
                "sidechain_pad",
                "reese_glide",
                MelodyContour::AnacrusisPickup,
                MotifForm::Aaba,
            ),
            GenreStyle::UkGarage2Step => (
                "uk_garage_2step",
                "offbeat_stabs",
                "garage_sub",
                MelodyContour::VocalChopSyllabic,
                MotifForm::CallResponse,
            ),
            GenreStyle::FrenchTouchDisco => (
                "french_disco",
                "offbeat_stabs",
                "octave",
                MelodyContour::BluesFunkRiff,
                MotifForm::Abac,
            ),
            GenreStyle::MelbourneBounce => (
                "melbourne_offbeat",
                "minimal_gap_answer",
                "offbeat_bounce",
                MelodyContour::RhythmicMorseHook,
                MotifForm::Aabb,
            ),
            GenreStyle::MidtempoCyberpunk => (
                "midtempo_stomp",
                "chord_pulses",
                "pedal",
                MelodyContour::WideIntervalLeap,
                MotifForm::Abac,
            ),
            GenreStyle::PianoHouse90s => (
                "house_m1",
                "piano_house_stabs",
                "m1_organ",
                MelodyContour::DescendingCascade,
                MotifForm::Aaba,
            ),
            GenreStyle::AfterlifeTechno => (
                "melodic_techno",
                "rising_arp",
                "rolling_16th",
                MelodyContour::DualCallResponse,
                MotifForm::CallResponse,
            ),
            GenreStyle::BalkanFestivalBrass => (
                "balkan_brass",
                "offbeat_stabs",
                "oompah_5th",
                MelodyContour::ExoticOrnamental,
                MotifForm::Abac,
            ),
            GenreStyle::InitialDEurobeat => (
                "eurobeat_156",
                "offbeat_stabs",
                "eurobeat_octave",
                MelodyContour::PedalPivot,
                MotifForm::Aaba,
            ),
            GenreStyle::AnimeOpeningTechno => (
                "anime_techno_166",
                "piano_house_stabs",
                "eurobeat_octave",
                MelodyContour::WideIntervalLeap,
                MotifForm::Aaba,
            ),
            GenreStyle::UprisingMelodicTechno => (
                "melodic_techno",
                "sidechain_pad",
                "rolling_16th",
                MelodyContour::DescendingCascade,
                MotifForm::Abac,
            ),
            GenreStyle::HyperEurodanceRave => (
                "four_on_floor",
                "offbeat_stabs",
                "eurobeat_octave",
                MelodyContour::PedalPivot,
                MotifForm::Aaba,
            ),
        };

        let groove = if config.groove_template != GrooveTemplate::Auto {
            config.groove_template.as_str()
        } else {
            g_groove
        };
        let accompaniment = if config.accompaniment_style != AccompanimentStyle::Auto {
            config.accompaniment_style.as_str()
        } else {
            g_acc
        };
        let contour = if config.melody_contour != MelodyContour::Auto {
            config.melody_contour
        } else if config.neon_funk && g_contour == MelodyContour::DualCallResponse {
            // Preserve lead lane visibility across bars for neon-funk
            MelodyContour::BluesFunkRiff
        } else {
            g_contour
        };
        let motif_form = if config.motif_form != MotifForm::Auto {
            config.motif_form
        } else {
            g_form
        };

        let built_sections = if !config.custom_sections.is_empty() {
            config
                .custom_sections
                .iter()
                .map(|spec| Section {
                    role: kind_to_role(spec.kind),
                    bars: usize::from(spec.bars),
                    kind: spec.kind,
                    melody_theme: spec.melody_theme,
                    progression: if spec.progression == ChordProgressionStyle::Auto {
                        config.progression_style
                    } else {
                        spec.progression
                    },
                    energy: spec.energy,
                    transpose_semitones: if config.key_change_finale
                        && spec.kind == SectionKind::FinalChorus
                        && spec.transpose_semitones == 0
                    {
                        2
                    } else {
                        spec.transpose_semitones
                    },
                    seed_offset: spec.seed_offset,
                })
                .collect()
        } else if config.genre_style.is_semi_pro() {
            semi_pro_sections(config)
        } else {
            sections(config.form, &mut rng)
        };

        let mut design = Self {
            groove: groove.into(),
            accompaniment: accompaniment.into(),
            bass_motion: g_bass.into(),
            melody_contour: contour.as_str().into(),
            motif_form: motif_form.as_str().into(),
            phrase_bars: if config.genre_style != GenreStyle::Auto || rng.chance(65) {
                2
            } else {
                1
            },
            bars_per_chord: if config.genre_style.is_semi_pro() || !config.custom_sections.is_empty()
            {
                1
            } else if rng.chance(35) {
                2
            } else {
                1
            },
            sections: built_sections,
            answer: rng.pick(4),
            register: if rng.chance(20) { 48 } else { 60 },
            articulation: rng.pick(3),
        };
        if config.neon_funk {
            design.groove = "syncopated".into();
            design.accompaniment = "offbeat_stabs".into();
            design.bass_motion = "octave".into();
        }
        design
    }
}

pub(super) struct PlannedBar {
    pub(super) section_bars: usize,
    pub(super) role: Role,
    pub(super) local_bar: usize,
    pub(super) section_end: bool,
    pub(super) final_phrase: bool,
    pub(super) kind: SectionKind,
    pub(super) melody_theme: u8,
    pub(super) progression: ChordProgressionStyle,
    pub(super) energy: u8,
    pub(super) transpose_semitones: i8,
    pub(super) seed_offset: u32,
}

const fn kind_to_role(kind: SectionKind) -> Role {
    match kind {
        SectionKind::Breakdown => Role::Breakdown,
        SectionKind::VerseB | SectionKind::PreChorus => Role::Contrast,
        SectionKind::IntroRiff
        | SectionKind::Chorus
        | SectionKind::Solo
        | SectionKind::FinalChorus => Role::Climax,
        SectionKind::VerseA | SectionKind::Outro => Role::Verse,
    }
}

fn semi_pro_sections(config: &ComposeConfig) -> Vec<Section> {
    let shift = if config.key_change_finale { 2 } else { 0 };
    let prog = config.progression_style;
    let sec = |kind: SectionKind, bars: usize, theme: u8, energy: u8, trans: i8| Section {
        role: kind_to_role(kind),
        bars,
        kind,
        melody_theme: theme,
        progression: prog,
        energy,
        transpose_semitones: trans,
        seed_offset: 0,
    };
    if config.form == Form::Short {
        return vec![
            sec(SectionKind::IntroRiff, 4, 0, 5, 0),
            sec(SectionKind::VerseA, 4, 1, 3, 0),
            sec(SectionKind::PreChorus, 4, 3, 4, 0),
            sec(SectionKind::FinalChorus, 8, 4, 5, shift),
        ];
    }
    let mut plan = vec![
        sec(SectionKind::IntroRiff, 4, 0, 5, 0),
        sec(SectionKind::VerseA, 8, 1, 3, 0),
        sec(SectionKind::VerseB, 8, 2, 4, 0),
        sec(SectionKind::PreChorus, 4, 3, 4, 0),
        sec(SectionKind::Chorus, 8, 4, 5, 0),
        sec(SectionKind::Solo, 8, 5, 5, 0),
        sec(SectionKind::FinalChorus, 8, 4, 5, shift),
    ];
    if !matches!(config.form, Form::Short | Form::Standard) {
        plan.splice(
            5..5,
            [
                sec(SectionKind::Breakdown, 4, 2, 2, 0),
                sec(SectionKind::VerseA, 4, 1, 3, 0),
                sec(SectionKind::PreChorus, 4, 3, 4, 0),
            ],
        );
    }
    plan
}

fn sections(form: Form, rng: &mut Rng) -> Vec<Section> {
    use Role::{Breakdown as X, Bridge as C, Climax as D, Contrast as B, Verse as A};
    let short: &[&[(Role, usize)]] = &[
        &[(A, 4), (B, 4), (C, 4), (D, 4)],
        &[(X, 2), (A, 4), (B, 2), (C, 4), (D, 4)],
        &[(A, 4), (C, 4), (B, 4), (D, 4)],
        &[(A, 2), (B, 2), (A, 4), (C, 4), (D, 4)],
    ];
    let long: &[&[(Role, usize)]] = &[
        &[(A, 8), (B, 4), (A, 4), (C, 4), (X, 2), (B, 2), (D, 8)],
        &[(X, 2), (A, 6), (B, 4), (A, 4), (C, 4), (B, 4), (D, 8)],
        &[(A, 4), (B, 4), (C, 8), (A, 4), (X, 4), (D, 8)],
        &[(A, 4), (A, 4), (B, 8), (C, 4), (X, 4), (D, 8)],
        &[(A, 4), (B, 4), (A, 4), (C, 4), (B, 4), (A, 4), (D, 8)],
    ];
    let choices = if form == Form::Short { short } else { long };
    let mut chosen = choices[rng.pick(choices.len())].to_vec();
    match form {
        Form::Short | Form::Standard => {}
        Form::Extended => {
            chosen.extend([(A, 4), (B, 4), (C, 4), (D, 4)]);
        }
        Form::Long => {
            chosen.extend([(A, 4), (B, 4), (C, 4), (D, 4), (X, 4), (A, 4), (B, 4), (D, 4)]);
        }
        Form::Epic => {
            chosen.extend([
                (A, 4), (B, 4), (C, 4), (D, 4),
                (X, 4), (A, 4), (B, 4), (D, 4),
                (A, 8), (C, 8), (B, 8), (D, 8),
            ]);
        }
        Form::Marathon => {
            chosen.extend([
                (A, 4), (B, 4), (C, 4), (D, 4),
                (X, 4), (A, 4), (B, 4), (D, 4),
                (A, 8), (C, 8), (B, 8), (D, 8),
                (X, 4), (A, 8), (B, 4), (C, 8), (D, 8),
            ]);
        }
    }
    chosen
        .into_iter()
        .map(|(role, bars)| Section {
            role,
            bars,
            kind: match role {
                Role::Verse => SectionKind::VerseA,
                Role::Contrast => SectionKind::VerseB,
                Role::Bridge => SectionKind::Breakdown,
                Role::Breakdown => SectionKind::Breakdown,
                Role::Climax => SectionKind::Chorus,
            },
            melody_theme: match role {
                Role::Contrast => 2,
                Role::Climax => 4,
                _ => 1,
            },
            progression: ChordProgressionStyle::Auto,
            energy: match role {
                Role::Climax => 5,
                Role::Contrast => 4,
                Role::Breakdown => 2,
                _ => 3,
            },
            transpose_semitones: 0,
            seed_offset: 0,
        })
        .collect()
}
