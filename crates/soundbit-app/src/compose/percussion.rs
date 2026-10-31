//! Recognizable drum grooves with section dynamics and phrase-end fills.
use super::{Role, SongDesign, parts::note, rhythm::Rng};
use soundbit_core::Note;

pub(super) fn drums(
    design: &SongDesign,
    role: Role,
    bar: usize,
    fill: bool,
    rng: &mut Rng,
) -> Result<([Vec<Note>; 3], [bool; 16]), String> {
    let sparse = matches!(role, Role::Bridge | Role::Breakdown);
    let groove = if sparse {
        "half_time"
    } else {
        design.groove.as_str()
    };
    let (kick_steps, snare_steps): (&[usize], &[usize]) = match groove {
        "four_on_floor" => (&[0, 4, 8, 12], &[4, 12]),
        "breakbeat" => (&[0, 6, 9], &[4, 12]),
        "tresillo" => (&[0, 6, 12], &[4, 10]),
        "half_time" => (&[0, 10], &[8]),
        "driving" => (&[0, 2, 8, 10], &[4, 12]),
        _ => (&[0, 3, 8, 11], &[4, 12]),
    };
    let mut kicks = [false; 16];
    let mut snare = [0_u8; 16];
    for &step in kick_steps {
        kicks[step] = true;
    }
    for &step in snare_steps {
        snare[step] = if sparse { 75 } else { 96 };
    }
    if !sparse && bar % 2 == 1 && rng.chance(45) {
        kicks[14] = true;
    }
    if fill && !sparse {
        let fill_steps: &[usize] = match rng.pick(3) {
            0 => &[13, 15],
            1 => &[10, 14, 15],
            _ => &[11, 14],
        };
        for &step in fill_steps {
            if snare[step] == 0 {
                snare[step] = 58;
            }
        }
    }
    let stride = if sparse {
        4
    } else if groove == "driving" || role == Role::Climax {
        1
    } else {
        2
    };
    let mut lanes: [Vec<Note>; 3] = std::array::from_fn(|_| Vec::new());
    for i in 0..16 {
        let time = u16::try_from(i).unwrap_or(0);
        if kicks[i] {
            lanes[0].push(note(36, time, 1, if sparse { 84 } else { 105 })?);
        }
        if snare[i] > 0 {
            lanes[1].push(note(81, time, 1, snare[i])?);
        }
        let hat_on = if groove == "four_on_floor" {
            i % 4 == 2
        } else {
            i % stride == 0
        };
        if hat_on {
            lanes[2].push(note(
                102,
                time,
                1,
                if i % 4 == 0 {
                    60
                } else if i % 2 == 0 {
                    48
                } else {
                    32
                },
            )?);
        }
    }
    Ok((lanes, kicks))
}
