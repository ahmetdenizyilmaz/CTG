//! Phrase identity, destinations, expressive spacing and melodic development.
use super::{
    Role, SongDesign,
    parts::{BarContext, note},
    phrase::Idea,
    pitch_path::{self, Target},
    rhythm::Rng,
};
use soundbit_core::Note;

pub(super) struct Melody {
    idea: Idea,
    previous: i16,
    motion: i16,
    previous_chord: Vec<i16>,
}

impl Melody {
    pub(super) fn new(rng: &mut Rng, design: &SongDesign) -> Self {
        Self {
            idea: Idea::new(rng, design),
            previous: design.register + 7,
            motion: 0,
            previous_chord: Vec::new(),
        }
    }

    pub(super) fn motif(&self) -> Vec<i16> {
        self.idea.degrees.clone()
    }

    fn target(&self, context: &BarContext<'_>, index: usize, rng: &mut Rng) -> i16 {
        let motif = &self.idea.degrees;
        let intervals = context.scale.kind.intervals();
        let mut degree = if context.config.motif_development {
            motif[index]
        } else {
            i16::try_from(rng.pick(intervals.len())).unwrap_or(0)
        };
        if context.role == Role::Contrast {
            degree = match context.design.answer {
                0 => motif[0] * 2 - degree,
                1 => degree + 2,
                2 => motif[motif.len() - 1 - index],
                _ => degree - 1,
            };
        } else if context.bar % 4 >= 2 && index + 2 >= motif.len() {
            degree -= 1;
        }
        // Climax builds toward its high point instead of moving every note up an octave.
        let lift = if context.role == Role::Climax {
            if context.bar % 4 >= 2 { 7 } else { 3 }
        } else {
            0
        };
        let size = i16::try_from(intervals.len()).unwrap_or(7);
        let pc = i16::from(intervals[usize::try_from(degree.rem_euclid(size)).unwrap_or(0)]);
        context.design.register
            + i16::from(context.scale.root.get())
            + pc
            + degree.div_euclid(size) * 12
            + lift
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
        let events = self.idea.events(context);
        let phrase_end = (context.bar + 1).is_multiple_of(context.design.phrase_bars);
        let targets: Vec<_> = events
            .iter()
            .enumerate()
            .map(|(i, &(start, index))| {
                let gap = events.get(i + 1).map_or(16, |e| e.0) - start;
                Target {
                    desired: self.target(context, index, rng),
                    stable: start.is_multiple_of(4) || gap >= 4 || i + 1 == events.len(),
                    cadence: context.config.cadences
                        && context.bar % 4 == 3
                        && i + 1 == events.len(),
                }
            })
            .collect();
        let planned = pitch_path::plan(context, &targets, self.previous, self.motion);
        let mut pitches: Vec<_> = events.iter().zip(planned).map(|(e, p)| (e.0, p)).collect();
        self.suspension(context, prepared, &mut pitches);
        if context.final_bar && context.config.cadences {
            pitches.retain(|(start, _)| *start < 8);
            let previous = pitches.last().map_or(prepared, |p| p.1);
            let tonic = (48_i16..=88)
                .filter(|p| p % 12 == context.chord.root)
                .min_by_key(|p| (p - previous).abs())
                .unwrap_or(72 + context.chord.root);
            pitches.push((8, tonic));
        }
        self.previous_chord.clone_from(&context.chord.tones);
        let mut notes = Vec::new();
        for (index, &(start, pitch)) in pitches.iter().enumerate() {
            let gap = pitches.get(index + 1).map_or(16, |p| p.0) - start;
            let last = index + 1 == pitches.len();
            let length = if context.final_bar || context.role == Role::Bridge {
                gap
            } else if phrase_end && last {
                // Phrase endpoint rings, then leaves room for bass/drum response.
                gap.saturating_sub(2).max(1)
            } else {
                match context.design.articulation {
                    0 => gap,
                    1 => gap.saturating_sub(1).max(1),
                    _ => gap.min(3),
                }
            };
            let base: u8 = if context.role == Role::Bridge {
                70
            } else if context.role == Role::Climax {
                98
            } else {
                88
            };
            let velocity = if index == 0 || start.is_multiple_of(4) {
                base + 6
            } else if gap == 1 {
                base - 12
            } else {
                base - 4
            };
            notes.push(note(pitch, start, length, velocity)?);
            self.motion = pitch - self.previous;
            self.previous = pitch;
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
