//! Procedural motifs, 10 distinct melodic contours, motif form grammar and breathing space.
use super::{
    Role, SongDesign,
    config::{MelodyContour, MotifForm},
    parts::{BarContext, note},
    rhythm::Rng,
};
use soundbit_core::Note;

pub(super) struct Melody {
    motif: Vec<i16>,
    motif_b: Vec<i16>,
    rhythm: Vec<u16>,
    contour: MelodyContour,
    form: MotifForm,
    previous: i16,
    previous_chord: Vec<i16>,
}

impl Melody {
    pub(super) fn new(rng: &mut Rng, design: &SongDesign, config: &super::ComposeConfig) -> Self {
        let contour = MelodyContour::from_name(&design.melody_contour);
        let contour = if contour == MelodyContour::Auto {
            MelodyContour::PedalPivot
        } else {
            contour
        };
        let form = MotifForm::from_name(&design.motif_form);
        let form = if form == MotifForm::Auto {
            MotifForm::Aaba
        } else {
            form
        };

        let anchor = i16::try_from(rng.pick(4)).unwrap_or(0);
        let motif = build_motif_for_contour(contour, anchor, rng);
        let motif_b = build_contrasting_motif(&motif, contour, rng);

        // Density scaled cleanly by rest_density (0..80): higher rest_density leaves more breathing gaps.
        let max_notes = usize::from(((100_u16.saturating_sub(u16::from(config.rest_density))) * 8 / 100).clamp(2, 8));
        let target_count = (motif.len()).min(max_notes).max(2);

        let span = u16::try_from(design.phrase_bars * 16).unwrap_or(32);
        let synco_bias = i32::from(config.syncopation);
        let mut ranked: Vec<_> = (0..span)
            .map(|t| {
                let in_bar = t % 16;
                let base_w = contour_rhythm_weight(contour, in_bar);
                let offbeat_bonus = if in_bar % 4 != 0 {
                    (synco_bias - 40) / 14
                } else {
                    (40 - synco_bias) / 18
                };
                let jitter = i32::try_from(rng.pick(28)).unwrap_or(0);
                (t, jitter + base_w * 6 + offbeat_bonus * 4)
            })
            .collect();
        ranked.sort_by_key(|&(_, score)| std::cmp::Reverse(score));
        let mut rhythm: Vec<_> = ranked.into_iter().take(target_count).map(|(t, _)| t).collect();
        rhythm.sort_unstable();

        // Anchor first onset according to contour identity (e.g. Offbeat rests on step 0!).
        if contour == MelodyContour::OffbeatSyncopated && rhythm[0] == 0 {
            rhythm[0] = 2;
        } else if matches!(contour, MelodyContour::AnacrusisPickup) {
            // Ensure late-bar pickup + downbeat hold when phrase starts
            if !rhythm.contains(&12) && !rhythm.contains(&14) {
                if let Some(last) = rhythm.last_mut() {
                    *last = span.saturating_sub(2);
                }
            }
            if rhythm[0] > 3 {
                rhythm[0] = 0;
            }
        } else if contour != MelodyContour::VocalChopSyllabic && rng.chance(72) {
            rhythm[0] = 0;
        }
        rhythm.sort_unstable();
        rhythm.dedup();

        Self {
            motif,
            motif_b,
            rhythm,
            contour,
            form,
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

        // Ensure a non-empty bar retains at least 2 expressive notes in active phrase sections.
        if events.is_empty() {
            events.push((
                if self.contour == MelodyContour::OffbeatSyncopated {
                    2
                } else {
                    0
                },
                0,
            ));
            events.push((8, 1 % self.motif.len()));
        }

        // Apply rhythmic variation on B/Response bars for CallResponse / ABAC / DualCallResponse
        if !sparse && context.bar % 2 == 1 && matches!(self.form, MotifForm::CallResponse | MotifForm::Abac) {
            for (idx, (time, _)) in events.iter_mut().enumerate() {
                if self.contour == MelodyContour::DualCallResponse && idx % 2 == 0 {
                    *time = (*time + 2).min(14);
                }
            }
        }
        events.sort_unstable_by_key(|e| e.0);
        events.dedup_by_key(|e| e.0);

        if context.role == Role::Climax {
            let additions: Vec<_> = events
                .windows(2)
                .filter_map(|w| {
                    (w[1].0.saturating_sub(w[0].0) >= 4).then_some((w[0].0 + 2, w[0].1 + 1))
                })
                .collect();
            events.extend(additions);
            events.sort_unstable_by_key(|e| e.0);
            events.dedup_by_key(|e| e.0);
        }
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
        let bar_slot = context.bar % 4;
        // Choose phrase A or B based on MotifForm grammar (AABA, ABAC, AABB, CallResponse)
        let active_motif: &[i16] = match self.form {
            MotifForm::Aaba => {
                if bar_slot == 2 {
                    &self.motif_b
                } else {
                    &self.motif
                }
            }
            MotifForm::Abac => {
                if bar_slot % 2 == 1 {
                    &self.motif_b
                } else {
                    &self.motif
                }
            }
            MotifForm::Aabb => {
                if bar_slot >= 2 {
                    &self.motif_b
                } else {
                    &self.motif
                }
            }
            MotifForm::CallResponse | MotifForm::Auto => {
                if bar_slot % 2 == 1 {
                    &self.motif_b
                } else {
                    &self.motif
                }
            }
        };

        let mut degree = if context.config.motif_development {
            active_motif[index % active_motif.len()]
        } else {
            i16::try_from(rng.pick(intervals.len())).unwrap_or(0)
        };

        if context.form_c_turnaround(bar_slot) {
            degree = active_motif[(index + 1) % active_motif.len()] + 1;
        } else if context.role == Role::Contrast {
            degree = match context.design.answer {
                0 => active_motif[0] * 2 - degree,
                1 => degree + 2,
                2 => active_motif[active_motif.len() - 1 - (index % active_motif.len())],
                _ => degree - 1,
            };
        } else if bar_slot == 3 && index + 1 == active_motif.len() {
            // Pull end of 4-bar phrase gently toward chord root / tonic
            degree = degree.clamp(-2, 4);
        }

        let size = i16::try_from(intervals.len().max(1)).unwrap_or(7);
        let pc = i16::from(intervals[usize::try_from(degree.rem_euclid(size)).unwrap_or(0)]);
        let climax_lift = if context.role == Role::Climax { 12 } else { 0 };
        let contour_oct = if self.contour == MelodyContour::DescendingCascade && bar_slot < 2 {
            12
        } else {
            0
        };
        let desired = (context.design.register
            + i16::from(scale.root.get())
            + pc
            + degree.div_euclid(size) * 12
            + climax_lift
            + contour_oct)
            .clamp(48, 92);

        let strong = start.is_multiple_of(4) || held || (index + 1) % active_motif.len() == 0;
        let max_motion = matches!(
            self.contour,
            MelodyContour::WideIntervalLeap | MelodyContour::VocalChopSyllabic
        );
        (48_i16..=90)
            .filter(|n| {
                if strong && context.chord.tones.iter().any(|t| *t == n % 12) {
                    true
                } else {
                    intervals
                        .iter()
                        .any(|d| (i16::from(scale.root.get()) + i16::from(*d)) % 12 == n % 12)
                }
            })
            .min_by_key(|n| {
                let motion = (n - self.previous).abs();
                let target_err = (n - desired).abs() * 5;
                if context.config.voice_leading && !max_motion {
                    target_err + motion + (motion - 7).max(0) * 4
                } else {
                    target_err
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
            let is_phrase_tail = index + 1 == pitches.len();
            let length = if context.final_bar {
                gap
            } else if context.role == Role::Bridge {
                gap.min(6)
            } else if is_phrase_tail {
                match self.contour {
                    MelodyContour::AnacrusisPickup | MelodyContour::WideIntervalLeap => gap.min(6),
                    MelodyContour::VocalChopSyllabic | MelodyContour::ExoticOrnamental => {
                        gap.min(2)
                    }
                    _ => gap.min(3),
                }
            } else {
                match self.contour {
                    MelodyContour::VocalChopSyllabic
                    | MelodyContour::ExoticOrnamental
                    | MelodyContour::RhythmicMorseHook => gap.min(2),
                    MelodyContour::AnacrusisPickup | MelodyContour::WideIntervalLeap => {
                        if start == 0 { gap.min(6) } else { gap.min(3) }
                    }
                    MelodyContour::PedalPivot | MelodyContour::BluesFunkRiff => {
                        gap.saturating_sub(1).clamp(1, 2)
                    }
                    _ => match context.design.articulation {
                        0 => gap.min(4),
                        1 => gap.saturating_sub(1).clamp(1, 3),
                        _ => gap.min(2),
                    },
                }
            };
            let velocity = if context.role == Role::Bridge {
                72
            } else if start % 4 == 0 {
                98
            } else if matches!(
                self.contour,
                MelodyContour::OffbeatSyncopated | MelodyContour::VocalChopSyllabic
            ) {
                94
            } else {
                82
            };
            notes.push(note(pitch, start, length.min(16 - start).max(1), velocity)?);
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

impl<'a> BarContext<'a> {
    fn form_c_turnaround(&self, bar_slot: usize) -> bool {
        bar_slot == 3 && self.design.motif_form == "abac"
    }
}

/// Multi-theme procedural song melody bank for 6 distinct section personalities:
/// 0 = Intro Brass Riff, 1 = Verse A, 2 = Verse B Continuation, 3 = Uprising Pre-Chorus,
/// 4 = Soaring Anime/Eurobeat Chorus ("Sabi"), 5 = Blazing Solo Guitar/Synth Duel.
pub(super) struct SemiProMelodyBank {
    seed: u32,
    previous_pitches: [i16; 6],
}

impl SemiProMelodyBank {
    pub(super) fn new(seed: u32) -> Self {
        Self {
            seed,
            previous_pitches: [72, 67, 71, 69, 76, 74],
        }
    }

    pub(super) fn bar(
        &mut self,
        context: &BarContext<'_>,
        theme_slot: u8,
        seed_offset: u32,
        kind: super::config::SectionKind,
    ) -> Result<Vec<Note>, String> {
        use super::config::SectionKind;
        if kind == SectionKind::Breakdown && context.bar % 4 == 0 {
            return Ok(Vec::new());
        }
        let slot = usize::from(theme_slot.min(5));
        let mut rng = Rng::new(
            self.seed
                ^ (u32::try_from(slot).unwrap_or(0).wrapping_mul(0x9E37_79B9))
                ^ seed_offset.wrapping_mul(0x517C_C1B7),
        );
        let scale = context.scale;
        let intervals = scale.kind.intervals();
        let bar_slot = context.bar % 4;
        let phrase_half = (context.bar / 4) % 2;

        let base_anchor = match slot {
            0 => i16::try_from(rng.pick(3)).unwrap_or(0) + 2, // Intro Riff bright anchor
            1 => i16::try_from(rng.pick(3)).unwrap_or(0),     // Verse A warm pocket
            2 => i16::try_from(rng.pick(3)).unwrap_or(0) + 3, // Verse B Continuation lifted +3 degrees!
            3 => i16::try_from(bar_slot * 2).unwrap_or(0),    // PreChorus staircase climbing every bar!
            4 => i16::try_from(rng.pick(3)).unwrap_or(0) + 5, // Chorus/Sabi high soaring range
            _ => i16::try_from(rng.pick(4)).unwrap_or(0) + 4, // Solo shred pocket
        };

        // 4-bar Question -> Continuation -> Peak -> Turnaround degrees (deterministic per slot + bar_slot)
        let mut degrees = Vec::with_capacity(12);
        let steps: Vec<u16> = match slot {
            // 0: Intro Synth Brass Riff (driving riff bars 0..2, 16th turnaround bar 3)
            0 => {
                if bar_slot == 3 {
                    vec![0, 2, 4, 6, 8, 10, 12, 14]
                } else if rng.chance(55) {
                    vec![0, 2, 3, 6, 8, 10, 12]
                } else {
                    vec![0, 3, 6, 8, 11, 14]
                }
            }
            // 1: Verse A lyrical melody with breathing space at bar end
            1 => {
                if bar_slot % 2 == 0 {
                    vec![0, 3, 6, 8, 10]
                } else {
                    vec![0, 2, 6, 9]
                }
            }
            // 2: Verse B Continuation (syncopated lift, answers Verse A higher up)
            2 => {
                if bar_slot == 3 {
                    vec![0, 2, 4, 6, 8, 10, 12]
                } else {
                    vec![0, 3, 4, 6, 10, 12]
                }
            }
            // 3: Pre-Chorus Uprising climb (accelerating on bar 2, climax hold on bar 3)
            3 => match bar_slot {
                0 | 1 => vec![0, 4, 8, 12],
                2 => vec![0, 2, 4, 6, 8, 10, 12],
                _ => vec![0, 8, 10, 12],
            },
            // 4: Soaring Chorus / FinalChorus ("Sabi")
            4 => {
                if kind == SectionKind::Breakdown {
                    vec![0, 6, 10]
                } else if bar_slot == 0 || bar_slot == 2 {
                    vec![0, 3, 6, 8, 11, 14]
                } else if bar_slot == 3 {
                    vec![0, 4, 6, 8, 12]
                } else {
                    vec![0, 2, 6, 8, 10, 12]
                }
            }
            // 5: Blazing Solo Guitar/Synth duel
            _ => {
                if bar_slot % 2 == 0 {
                    vec![0, 1, 2, 3, 4, 6, 8, 10, 12, 14]
                } else {
                    vec![0, 2, 3, 5, 6, 8, 9, 10, 12, 13, 14]
                }
            }
        };

        for idx in 0..steps.len() {
            let d = match slot {
                0 => {
                    // Brass riff: alternate anchor against chord 5th/octave + turnaround cascade
                    if bar_slot == 3 {
                        base_anchor + 7 - i16::try_from(idx).unwrap_or(0)
                    } else if idx % 2 == 0 {
                        base_anchor
                    } else {
                        base_anchor + [2, 4, 5, 7][rng.pick(4)]
                    }
                }
                1 => {
                    let wave = if idx % 3 == 0 { 0 } else { i16::try_from(rng.pick(4)).unwrap_or(1) };
                    base_anchor + wave + i16::try_from(phrase_half).unwrap_or(0)
                }
                2 => {
                    let lift = if idx < steps.len() / 2 { 1 } else { 3 };
                    base_anchor + lift + (i16::try_from(rng.pick(3)).unwrap_or(0) - 1)
                }
                3 => base_anchor + i16::try_from(idx).unwrap_or(0),
                4 => {
                    // Sabi stays in singing upper octave with bold 3rd/5th/octave jumps
                    let leap = [0, 2, 4, 5, 7, 4, 2][idx % 7];
                    base_anchor + leap + if kind == SectionKind::FinalChorus { 1 } else { 0 }
                }
                _ => {
                    // Solo guitar arpeggio sweep + rapid scalar climax
                    if idx % 4 == 3 {
                        base_anchor + 7
                    } else {
                        base_anchor + i16::try_from((idx * 2) % 8).unwrap_or(0)
                    }
                }
            };
            degrees.push(d);
        }

        let size = i16::try_from(intervals.len().max(1)).unwrap_or(7);
        let reg = match slot {
            0 => 64,
            1 => 62,
            2 => 67,
            3 => 64,
            4 => 72,
            _ => 67,
        };

        let mut pitches = Vec::with_capacity(steps.len());
        let mut prev = self.previous_pitches[slot];
        for (idx, &start) in steps.iter().enumerate() {
            let deg = degrees[idx];
            let pc = i16::from(intervals[usize::try_from(deg.rem_euclid(size)).unwrap_or(0)]);
            let desired = (reg + i16::from(scale.root.get()) + pc + deg.div_euclid(size) * 12)
                .clamp(55, 92);
            let strong = start.is_multiple_of(4) || idx + 1 == steps.len();
            let chosen = (55_i16..=92)
                .filter(|n| {
                    if strong && context.chord.tones.iter().any(|t| *t == n % 12) {
                        true
                    } else {
                        intervals
                            .iter()
                            .any(|iv| (i16::from(scale.root.get()) + i16::from(*iv)) % 12 == n % 12)
                    }
                })
                .min_by_key(|n| {
                    let err = (n - desired).abs() * 5;
                    let jump = (n - prev).abs();
                    if slot == 4 || slot == 5 {
                        err + (jump - 9).max(0) * 3
                    } else {
                        err + jump
                    }
                })
                .unwrap_or(60 + context.chord.root);
            pitches.push((start, chosen));
            prev = chosen;
        }
        self.previous_pitches[slot] = prev;

        if context.final_bar && context.config.cadences {
            pitches.retain(|(start, _)| *start < 8);
            pitches.push((8, 72 + context.chord.root));
        }

        let mut notes = Vec::with_capacity(pitches.len());
        for (idx, &(start, pitch)) in pitches.iter().enumerate() {
            let gap = pitches.get(idx + 1).map_or(16, |p| p.0).saturating_sub(start).max(1);
            let dur = if context.final_bar {
                gap
            } else if slot == 3 && bar_slot == 3 && idx == 0 {
                gap.min(7)
            } else if slot == 4 && idx + 1 == pitches.len() {
                gap.min(4)
            } else if slot == 5 {
                gap.clamp(1, 2)
            } else if slot == 0 {
                gap.min(2)
            } else {
                gap.saturating_sub(1).clamp(1, 3)
            };
            let vel = if slot == 4 || slot == 0 {
                104
            } else if start.is_multiple_of(4) {
                96
            } else {
                86
            };
            let mut n = note(pitch, start, dur.min(16 - start).max(1), vel)?;
            // Expressive pitch scoop/bend on solo guitar peaks, brass stabs & chorus launches
            if !context.final_bar
                && ((slot == 5 && dur >= 2 && idx % 4 == 2)
                    || (slot == 4 && idx == 0 && bar_slot % 2 == 0)
                    || (slot == 0 && idx == 0 && bar_slot == 2))
                && n.pitch.get() >= 4
            {
                n.pitch = soundbit_core::Pitch::new(n.pitch.get() - 2).map_err(|e| e.to_string())?;
                n.slide_semitones = 2;
            }
            notes.push(n);
        }
        Ok(notes)
    }
}

fn build_motif_for_contour(contour: MelodyContour, anchor: i16, rng: &mut Rng) -> Vec<i16> {
    let count = 5 + rng.pick(4);
    let mut out = Vec::with_capacity(count);
    match contour {
        MelodyContour::PedalPivot => {
            // Alternating fixed anchor pedal and rising/falling upper tones
            let tops = [anchor + 2, anchor + 4, anchor + 5, anchor + 7, anchor + 4];
            for i in 0..count {
                if i % 2 == 0 {
                    out.push(anchor);
                } else {
                    let jitter = i16::try_from(rng.pick(3)).unwrap_or(0) - 1;
                    out.push((tops[(i / 2) % tops.len()] + jitter).clamp(-2, 10));
                }
            }
        }
        MelodyContour::DescendingCascade => {
            // Starts high (7th/octave) and cascades downward in sequential motifs
            let mut cur = 7 + i16::try_from(rng.pick(3)).unwrap_or(0);
            out.push(cur);
            for i in 1..count {
                let delta = if i % 3 == 2 {
                    1
                } else {
                    -i16::try_from(1 + rng.pick(2)).unwrap_or(1)
                };
                cur = (cur + delta).clamp(-2, 10);
                out.push(cur);
            }
        }
        MelodyContour::AnacrusisPickup => {
            // Pickup notes rising into a held tonic/fifth goal note
            let goal = 2 + anchor;
            for i in 0..count {
                let d = match i % 4 {
                    0 => goal,
                    1 => goal - 2 + i16::try_from(rng.pick(2)).unwrap_or(0),
                    2 => goal - 1,
                    _ => goal + i16::try_from(1 + rng.pick(3)).unwrap_or(2),
                };
                out.push(d.clamp(-3, 9));
            }
        }
        MelodyContour::OffbeatSyncopated => {
            // Punchy octave & chord-tone jumps around an offbeat pocket
            let mut cur = anchor;
            out.push(cur);
            for _ in 1..count {
                let step = [-2, -1, 2, 3, 4][rng.pick(5)];
                cur = (cur + step).clamp(-2, 9);
                out.push(cur);
            }
        }
        MelodyContour::DualCallResponse => {
            // Question arching upward on odd degrees
            let mut cur = anchor;
            out.push(cur);
            for i in 1..count {
                let step = if i < count / 2 {
                    i16::try_from(1 + rng.pick(2)).unwrap_or(1)
                } else {
                    [-1, 0, 2][rng.pick(3)]
                };
                cur = (cur + step).clamp(-2, 9);
                out.push(cur);
            }
        }
        MelodyContour::VocalChopSyllabic => {
            // Wide fifth / octave chop syllables with neighbor flips
            let pool = [anchor, anchor + 3, anchor + 4, anchor + 7, anchor + 5];
            for _ in 0..count {
                out.push(pool[rng.pick(pool.len())].clamp(-2, 10));
            }
        }
        MelodyContour::ExoticOrnamental => {
            // Tight modal neighboring turns around 0, +1, +2, +3
            let mut cur = 0;
            out.push(cur);
            for _ in 1..count {
                let step = [-1, 1, 1, -1, 2][rng.pick(5)];
                cur = (cur + step).clamp(-1, 5);
                out.push(cur);
            }
        }
        MelodyContour::RhythmicMorseHook => {
            // Hypnotic 2-3 pitch hook with one tension twist near phrase end
            let a = anchor;
            let b = anchor + i16::try_from(2 + rng.pick(3)).unwrap_or(3);
            let twist = anchor + i16::try_from(4 + rng.pick(3)).unwrap_or(5);
            for i in 0..count {
                let deg = if i + 1 == count {
                    twist
                } else if rng.chance(62) {
                    a
                } else {
                    b
                };
                out.push(deg.clamp(-2, 9));
            }
        }
        MelodyContour::WideIntervalLeap => {
            // Bold 4th, 6th and octave leaps
            let mut cur = anchor;
            out.push(cur);
            for i in 1..count {
                let leap = if i % 2 == 1 {
                    [3, 4, 5, 7][rng.pick(4)]
                } else {
                    [-3, -4, -2, -5][rng.pick(4)]
                };
                cur = (cur + leap).clamp(-2, 11);
                out.push(cur);
            }
        }
        MelodyContour::BluesFunkRiff | MelodyContour::Auto => {
            // Blues/funk riff with tight pocket turns and octave stabs
            let mut cur = anchor;
            out.push(cur);
            for _ in 1..count {
                let step = [-2, -1, 0, 1, 2, 3][rng.pick(6)];
                cur = (cur + step).clamp(-2, 8);
                out.push(cur);
            }
        }
    }
    if out.iter().all(|n| *n == out[0]) {
        out[count / 2] += 2;
    }
    out
}

fn build_contrasting_motif(motif: &[i16], contour: MelodyContour, rng: &mut Rng) -> Vec<i16> {
    let shift = if contour == MelodyContour::DescendingCascade {
        -1
    } else if rng.chance(50) {
        2
    } else {
        -1
    };
    motif
        .iter()
        .enumerate()
        .map(|(i, &deg)| {
            if i + 1 == motif.len() {
                (motif[0] + if i % 2 == 0 { 0 } else { 1 }).clamp(-2, 9)
            } else {
                (deg + shift).clamp(-3, 10)
            }
        })
        .collect()
}

fn contour_rhythm_weight(contour: MelodyContour, in_bar: u16) -> i32 {
    match contour {
        MelodyContour::PedalPivot => {
            if matches!(in_bar, 0 | 2 | 3 | 6 | 8 | 10 | 12) {
                7
            } else if in_bar >= 14 {
                0
            } else {
                2
            }
        }
        MelodyContour::DescendingCascade => {
            if matches!(in_bar, 0 | 3 | 6 | 8 | 11 | 14) {
                7
            } else {
                2
            }
        }
        MelodyContour::AnacrusisPickup => {
            if matches!(in_bar, 0 | 6 | 12 | 14 | 15) {
                7
            } else {
                1
            }
        }
        MelodyContour::OffbeatSyncopated => {
            if matches!(in_bar, 2 | 5 | 6 | 10 | 14) {
                8
            } else if in_bar == 0 {
                0
            } else {
                2
            }
        }
        MelodyContour::DualCallResponse => {
            if in_bar <= 9 && (in_bar % 3 == 0 || in_bar % 4 == 2) {
                7
            } else {
                1
            }
        }
        MelodyContour::VocalChopSyllabic => {
            if matches!(in_bar, 0 | 2 | 3 | 8 | 10 | 14) {
                7
            } else {
                0
            }
        }
        MelodyContour::ExoticOrnamental => {
            if matches!(in_bar, 0 | 1 | 2 | 4 | 6 | 7 | 8 | 11) {
                7
            } else {
                1
            }
        }
        MelodyContour::RhythmicMorseHook => {
            if matches!(in_bar, 0 | 2 | 3 | 6 | 8 | 9 | 12) {
                8
            } else {
                1
            }
        }
        MelodyContour::WideIntervalLeap => {
            if matches!(in_bar, 0 | 6 | 10) {
                8
            } else {
                1
            }
        }
        MelodyContour::BluesFunkRiff | MelodyContour::Auto => {
            if matches!(in_bar, 0 | 3 | 6 | 9 | 10 | 14) {
                7
            } else {
                2
            }
        }
    }
}
