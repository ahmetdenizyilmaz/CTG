//! Diverse accompaniment modes that leave mid-range space and never default to monotonous arpeggios.
use super::{
    Role,
    parts::{BarContext, note},
};
use soundbit_core::Note;

pub(super) fn harmony(
    context: &BarContext<'_>,
    voicing: &[i16],
    melody: &[Note],
) -> Result<Vec<Note>, String> {
    if context.role == Role::Breakdown {
        return Ok(Vec::new());
    }
    let mut notes = Vec::new();
    if context.role == Role::Bridge || context.final_bar {
        for &pitch in voicing {
            notes.push(note(pitch, 0, 16, 55)?);
        }
        return Ok(notes);
    }
    let style = context.design.accompaniment.as_str();
    let count = voicing.len();
    let lead_active_at = |step: u16| {
        melody
            .iter()
            .any(|n| step >= n.start && step < n.start + n.length.get())
    };

    match style {
        // Sidechain Pad Bed: full sustained chord swells driven by the audio-rate kick sidechain bus
        "sidechain_pad" => {
            for &pitch in voicing {
                notes.push(note(pitch, 0, 16, 56)?);
            }
        }
        // 90s Korg M1 Piano House Stabs: infectious 3+3+2+2+4 club piano stab rhythm
        "piano_house_stabs" => {
            for &(start, dur, vel) in &[
                (0_u16, 2_u16, 70_u8),
                (3, 2, 62),
                (6, 2, 68),
                (10, 2, 64),
                (14, 2, 58),
            ] {
                for &pitch in voicing {
                    notes.push(note(pitch, start, dur, vel)?);
                }
            }
        }
        // Minimal Gap Answer: strikes chords ONLY during beats where the lead hook is resting
        "minimal_gap_answer" => {
            for &start in &[2_u16, 6, 10, 14] {
                if lead_active_at(start) {
                    continue;
                }
                for &pitch in voicing {
                    notes.push(note(pitch, start, 2, 62)?);
                }
            }
        }
        // Mode 1: Half-bar stereo pad pulses
        "chord_pulses" => {
            for &start in &[0_u16, 8_u16] {
                for &pitch in voicing {
                    notes.push(note(pitch, start, 7, 56)?);
                }
            }
        }
        // Mode 2: Offbeat Rave / House Chord Stabs
        "offbeat_stabs" => {
            let starts: &[u16] = match context.bar % 3 {
                0 => &[2, 6, 10, 14],
                1 => &[2, 7, 10, 14],
                _ => &[3, 6, 10, 15],
            };
            for &start in starts {
                let dur = (16 - start).min(2);
                for &pitch in voicing {
                    notes.push(note(pitch, start, dur, 60)?);
                }
            }
        }
        // Mode 3: Syncopated Piano / Rhodes Dyad Comping (3+3+2 rhythmic comping, not an arp!)
        "broken_chords" => {
            for &(start, dur, pair_idx, vel) in &[
                (0_u16, 2_u16, 0_usize, 62_u8),
                (3, 2, 1, 58),
                (6, 2, 2, 64),
                (10, 2, 1, 56),
                (12, 3, 0, 60),
            ] {
                if start != 0 && lead_active_at(start) {
                    continue;
                }
                let p0 = voicing[pair_idx % count];
                let p1 = voicing[(pair_idx + 1) % count];
                notes.push(note(p0, start, dur, vel)?);
                if p1 != p0 {
                    notes.push(note(p1, start, dur, vel.saturating_sub(6))?);
                }
            }
        }
        // Mode 4: Interlocking Minimal Gap Answer (plays ONLY where the lead hook rests!)
        "pendulum" => {
            for (idx, &start) in [2_u16, 6, 10, 12, 14].iter().enumerate() {
                if lead_active_at(start) {
                    continue;
                }
                let pitch = voicing[(idx + context.bar) % count];
                notes.push(note(pitch, start, 2, 62)?);
            }
        }
        // Mode 5: Polyrhythmic 3-over-4 Dotted Pluck
        "rising_arp" => {
            for (idx, &start) in [0_u16, 3, 6, 9, 12, 14].iter().enumerate() {
                if start != 0 && lead_active_at(start) {
                    continue;
                }
                let pitch = voicing[idx % count];
                notes.push(note(pitch, start, 2, if idx % 2 == 0 { 64 } else { 52 })?);
            }
        }
        // Mode 6: Phrase-End Turnaround Cascade (leaves beat 1-2 open for the hook, answers on 10..15)
        _ => {
            notes.push(note(voicing[0], 0, 3, 56)?);
            for (idx, &start) in [10_u16, 12, 14].iter().enumerate() {
                if !lead_active_at(start) || idx == 2 {
                    let pitch = voicing[count - 1 - (idx % count)];
                    notes.push(note(pitch, start, 2, 62)?);
                }
            }
        }
    }
    Ok(notes)
}

/// Crystal upper-octave anime/Eurobeat sparkle arpeggio for high-energy chorus, solo & buildup sections.
pub(super) fn sparkle_arp(
    voicing: &[i16],
    kind: super::config::SectionKind,
    energy: u8,
) -> Result<Vec<Note>, String> {
    use super::config::SectionKind;
    if voicing.is_empty()
        || energy < 3
        || matches!(kind, SectionKind::VerseA | SectionKind::Breakdown)
    {
        return Ok(Vec::new());
    }
    let count = voicing.len();
    let mut notes = Vec::with_capacity(8);
    for (idx, &start) in [0_u16, 2, 4, 6, 8, 10, 12, 14].iter().enumerate() {
        let base = voicing[(idx * 2 + idx / count) % count];
        let pitch = (base + 12).clamp(60, 96);
        let vel = if start.is_multiple_of(4) { 58 } else { 46 };
        notes.push(note(pitch, start, 1, vel)?);
    }
    Ok(notes)
}
