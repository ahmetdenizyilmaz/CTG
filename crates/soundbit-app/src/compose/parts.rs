//! Melody, bass, accompaniment and independent percussion lanes.
use super::{
    ComposeConfig, SongDesign,
    config::Vibe,
    harmony::{Chord, Role},
    rhythm::{Rng, bass_mask},
};
use soundbit_core::{Note, Pitch, Scale, Velocity};

pub(super) fn note(pitch: i16, start: u16, length: u16, velocity: u8) -> Result<Note, String> {
    let pitch = u8::try_from(pitch).map_err(|_| "generated pitch is outside MIDI range")?;
    Note::new(Pitch::new(pitch).map_err(|e| e.to_string())?, start, length)
        .map(|n| n.with_velocity(Velocity::new(velocity).unwrap_or_default()))
        .map_err(|e| e.to_string())
}

pub(super) struct BarContext<'a> {
    pub(super) config: &'a ComposeConfig,
    pub(super) design: &'a SongDesign,
    pub(super) chord: &'a Chord,
    pub(super) scale: Scale,
    pub(super) role: Role,
    pub(super) bar: usize,
    pub(super) final_bar: bool,
}

pub(super) fn bass(
    context: &BarContext<'_>,
    kicks: &[bool; 16],
    rng: &mut Rng,
) -> Result<Vec<Note>, String> {
    let motion = context.design.bass_motion.as_str();
    let hits = if motion == "eurobeat_octave" && context.role != Role::Breakdown {
        8
    } else {
        let base = usize::from(context.config.bass_hits);
        match context.role {
            Role::Bridge => (base * 7 + 5) / 10,
            Role::Climax => (base * 12 + 5) / 10,
            _ => base,
        }
        .min(16)
    };
    let mask = if hits > 0
        && context.config.bass_rhythm != super::BassRhythm::Euclidean
        && matches!(
            motion,
            "offbeat_bounce"
                | "m1_organ"
                | "acid_303"
                | "garage_sub"
                | "oompah_5th"
                | "rolling_16th"
                | "eurobeat_octave"
        ) {
        styled_bass_mask(hits, motion)
    } else {
        bass_mask(hits, context.config.bass_rhythm, kicks, rng)
    };
    let starts: Vec<_> = mask
        .iter()
        .enumerate()
        .filter(|(_, on)| **on)
        .map(|(i, _)| u16::try_from(i).unwrap_or(0))
        .collect();
    let mut notes = Vec::new();
    for (i, &start) in starts.iter().enumerate() {
        let chord = context.chord;
        let tone = if motion == "eurobeat_octave" {
            chord.root
        } else if i == 0 || motion == "pedal" || (start.is_multiple_of(4) && motion != "walking")
        {
            chord.root
        } else if motion == "walking" {
            chord.tones[(i + context.bar) % chord.tones.len()]
        } else if motion == "oompah_5th" {
            if i % 2 == 1 {
                chord.tones[2 % chord.tones.len()]
            } else {
                chord.root
            }
        } else {
            chord.tones[if i % 3 == 1 {
                2 % chord.tones.len()
            } else {
                1 % chord.tones.len()
            }]
        };
        let oct_lift = if (motion == "eurobeat_octave" && start % 4 == 2)
            || (motion == "octave" && i % 2 == 1)
            || (motion == "acid_303" && i % 3 == 2)
            || (motion == "offbeat_bounce" && start % 4 == 2 && i % 2 == 1)
            || (context.config.vibe == Vibe::Funk && i % 4 == 3)
        {
            12
        } else {
            0
        };
        let pitch = 36 + tone + oct_lift;
        let next = starts.get(i + 1).copied().unwrap_or(16);
        let max_len = if context.role == Role::Bridge || matches!(motion, "pedal" | "reese_glide") {
            6
        } else {
            2
        };
        let vel = if motion == "acid_303" && (i % 3 == 1 || oct_lift == 12) {
            118
        } else if start.is_multiple_of(4) {
            105
        } else if motion == "eurobeat_octave" {
            100
        } else {
            90
        };
        let mut bass_note = note(pitch, start, (next - start).min(max_len), vel)?;
        if matches!(motion, "acid_303" | "reese_glide")
            && !context.final_bar
            && i % 4 == 2
            && bass_note.pitch.get() >= 24
        {
            bass_note.slide_semitones = if oct_lift == 12 { -12 } else { 2 };
        }
        notes.push(bass_note);
    }
    Ok(notes)
}

fn styled_bass_mask(hits: usize, motion: &str) -> Vec<bool> {
    let priority: &[usize] = match motion {
        "offbeat_bounce" | "m1_organ" => &[2, 6, 10, 14, 0, 8, 4, 12, 3, 7, 11, 15, 1, 5, 9, 13],
        "garage_sub" => &[0, 3, 6, 10, 14, 8, 12, 4, 2, 7, 11, 15, 1, 5, 9, 13],
        "acid_303" => &[0, 2, 3, 6, 8, 10, 11, 14, 4, 12, 7, 15, 1, 5, 9, 13],
        "oompah_5th" | "eurobeat_octave" => {
            &[0, 2, 4, 6, 8, 10, 12, 14, 1, 3, 5, 7, 9, 11, 13, 15]
        }
        _ => &[0, 2, 3, 4, 6, 8, 10, 11, 12, 14, 15, 1, 5, 7, 9, 13],
    };
    let mut mask = vec![false; 16];
    for &step in priority.iter().take(hits.min(16)) {
        mask[step] = true;
    }
    mask
}
