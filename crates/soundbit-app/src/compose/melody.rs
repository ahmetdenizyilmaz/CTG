//! Procedural motifs, answering phrases, breathing space and melodic development.
use super::{
    Role, SongDesign,
    parts::{BarContext, note},
    rhythm::Rng,
};
use soundbit_core::Note;

pub(super) struct Melody {
    motif: Vec<i16>,
    rhythm: Vec<u16>,
    previous: i16,
    previous_chord: Vec<i16>,
}

impl Melody {
    pub(super) fn new(rng: &mut Rng, design: &SongDesign) -> Self {
        let count = 5 + rng.pick(4);
        let mut motif = vec![i16::try_from(rng.pick(5)).unwrap_or(0)];
        let contour = rng.pick(4);
        for i in 1..count {
            let direction = match contour {
                0 => {
                    if i < count / 2 {
                        1
                    } else {
                        -1
                    }
                }
                1 => -1,
                2 => 1,
                _ => {
                    if rng.chance(50) {
                        1
                    } else {
                        -1
                    }
                }
            };
            let step = [0, 1, 1, 2, 3][rng.pick(5)];
            let last = motif[i - 1];
            let motion = if i > 1 && (last - motif[i - 2]).abs() > 2 {
                -(last - motif[i - 2]).signum()
            } else {
                direction * step
            };
            motif.push((last + motion).clamp(-2, 8));
        }
        if motif.iter().all(|n| *n == motif[0]) {
            motif[2] += 2;
        }
        let span = u16::try_from(design.phrase_bars * 16).unwrap_or(32);
        let mut ranked: Vec<_> = (0..span)
            .map(|t| {
                let weight = if t % 4 == 0 {
                    4
                } else if t % 2 == 0 {
                    3
                } else {
                    1
                };
                (t, rng.pick(32) + weight * 5)
            })
            .collect();
        ranked.sort_by_key(|&(_, score)| std::cmp::Reverse(score));
        let mut rhythm: Vec<_> = ranked.into_iter().take(count).map(|(t, _)| t).collect();
        rhythm.sort_unstable();
        rhythm[0] = if rng.chance(75) { 0 } else { rhythm[0].min(3) };
        Self {
            motif,
            rhythm,
            previous: design.register + 7,
            previous_chord: Vec::new(),
        }
    }

    pub(super) fn motif(&self) -> Vec<i16> {
        self.motif.clone()
    }

    fn events(&self, context: &BarContext<'_>) -> Vec<(u16, usize)> {
        let sparse = context.role == Role::Bridge;
        let stretch = if sparse { 2 } else { 1 };
        let span = context.design.phrase_bars * stretch;
        let local = context.bar % span;
        let mut events: Vec<_> = self
            .rhythm
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                let time = usize::from(*t) * stretch;
                (time / 16 == local).then_some((u16::try_from(time % 16).unwrap_or(0), i))
            })
            .collect();
        // The second two-bar phrase answers the first; occasional pickups vary the return.
        if !sparse && context.bar % 4 >= 2 && context.design.answer == 1 {
            for (time, _) in &mut events {
                *time = (*time + 1).min(15);
            }
        }
        if context.role == Role::Climax {
            let additions: Vec<_> = events
                .windows(2)
                .filter_map(|w| (w[1].0 - w[0].0 >= 3).then_some((w[0].0 + 2, w[0].1)))
                .collect();
            events.extend(additions);
        }
        events.sort_unstable();
        events.dedup_by_key(|e| e.0);
        events
    }

    fn pitch(
        &self,
        context: &BarContext<'_>,
        start: u16,
        index: usize,
        held: bool,
        rng: &mut Rng,
    ) -> i16 {
        let scale = context.scale;
        let intervals = scale.kind.intervals();
        let mut degree = if context.config.motif_development {
            self.motif[index]
        } else {
            i16::try_from(rng.pick(intervals.len())).unwrap_or(0)
        };
        if context.role == Role::Contrast {
            degree = match context.design.answer {
                0 => self.motif[0] * 2 - degree,
                1 => degree + 2,
                2 => self.motif[self.motif.len() - 1 - index],
                _ => degree - 1,
            };
        } else if context.bar % 4 >= 2 && index + 2 >= self.motif.len() {
            degree -= 1;
        }
        let size = i16::try_from(intervals.len()).unwrap_or(7);
        let pc = i16::from(intervals[usize::try_from(degree.rem_euclid(size)).unwrap_or(0)]);
        let desired = context.design.register
            + i16::from(scale.root.get())
            + pc
            + degree.div_euclid(size) * 12
            + if context.role == Role::Climax { 12 } else { 0 };
        let strong = start.is_multiple_of(4) || held || index + 1 == self.motif.len();
        (48_i16..=88)
            .filter(|n| {
                if strong {
                    context.chord.tones.contains(&(n % 12))
                } else {
                    intervals
                        .iter()
                        .any(|d| (i16::from(scale.root.get()) + i16::from(*d)) % 12 == n % 12)
                }
            })
            .min_by_key(|n| {
                let motion = (n - self.previous).abs();
                (n - desired).abs() * 4
                    + if context.config.voice_leading {
                        motion + (motion - 7).max(0) * 4
                    } else {
                        0
                    }
            })
            .unwrap_or(60 + context.chord.root)
    }

    pub(super) fn bar(
        &mut self,
        context: &BarContext<'_>,
        rng: &mut Rng,
    ) -> Result<Vec<Note>, String> {
        if context.role == Role::Breakdown {
            return Ok(Vec::new());
        }
        let prepared = self.previous;
        let events = self.events(context);
        let mut pitches = Vec::new();
        for (event, &(start, index)) in events.iter().enumerate() {
            let duration = events.get(event + 1).map_or(16, |e| e.0) - start;
            let pitch = self.pitch(context, start, index, duration >= 4, rng);
            pitches.push((start, pitch));
            self.previous = pitch;
        }
        self.suspension(context, prepared, &mut pitches);
        if context.final_bar && context.config.cadences {
            pitches.retain(|(start, _)| *start < 8);
            pitches.push((8, 72 + context.chord.root));
        }
        self.previous_chord.clone_from(&context.chord.tones);
        let mut notes = Vec::new();
        for (index, &(start, pitch)) in pitches.iter().enumerate() {
            let gap = pitches.get(index + 1).map_or(16, |p| p.0) - start;
            let length = if context.final_bar || context.role == Role::Bridge {
                gap
            } else {
                match context.design.articulation {
                    0 => gap,
                    1 => gap.saturating_sub(1).max(1),
                    _ => gap.min(3),
                }
            };
            let velocity = if context.role == Role::Bridge {
                72
            } else if start % 4 == 0 {
                96
            } else {
                82
            };
            notes.push(note(pitch, start, length, velocity)?);
        }
        if let Some(last) = notes.last() {
            self.previous = i16::from(last.pitch.get());
        }
        Ok(notes)
    }

    fn suspension(&self, context: &BarContext<'_>, prepared: i16, pitches: &mut Vec<(u16, i16)>) {
        if !context.config.suspensions || context.role != Role::Bridge || context.bar == 0 {
            return;
        }
        let Some(&(_, target)) = pitches.first() else {
            return;
        };
        if (1..=2).contains(&(prepared - target))
            && self.previous_chord.contains(&(prepared % 12))
            && !context.chord.tones.contains(&(prepared % 12))
        {
            pitches.retain(|(start, _)| *start != 0 && *start != 2);
            pitches.extend([(0, prepared), (2, target)]);
            pitches.sort_unstable();
        }
    }
}
