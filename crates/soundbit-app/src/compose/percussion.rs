//! Recognizable drum grooves with section dynamics, accelerating risers and studio percussion.
use super::{Role, SongDesign, config::SectionKind, parts::note, rhythm::Rng};
use soundbit_core::Note;

pub(super) fn drums(
    design: &SongDesign,
    role: Role,
    bar: usize,
    fill: bool,
    rng: &mut Rng,
) -> Result<([Vec<Note>; 3], [bool; 16]), String> {
    drums_with_section(design, role, SectionKind::Chorus, bar, 4, fill, false, rng)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn drums_with_section(
    design: &SongDesign,
    role: Role,
    kind: SectionKind,
    bar: usize,
    section_bars: usize,
    fill: bool,
    riser_builds: bool,
    rng: &mut Rng,
) -> Result<([Vec<Note>; 3], [bool; 16]), String> {
    let sparse = matches!(role, Role::Bridge | Role::Breakdown);
    let groove = if sparse {
        "half_time"
    } else {
        design.groove.as_str()
    };
    let (kick_steps, snare_steps): (&[usize], &[usize]) = match groove {
        "four_on_floor" | "acid_warehouse" | "french_disco" | "melbourne_offbeat"
        | "house_m1" | "melodic_techno" | "eurobeat_156" | "anime_techno_166" => {
            (&[0, 4, 8, 12], &[4, 12])
        }
        "liquid_dnb_174" | "uk_garage_2step" => (&[0, 10], &[4, 12]),
        "midtempo_stomp" => {
            if role == Role::Climax {
                (&[0, 4, 8, 12], &[4, 8, 12])
            } else {
                (&[0, 4, 8, 12], &[8])
            }
        }
        "balkan_brass" => (&[0, 4, 8, 12, 14], &[2, 4, 10, 12]),
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
        snare[step] = if sparse { 75 } else { 98 };
    }
    if !sparse && groove == "liquid_dnb_174" && bar % 2 == 1 {
        snare[7] = 52;
        snare[9] = 48;
    }
    if !sparse
        && !matches!(
            groove,
            "liquid_dnb_174" | "uk_garage_2step" | "balkan_brass" | "eurobeat_156" | "anime_techno_166"
        )
        && bar % 2 == 1
        && rng.chance(45)
    {
        kicks[14] = true;
    }

    // Accelerating snare riser rolls on the final 2 bars of PreChorus or Breakdown!
    let in_riser = riser_builds
        && matches!(kind, SectionKind::PreChorus | SectionKind::Breakdown)
        && section_bars >= 2
        && bar + 2 >= section_bars;
    if in_riser {
        if bar + 2 == section_bars {
            for (idx, step) in [0_usize, 2, 4, 6, 8, 10, 12, 14].into_iter().enumerate() {
                snare[step] = 64 + u8::try_from(idx * 6).unwrap_or(0);
            }
        } else {
            // Final bar before drop: kick drops out at beat 3, fast 16th snare roll, dramatic breath at 14..15!
            kicks[8] = false;
            kicks[12] = false;
            for step in 0..13 {
                snare[step] = 72 + u8::try_from(step * 4).unwrap_or(0);
            }
            snare[14] = 0;
            snare[15] = 0;
        }
    } else if fill && !sparse {
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
    } else if matches!(
        groove,
        "driving" | "melodic_techno" | "acid_warehouse" | "eurobeat_156" | "anime_techno_166"
    ) || role == Role::Climax
    {
        1
    } else {
        2
    };
    let mut lanes: [Vec<Note>; 3] = std::array::from_fn(|_| Vec::new());
    for i in 0..16 {
        let time = u16::try_from(i).unwrap_or(0);
        if kicks[i] {
            lanes[0].push(note(36, time, 1, if sparse { 84 } else { 108 })?);
        }
        if snare[i] > 0 {
            lanes[1].push(note(81, time, 1, snare[i])?);
        }
        let hat_on = if in_riser && bar + 1 == section_bars && i >= 12 {
            false
        } else {
            match groove {
                "four_on_floor" | "melbourne_offbeat" | "balkan_brass" => i % 4 == 2,
                "french_disco" | "house_m1" => {
                    i % 4 == 2 || i == 15 || (groove == "house_m1" && i == 7)
                }
                "uk_garage_2step" => matches!(i, 2 | 5 | 7 | 10 | 13 | 15),
                _ => i % stride == 0,
            }
        };
        if hat_on {
            let hat_vel = if matches!(
                groove,
                "melodic_techno" | "acid_warehouse" | "eurobeat_156" | "anime_techno_166"
            ) && i % 4 == 2
            {
                76
            } else if i % 4 == 0 {
                54
            } else if i % 2 == 0 {
                50
            } else {
                34
            };
            lanes[2].push(note(102, time, 1, hat_vel)?);
        }
    }
    Ok((lanes, kicks))
}

/// Studio layers: Initial D / Eurobeat offbeat cowbell (`[0]`) and section-drop crash cymbal (`[1]`).
pub(super) fn studio_percussion(
    kind: SectionKind,
    local_bar: usize,
    energy: u8,
) -> Result<[Vec<Note>; 2], String> {
    let mut cowbell = Vec::new();
    let mut crash = Vec::new();
    if energy >= 4
        && matches!(
            kind,
            SectionKind::IntroRiff | SectionKind::Chorus | SectionKind::Solo | SectionKind::FinalChorus
        )
    {
        for step in [2_u16, 6, 10, 14] {
            cowbell.push(note(72, step, 1, 94)?);
        }
        if local_bar % 4 == 3 {
            cowbell.push(note(72, 7, 1, 84)?);
            cowbell.push(note(72, 15, 1, 96)?);
        }
    }
    if (local_bar == 0 || (local_bar == 4 && energy >= 5))
        && matches!(
            kind,
            SectionKind::IntroRiff | SectionKind::Chorus | SectionKind::Solo | SectionKind::FinalChorus
        )
    {
        crash.push(note(60, 0, 8, 110)?);
    }
    Ok([cowbell, crash])
}
