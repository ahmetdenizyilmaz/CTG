//! Weighted progressions, functional color and economical chord voicings.
use super::{ComposeConfig, config::Vibe, rhythm::Rng};
use serde::Serialize;
use soundbit_core::{Scale, ScaleKind};

/// Function of a section in the arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Main recurring hook.
    Verse,
    /// Related contrasting material.
    Contrast,
    /// Reflective, less dense statement.
    Bridge,
    /// Rhythm section alone.
    Breakdown,
    /// Developed hook at peak energy.
    Climax,
}

#[derive(Debug, Clone)]
pub(super) struct Chord {
    pub(super) root: i16,
    pub(super) tones: Vec<i16>,
    pub(super) label: String,
}

// Adapted from ZzFX Studio's weighted progression vocabulary (MIT, Justin Walsh).
const ADVENTURE: [([usize; 4], usize); 4] = [
    ([0, 4, 5, 3], 3),
    ([0, 3, 4, 0], 2),
    ([0, 5, 3, 4], 2),
    ([0, 3, 0, 4], 1),
];
const BATTLE: [([usize; 4], usize); 4] = [
    ([0, 2, 6, 5], 3),
    ([0, 3, 4, 0], 2),
    ([0, 6, 5, 4], 2),
    ([0, 4, 3, 6], 1),
];
const DUNGEON: [([usize; 4], usize); 4] = [
    ([0, 3, 0, 4], 3),
    ([0, 6, 0, 5], 2),
    ([0, 5, 3, 4], 2),
    ([0, 3, 5, 6], 1),
];
const TITLE: [([usize; 4], usize); 4] = [
    ([0, 4, 5, 3], 3),
    ([0, 3, 4, 0], 3),
    ([0, 5, 3, 4], 2),
    ([0, 2, 3, 4], 1),
];
const BOSS: [([usize; 4], usize); 4] = [
    ([0, 6, 5, 6], 3),
    ([0, 2, 5, 4], 2),
    ([0, 4, 3, 6], 2),
    ([0, 3, 6, 4], 1),
];
const FUNK: [([usize; 4], usize); 4] = [
    ([0, 3, 0, 3], 3),
    ([0, 6, 3, 0], 2),
    ([0, 3, 5, 4], 2),
    ([0, 2, 3, 4], 1),
];

pub(super) fn progression(vibe: Vibe, rng: &mut Rng) -> [usize; 4] {
    // Additional functional routes: falling thirds, ii-V motion, plagal cycles
    // and minor-key mediant movement. Keep one route per recurring section role.
    if rng.chance(45) {
        let alternatives = match vibe {
            Vibe::Funk => [[0, 3, 1, 4], [1, 4, 0, 5], [0, 0, 3, 4], [3, 0, 1, 4]],
            Vibe::Dungeon => [[5, 3, 0, 0], [0, 0, 5, 6], [3, 0, 6, 0], [0, 2, 6, 3]],
            Vibe::Battle | Vibe::Boss => [[0, 5, 2, 6], [3, 5, 4, 0], [0, 3, 1, 4], [5, 6, 0, 4]],
            _ => [[5, 3, 0, 4], [1, 4, 0, 5], [3, 4, 2, 5], [0, 2, 5, 3]],
        };
        return alternatives[rng.pick(alternatives.len())];
    }
    let choices = match vibe {
        Vibe::Adventure => &ADVENTURE,
        Vibe::Battle => &BATTLE,
        Vibe::Dungeon => &DUNGEON,
        Vibe::TitleScreen => &TITLE,
        Vibe::Boss => &BOSS,
        Vibe::Funk => &FUNK,
    };
    let mut selected = rng.pick(choices.iter().map(|(_, w)| w).sum());
    for &(degrees, weight) in choices {
        if selected < weight {
            return degrees;
        }
        selected -= weight;
    }
    choices[0].0
}

pub(super) fn harmonic_scale(scale: Scale) -> &'static [u8] {
    // Pentatonic is a melody palette, not a seven-degree chord spelling system.
    match scale.kind {
        ScaleKind::MajorPentatonic => ScaleKind::Major.intervals(),
        ScaleKind::MinorPentatonic => ScaleKind::NaturalMinor.intervals(),
        _ => scale.kind.intervals(),
    }
}

pub(super) fn chord(scale: Scale, degree: usize, colors: bool) -> Chord {
    let intervals = harmonic_scale(scale);
    let root = (i16::from(scale.root.get()) + i16::from(intervals[degree % 7])) % 12;
    let mut tones: Vec<_> = [degree, degree + 2, degree + 4]
        .into_iter()
        .map(|d| (i16::from(scale.root.get()) + i16::from(intervals[d % 7])) % 12)
        .collect();
    let third = (tones[1] - root).rem_euclid(12);
    let fifth = (tones[2] - root).rem_euclid(12);
    let quality = if fifth == 6 {
        "dim"
    } else if fifth == 8 {
        "aug"
    } else if third == 3 {
        "m"
    } else {
        ""
    };
    let mut label = format!("{}{quality}", pitch_name(root));
    if colors && fifth == 7 {
        let extra = if degree == 0 {
            (root + 2) % 12
        } else {
            (i16::from(scale.root.get()) + i16::from(intervals[(degree + 6) % 7])) % 12
        };
        tones.push(extra);
        label.push_str(if degree == 0 {
            "add9"
        } else if (extra - root).rem_euclid(12) == 11 {
            "maj7"
        } else {
            "7"
        });
    }
    Chord { root, tones, label }
}

pub(super) fn bar_chord(
    config: &ComposeConfig,
    scale: Scale,
    degrees: [usize; 4],
    role: Role,
    bar: usize,
    last_section: bool,
    phrase_bars: usize,
) -> Chord {
    if config.cadences && last_section && bar == 3 {
        return chord(scale, 0, config.chord_colors);
    }
    if config.cadences && last_section && bar == 2 {
        return dominant(i16::from(scale.root.get()), "V7");
    }
    if config.modal_mixture
        && role == Role::Bridge
        && bar == 2
        && matches!(scale.kind, ScaleKind::Major | ScaleKind::MajorPentatonic)
    {
        let root = (i16::from(scale.root.get()) + 5) % 12;
        return Chord {
            root,
            tones: vec![root, (root + 3) % 12, (root + 7) % 12],
            label: format!("{}m (borrowed iv)", pitch_name(root)),
        };
    }
    if config.secondary_dominants
        && role == Role::Contrast
        && Some(bar)
            == (0..phrase_bars.min(4).saturating_sub(1))
                .rev()
                .find(|i| degrees[i + 1] != 0)
    {
        return dominant(
            chord(scale, degrees[bar + 1], false).root,
            "secondary dominant",
        );
    }
    if config.cadences && role == Role::Contrast && bar == 3 && !config.secondary_dominants {
        return dominant(i16::from(scale.root.get()), "half cadence");
    }
    chord(scale, degrees[bar], config.chord_colors)
}

fn dominant(target: i16, function: &str) -> Chord {
    let root = (target + 7) % 12;
    Chord {
        root,
        tones: [0, 4, 7, 10].map(|i| (root + i) % 12).to_vec(),
        label: format!("{}7 ({function})", pitch_name(root)),
    }
}

pub(super) fn pitch_name(pc: i16) -> &'static str {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
    ];
    NAMES[usize::try_from(pc.rem_euclid(12)).unwrap_or(0)]
}

pub(super) fn voicing(chord: &Chord, previous: &[i16], smooth: bool) -> Vec<i16> {
    let mut candidates = Vec::new();
    for inversion in 0..chord.tones.len() {
        let mut pitches = Vec::new();
        for i in 0..chord.tones.len() {
            let pc = chord.tones[(inversion + i) % chord.tones.len()];
            let mut pitch = 48 + pc;
            while pitches.last().is_some_and(|&last| pitch <= last) {
                pitch += 12;
            }
            pitches.push(pitch);
        }
        for shift in [-12, 0, 12] {
            let moved: Vec<_> = pitches.iter().map(|n| n + shift).collect();
            if moved[0] >= 48 && moved[moved.len() - 1] <= 79 {
                candidates.push(moved);
            }
        }
    }
    if !smooth || previous.is_empty() {
        return candidates
            .into_iter()
            .next()
            .unwrap_or_else(|| vec![60 + chord.root]);
    }
    candidates
        .into_iter()
        .min_by_key(|p| {
            p.iter()
                .enumerate()
                .map(|(i, n)| (n - previous[i.min(previous.len() - 1)]).abs())
                .sum::<i16>()
        })
        .unwrap_or_else(|| vec![60 + chord.root])
}
