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
    let base = usize::from(context.config.bass_hits);
    let hits = match context.role {
        Role::Bridge => (base * 7 + 5) / 10,
        Role::Climax => (base * 12 + 5) / 10,
        _ => base,
    }
    .min(16);
    let mask = bass_mask(hits, context.config.bass_rhythm, kicks, rng);
    let starts: Vec<_> = mask
        .iter()
        .enumerate()
        .filter(|(_, on)| **on)
        .map(|(i, _)| u16::try_from(i).unwrap_or(0))
        .collect();
    let mut notes = Vec::new();
    for (i, &start) in starts.iter().enumerate() {
        let chord = context.chord;
        let motion = context.design.bass_motion.as_str();
        let tone =
            if i == 0 || motion == "pedal" || (start.is_multiple_of(4) && motion != "walking") {
                chord.root
            } else if motion == "walking" {
                chord.tones[(i + context.bar) % 3]
            } else {
                chord.tones[if i % 3 == 1 { 2 } else { 1 }]
            };
        let pitch = 36
            + tone
            + if (motion == "octave" && i % 2 == 1)
                || (context.config.vibe == Vibe::Funk && i % 4 == 3)
            {
                12
            } else {
                0
            };
        let next = starts.get(i + 1).copied().unwrap_or(16);
        notes.push(note(
            pitch,
            start,
            (next - start).min(if context.role == Role::Bridge || motion == "pedal" {
                5
            } else {
                2
            }),
            if start.is_multiple_of(4) { 105 } else { 88 },
        )?);
    }
    Ok(notes)
}
