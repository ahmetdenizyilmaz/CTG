//! Complementary textures that leave space for the lead.
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
    if matches!(style, "offbeat_stabs" | "chord_pulses") {
        let starts: &[u16] = if style == "offbeat_stabs" {
            match context.bar % 3 {
                0 => &[2, 7, 10, 14],
                1 => &[3, 6, 11],
                _ => &[2, 6, 10, 15],
            }
        } else {
            &[0, 8]
        };
        for &start in starts {
            for &pitch in voicing {
                notes.push(note(
                    pitch,
                    start,
                    if style == "chord_pulses" { 4 } else { 1 },
                    58,
                )?);
            }
        }
    } else {
        let stride = if melody.len() > 5 { 4 } else { 2 };
        for start in (0..16_u16).step_by(stride) {
            if melody.iter().any(|n| n.start == start) && start % 8 != 0 {
                continue;
            }
            let step = usize::from(start / 2);
            let count = voicing.len();
            let index = match style {
                "falling_arp" => count - 1 - step % count,
                "pendulum" => {
                    let phase = step % (count * 2 - 2);
                    if phase < count {
                        phase
                    } else {
                        count * 2 - 2 - phase
                    }
                }
                "broken_chords" => [0, count - 1, 1, count - 1][step % 4],
                _ => step % count,
            };
            notes.push(note(
                voicing[index],
                start,
                2,
                if step % 2 == 0 { 62 } else { 50 },
            )?);
        }
    }
    Ok(notes)
}
