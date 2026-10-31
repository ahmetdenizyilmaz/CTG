//! Rhythmic cells form a hook, an answer, then a developed return.
use super::{Role, SongDesign, parts::BarContext, rhythm::Rng};

pub(super) struct Idea {
    pub(super) degrees: Vec<i16>,
    rhythm: Vec<u16>,
}

impl Idea {
    pub(super) fn new(rng: &mut Rng, design: &SongDesign) -> Self {
        // Two-beat cells, with internal space. These are rhythm vocabulary, not tunes.
        let cells: &[&[u16]] = &[
            &[0, 3, 6],
            &[0, 2, 3, 6],
            &[0, 2, 6],
            &[1, 4, 6],
            &[0, 3, 4, 7],
            &[0, 2, 4],
            &[2, 3, 6],
            &[0, 6],
            &[0, 1, 4, 6],
            &[0, 4, 7],
            &[1, 3, 6],
            &[0, 2, 5],
        ];
        let a = cells[rng.pick(cells.len())];
        let b = cells[rng.pick(cells.len())];
        let span = u16::try_from(design.phrase_bars * 16).unwrap_or(32);
        let mut rhythm = Vec::new();
        for cell in 0..span / 8 {
            let source = if cell % 2 == 0 { a } else { b };
            rhythm.extend(source.iter().map(|t| cell * 8 + t));
        }
        // A long destination and a breath make the cell into a phrase.
        rhythm.retain(|t| *t < span - 8);
        rhythm.push(span - if rng.chance(50) { 8 } else { 6 });
        let pivot = i16::try_from(rng.pick(5)).unwrap_or(0);
        let contour = rng.pick(4);
        let mut degrees = vec![pivot];
        for i in 1..rhythm.len() {
            let previous = degrees[i - 1];
            let direction = match contour {
                0 => {
                    if i < rhythm.len() / 2 {
                        1
                    } else {
                        -1
                    }
                }
                1 => {
                    if i < rhythm.len() / 2 {
                        -1
                    } else {
                        1
                    }
                }
                2 => 1,
                _ => -1,
            };
            // Repeat the opening gesture once, then change its destination.
            let degree = if i >= a.len() && i < a.len() * 2 && rng.chance(70) {
                degrees[i - a.len()] + i16::from(contour == 2)
            } else {
                let step = [0, 1, 1, 2, 3][rng.pick(5)];
                let motion = if i > 1 && (previous - degrees[i - 2]).abs() > 2 {
                    -(previous - degrees[i - 2]).signum()
                } else {
                    direction * step
                };
                (previous + motion).clamp(-2, 9)
            };
            degrees.push(degree);
        }
        if degrees.iter().all(|n| *n == pivot) {
            degrees[1] += 2;
        }
        Self { degrees, rhythm }
    }

    pub(super) fn events(&self, context: &BarContext<'_>) -> Vec<(u16, usize)> {
        let bridge = context.role == Role::Bridge;
        let stretch = if bridge { 2 } else { 1 };
        let span = context.design.phrase_bars * stretch;
        let local = context.bar % span;
        let mut events: Vec<_> = self
            .rhythm
            .iter()
            .enumerate()
            .filter_map(|(i, time)| {
                let time = usize::from(*time) * stretch;
                (time / 16 == local).then_some((u16::try_from(time % 16).unwrap_or(0), i))
            })
            .collect();
        if bridge {
            // Keep the slow theme spacious even when the original hook is dense.
            let mut last = None;
            events.retain(|(t, _)| {
                let keep = last.is_none_or(|previous| *t >= previous + 4);
                if keep {
                    last = Some(*t);
                }
                keep
            });
        } else {
            // A returning phrase keeps its first cell and develops the second.
            let developed = context.bar % 8 >= 4 || context.role == Role::Climax;
            if developed && local == 0 {
                events.retain(|(t, _)| *t < 8);
                let fragment: Vec<_> = events
                    .iter()
                    .take(3)
                    .map(|(t, i)| (8 + t / 2, *i))
                    .collect();
                events.extend(fragment);
            }
            if context.role == Role::Contrast && context.design.answer == 1 {
                for (t, _) in &mut events {
                    *t = (*t + 1).min(15);
                }
            }
            if context.role == Role::Climax && local + 1 < span {
                let runs: Vec<_> = events
                    .windows(2)
                    .filter_map(|w| {
                        (w[1].0 - w[0].0 >= 3)
                            .then_some((w[1].0 - 1, (w[0].1 + 1) % self.degrees.len()))
                    })
                    .collect();
                events.extend(runs);
            }
        }
        events.sort_unstable();
        events.dedup_by_key(|e| e.0);
        events
    }
}
