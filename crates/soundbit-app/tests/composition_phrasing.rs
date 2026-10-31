//! Observable musical structure, independent of pitch-search implementation.
#![allow(clippy::expect_used, reason = "test fixtures must generate")]
use soundbit_app::{
    compose::{ComposeConfig, Role, generate},
    model::{Note, Song},
};

fn notes(song: &Song, lane: usize, bar: usize) -> Vec<Note> {
    let (_, track) = song.tracks().nth(lane).expect("lane");
    track
        .clip_at(u16::try_from(bar).expect("bar"))
        .and_then(|id| song.pattern(id))
        .map_or_else(Vec::new, |p| p.notes().to_vec())
}

#[test]
fn returning_hooks_keep_their_opening_rhythm_and_phrases_have_breaths() {
    let mut compared = 0;
    for seed in 0..48 {
        let result = generate(&ComposeConfig {
            seed,
            ..ComposeConfig::default()
        })
        .expect("song");
        assert_eq!(result.report.composer_version, 3);
        let mut start = 0;
        for section in &result.report.design.sections {
            if section.role == Role::Verse && section.bars >= 4 {
                let opening = |bar| {
                    notes(&result.song, 0, bar)
                        .into_iter()
                        .filter(|n| n.start < 8)
                        .map(|n| n.start)
                        .collect::<Vec<_>>()
                };
                assert_eq!(opening(start), opening(start + 2));
                assert!(!opening(start).is_empty());
                for local in 0..section.bars {
                    if (local + 1).is_multiple_of(result.report.design.phrase_bars) {
                        let phrase = notes(&result.song, 0, start + local);
                        assert!(phrase.last().expect("ending").end() <= 14, "seed {seed}");
                    }
                }
                compared += 1;
            }
            start += section.bars;
        }
    }
    assert!(compared >= 48);
}

#[test]
fn rhythm_section_remembers_its_pocket_and_bridge_has_lower_density() {
    for seed in 0..48 {
        let result = generate(&ComposeConfig {
            seed,
            ..ComposeConfig::default()
        })
        .expect("song");
        let mut start = 0;
        let (mut bridge, mut climax) = ((0, 0), (0, 0));
        for section in &result.report.design.sections {
            if section.bars >= 4 {
                for lane in [2, 4] {
                    let rhythm = |bar| {
                        notes(&result.song, lane, bar)
                            .into_iter()
                            .map(|n| n.start)
                            .collect::<Vec<_>>()
                    };
                    assert_eq!(rhythm(start), rhythm(start + 2));
                }
            }
            for local in 0..section.bars {
                if section.role == Role::Bridge {
                    bridge.0 += notes(&result.song, 3, start + local).len();
                    bridge.1 += 1;
                } else if section.role == Role::Climax {
                    climax.0 += notes(&result.song, 0, start + local).len();
                    climax.1 += 1;
                }
            }
            start += section.bars;
        }
        assert!(bridge.0 * climax.1 < climax.0 * bridge.1, "seed {seed}");
    }
}

#[test]
fn long_or_accented_lead_notes_follow_actual_harmony_and_never_overlap() {
    for scale in ["major", "dorian", "harmonic_minor", "minor_pentatonic"] {
        for seed in 0..16 {
            let result = generate(&ComposeConfig {
                seed,
                scale: scale.into(),
                secondary_dominants: true,
                modal_mixture: true,
                ..ComposeConfig::default()
            })
            .expect("song");
            for bar in &result.report.bars {
                let melody = notes(&result.song, 0, bar.bar);
                for n in &melody {
                    if n.start.is_multiple_of(4) || n.length.get() >= 4 {
                        assert!(
                            bar.voicing
                                .iter()
                                .any(|p| p % 12 == i16::from(n.pitch.class().get())),
                            "{scale} seed {seed}, bar {} pitch {}",
                            bar.bar,
                            n.pitch.get()
                        );
                    }
                }
                assert!(
                    melody
                        .windows(2)
                        .all(|w| w[0].end() <= u32::from(w[1].start))
                );
            }
        }
    }
}
