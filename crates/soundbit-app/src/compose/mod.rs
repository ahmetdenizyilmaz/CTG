//! CTG's deterministic native-instrument composition engine.
//! Musical ideas are shared with `ZzFX` Studio; synthesis and projects remain native SoundBit.
mod accompaniment;
mod build;
mod config;
mod design;
mod harmony;
mod melody;
mod parts;
mod percussion;
mod random;
mod rhythm;

pub use config::{
    AccompanimentStyle, BassRhythm, ChordProgressionStyle, ComposeConfig, Form, GenreStyle,
    GrooveTemplate, MelodyContour, MotifForm, Palette, SectionKind, SectionSpec, Vibe,
};
pub use design::SongDesign;
pub use harmony::Role;
pub use random::{random_palette, random_recipe, randomize};
pub use rhythm::euclidean;

use harmony::{bar_chord, progression, semi_pro_bar_chord, semi_pro_progression, voicing};
use melody::{Melody, SemiProMelodyBank};
use parts::BarContext;
use rhythm::Rng;
use serde::Serialize;
use soundbit_core::{Note, Song};

/// Inspectable explanation of one generated bar.
#[derive(Debug, Clone, Serialize)]
pub struct BarReport {
    /// Zero-based arrangement bar.
    pub bar: usize,
    /// Role in the form.
    pub role: Role,
    /// Chord name and any functional substitution.
    pub chord: String,
    /// Harmony voicing as MIDI pitches.
    pub voicing: Vec<i16>,
    /// Attack counts in lead, harmony, bass, emotional lead, kick, snare and hat order.
    pub attacks: Vec<usize>,
}

/// Recipe and analysis returned by every composition interface.
#[derive(Debug, Clone, Serialize)]
pub struct CompositionReport {
    /// Composition algorithm revision.
    pub composer_version: u8,
    /// Complete repeatable recipe, including instruments.
    pub config: ComposeConfig,
    /// Hook contour in scale degrees (zero based).
    pub motif: Vec<i16>,
    /// Song-level groove, texture and section plan.
    pub design: SongDesign,
    /// Musical decisions per bar.
    pub bars: Vec<BarReport>,
    /// Musical duration, excluding release tails.
    pub duration_seconds: f64,
}

/// Generated, editable song and its explanation.
#[derive(Debug, Clone)]
pub struct Composition {
    /// Native project; ordinary editor commands can change every note.
    pub song: Song,
    /// Deterministic provenance and musical analysis.
    pub report: CompositionReport,
}

/// Generate a native project without changing any open editor.
///
/// # Errors
/// Rejects invalid recipes and unknown presets before returning a project.
pub fn generate(config: &ComposeConfig) -> Result<Composition, String> {
    let scale = config.validate()?;
    let (mut song, lanes_index) = build::prepare(config, scale)?;
    let mut rng = Rng::new(config.seed);
    let design = SongDesign::new(config);
    let mut melody = Melody::new(&mut rng, &design, config);
    let mut semi_pro_bank = SemiProMelodyBank::new(config.seed);
    let semi_pro = config.genre_style.is_semi_pro() || !config.custom_sections.is_empty();
    let studio_layers = config.studio_layers || semi_pro;
    let motif = melody.motif();
    let planned = design.bars();
    let progressions: Vec<_> = (0..5).map(|_| progression(config.vibe, &mut rng)).collect();
    let mut previous_voicing = Vec::new();
    let mut bars = Vec::new();
    for (bar, plan) in planned.iter().enumerate() {
        let role = plan.role;
        let local_bar = plan.local_bar;
        let final_bar = bar + 1 == planned.len();
        let chord = if semi_pro {
            let deg8 = semi_pro_progression(plan.progression, plan.kind, scale);
            semi_pro_bar_chord(config, scale, deg8, plan.kind, local_bar, plan.section_bars, final_bar)
        } else {
            let span = if matches!(role, Role::Verse | Role::Bridge) {
                design.bars_per_chord
            } else {
                1
            };
            let source = progressions[role_index(role)];
            let degrees = std::array::from_fn(|i| source[((local_bar / 4 * 4 + i) / span) % 4]);
            bar_chord(
                config,
                scale,
                degrees,
                role,
                local_bar % 4,
                plan.final_phrase,
                plan.section_bars,
            )
        };
        let voiced = voicing(&chord, &previous_voicing, config.voice_leading);
        let context = BarContext {
            config,
            design: &design,
            chord: &chord,
            scale,
            role,
            bar: local_bar,
            final_bar,
        };
        let mut lead = if semi_pro {
            semi_pro_bank.bar(&context, plan.melody_theme, plan.seed_offset, plan.kind)?
        } else {
            let mut l = melody.bar(&context, &mut rng)?;
            express_lead(&mut l, &context, &mut rng)?;
            l
        };
        let mut harmony = accompaniment::harmony(&context, &voiced, &lead)?;
        let (drums, kicks) = percussion::drums_with_section(
            &design,
            role,
            plan.kind,
            local_bar,
            plan.section_bars,
            plan.section_end,
            config.riser_builds || semi_pro,
            &mut rng,
        )?;
        let mut bass_notes = parts::bass(&context, &kicks, &mut rng)?;
        let mut arp_notes = if studio_layers {
            accompaniment::sparkle_arp(&voiced, plan.kind, plan.energy)?
        } else {
            Vec::new()
        };
        // Apply per-section modulation / key-change finale transposition to pitched lanes
        let trans = plan.transpose_semitones;
        if trans != 0 {
            for list in [&mut lead, &mut harmony, &mut bass_notes, &mut arp_notes] {
                for n in list.iter_mut() {
                    let next = (i16::from(n.pitch.get()) + i16::from(trans)).clamp(24, 108);
                    if let Ok(midi) = u8::try_from(next)
                        && let Ok(p) = soundbit_core::Pitch::new(midi)
                    {
                        n.pitch = p;
                    }
                }
            }
        }
        let [kick, snare, hat] = drums;
        let is_solo = semi_pro && plan.kind == SectionKind::Solo && !final_bar;
        let is_dual_answer = !semi_pro
            && design.melody_contour == "dual_call_response"
            && role != Role::Bridge
            && !final_bar
            && local_bar % 2 == 1;
        let (lead, emotional) = if role == Role::Bridge || is_dual_answer || is_solo {
            (Vec::new(), lead)
        } else {
            (lead, Vec::new())
        };
        let mut lanes = vec![lead, harmony, bass_notes, emotional, kick, snare, hat];
        if studio_layers {
            let [cowbell, crash] =
                percussion::studio_percussion(plan.kind, local_bar, plan.energy)?;
            lanes.extend([arp_notes, cowbell, crash]);
        }
        bars.push(BarReport {
            bar,
            role,
            chord: chord.label.clone(),
            voicing: voiced.clone(),
            attacks: lanes.iter().map(Vec::len).collect(),
        });
        build::place(&mut song, &lanes_index, role, local_bar, bar, lanes)?;
        previous_voicing = voiced;
    }
    let duration_seconds = song.length_seconds();
    let recipe = serde_json::to_string(config).map_err(|e| e.to_string())?;
    let mut meta = song.meta().clone();
    meta.description = format!("CTG Studio / ChipTuneGen v2 recipe: {recipe}");
    song.set_meta(meta);
    Ok(Composition {
        song,
        report: CompositionReport {
            composer_version: 2,
            config: config.clone(),
            motif,
            design,
            bars,
            duration_seconds,
        },
    })
}

const fn role_index(role: Role) -> usize {
    match role {
        Role::Verse => 0,
        Role::Contrast => 1,
        Role::Bridge => 2,
        Role::Breakdown => 3,
        Role::Climax => 4,
    }
}

/// Transform a pattern's pitches using its song scale while keeping native note expression.
///
/// # Errors
/// Rejects a result outside the MIDI range. Transposition is in scale degrees.
pub fn transform_notes(
    notes: &[Note],
    scale: soundbit_core::Scale,
    degrees: i16,
    invert: bool,
) -> Result<Vec<Note>, String> {
    let allowed: Vec<_> =
        (0..=127_u8)
            .filter(|n| {
                scale.kind.intervals().iter().any(|d| {
                    (u16::from(scale.root.get()) + u16::from(*d)) % 12 == u16::from(*n) % 12
                })
            })
            .collect();
    let index = |pitch: u8| {
        allowed
            .iter()
            .enumerate()
            .min_by_key(|(_, n)| i16::from(**n).abs_diff(i16::from(pitch)))
            .map_or(0, |(i, _)| i16::try_from(i).unwrap_or(0))
    };
    let pivot = notes.first().map_or(0, |n| index(n.pitch.get()));
    notes
        .iter()
        .map(|n| {
            let i = index(n.pitch.get());
            let target = if invert { 2 * pivot - i } else { i } + degrees;
            let pitch = usize::try_from(target)
                .ok()
                .and_then(|i| allowed.get(i))
                .ok_or("transformation exceeds MIDI range")?;
            Ok(Note {
                pitch: soundbit_core::Pitch::new(*pitch).map_err(|e| e.to_string())?,
                ..*n
            })
        })
        .collect()
}

fn express_lead(
    lead: &mut [Note],
    context: &BarContext<'_>,
    rng: &mut Rng,
) -> Result<(), String> {
    if context.final_bar || context.role == Role::Bridge {
        return Ok(());
    }
    let style_slides = matches!(
        context.design.melody_contour.as_str(),
        "vocal_chop_syllabic" | "exotic_ornamental"
    ) || matches!(
        context.config.genre_style,
        GenreStyle::AcidWarehouse303 | GenreStyle::UkGarage2Step | GenreStyle::BalkanFestivalBrass
    );
    if !context.config.expressive_slides && !style_slides {
        return Ok(());
    }
    let prob = usize::from(if context.config.expressive_slides {
        context.config.slide_probability.max(45)
    } else {
        context.config.slide_probability
    });
    if ((context.bar % 4 == 3 || (context.role == Role::Climax && context.bar % 4 == 1))
        || rng.chance(prob))
        && let Some(note) = lead.iter_mut().find(|n| n.length.get() >= 2)
    {
        let interval = if context.role == Role::Climax && context.config.expressive_slides {
            12
        } else {
            2
        };
        if note.pitch.get() >= interval {
            note.pitch = soundbit_core::Pitch::new(note.pitch.get() - interval)
                .map_err(|e| e.to_string())?;
            note.slide_semitones = i8::try_from(interval).unwrap_or(2);
            note.glissando = context.role == Role::Climax && context.bar % 2 == 1;
        }
    }
    Ok(())
}
