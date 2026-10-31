//! A song-level identity: coherent choices reused across its changing sections.
use super::{ComposeConfig, Form, Role, Vibe, rhythm::Rng};
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
        Self {
            groove: grooves[rng.pick(grooves.len())].into(),
            accompaniment: [
                "rising_arp",
                "falling_arp",
                "pendulum",
                "broken_chords",
                "offbeat_stabs",
                "chord_pulses",
            ][rng.pick(6)]
            .into(),
            bass_motion: ["root_fifth", "octave", "walking", "pedal"][rng.pick(4)].into(),
            phrase_bars: if rng.chance(65) { 2 } else { 1 },
            bars_per_chord: if rng.chance(35) { 2 } else { 1 },
            sections: sections(config.form, &mut rng),
            answer: rng.pick(4),
            register: if rng.chance(20) { 48 } else { 60 },
            articulation: rng.pick(3),
        }
    }
}

pub(super) struct PlannedBar {
    pub(super) section_bars: usize,
    pub(super) role: Role,
    pub(super) local_bar: usize,
    pub(super) section_end: bool,
    pub(super) final_phrase: bool,
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
    if form == Form::Extended {
        chosen.extend([(A, 4), (B, 4), (C, 4), (D, 4)]);
    }
    chosen
        .into_iter()
        .map(|(role, bars)| Section { role, bars })
        .collect()
}
