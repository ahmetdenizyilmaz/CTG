//! Compare fixed-style songs, so key, tempo and instrument swaps cannot fake diversity.
#![allow(clippy::expect_used, reason = "tests fail on invalid generated music")]
use soundbit_app::compose::{ComposeConfig, Form, generate};
use std::collections::BTreeSet;

#[test]
fn same_style_key_tempo_and_instruments_still_produce_distinct_compositions() {
    let mut contours = BTreeSet::new();
    let mut rhythms = BTreeSet::new();
    let mut written_contours = BTreeSet::new();
    let mut forms = BTreeSet::new();
    let mut grooves = BTreeSet::new();
    let mut textures = BTreeSet::new();
    let mut progressions = BTreeSet::new();
    for seed in 0..128 {
        let result = generate(&ComposeConfig {
            seed,
            form: Form::Standard,
            ..ComposeConfig::default()
        })
        .expect("song");
        contours.insert(
            result
                .report
                .motif
                .windows(2)
                .map(|w| w[1] - w[0])
                .collect::<Vec<_>>(),
        );
        let (rhythm, contour) = lead_signatures(&result.song);
        rhythms.insert(rhythm);
        written_contours.insert(contour);
        forms.insert(
            result
                .report
                .design
                .sections
                .iter()
                .map(|s| format!("{:?}:{}", s.role, s.bars))
                .collect::<Vec<_>>(),
        );
        grooves.insert(result.report.design.groove);
        textures.insert(result.report.design.accompaniment);
        progressions.insert(
            result
                .report
                .bars
                .iter()
                .take(8)
                .map(|b| b.chord.clone())
                .collect::<Vec<_>>(),
        );
        let last = result.report.bars.last().expect("last");
        assert_eq!(last.bar, 31);
        assert!(result.song.profile_violations().is_empty());
    }
    assert!(
        contours.len() > 100,
        "only {} distinct contours",
        contours.len()
    );
    assert!(
        rhythms.len() > 110,
        "only {} distinct rhythms",
        rhythms.len()
    );
    assert!(forms.len() >= 4);
    assert!(
        written_contours.len() > 100,
        "only {} actual melodic contours",
        written_contours.len()
    );
    assert_eq!(grooves.len(), 4);
    assert_eq!(textures.len(), 6);
    assert!(progressions.len() > 15);
    println!(
        "Diversity across 128 seeds with identical settings: {} motifs, {} written melodic contours, {} rhythms, {} forms, {} grooves, {} textures, {} chord paths",
        contours.len(),
        written_contours.len(),
        rhythms.len(),
        forms.len(),
        grooves.len(),
        textures.len(),
        progressions.len()
    );
}

#[test]
fn every_form_has_an_emotional_section_and_a_resolved_finale() {
    for (form, length) in [
        (Form::Short, 16),
        (Form::Standard, 32),
        (Form::Extended, 48),
    ] {
        for seed in 0..32 {
            let result = generate(&ComposeConfig {
                seed,
                form,
                ..ComposeConfig::default()
            })
            .expect("song");
            assert_eq!(result.song.length_bars(), length);
            assert!(
                result
                    .report
                    .design
                    .sections
                    .iter()
                    .any(|s| matches!(s.role, soundbit_app::compose::Role::Bridge))
            );
            let (_, track) = result.song.tracks().next().expect("lead");
            let end = result
                .song
                .pattern(track.clip_at(length - 1).expect("final clip"))
                .expect("pattern")
                .notes()
                .last()
                .expect("tonic");
            assert_eq!(end.pitch.class().get(), result.song.scale().root.get());
            assert_eq!(end.end(), 16);
        }
    }
}

type Rhythm = Vec<Option<Vec<(u16, u16)>>>;
fn lead_signatures(song: &soundbit_app::model::Song) -> (Rhythm, Vec<i16>) {
    let (_, lead) = song.tracks().next().expect("lead");
    let rhythm = (0..8)
        .map(|bar| {
            lead.clip_at(bar).map(|id| {
                song.pattern(id)
                    .expect("pattern")
                    .notes()
                    .iter()
                    .map(|n| (n.start, n.length.get()))
                    .collect()
            })
        })
        .collect();
    let pitches: Vec<_> = (0..8)
        .filter_map(|bar| lead.clip_at(bar))
        .flat_map(|id| {
            song.pattern(id)
                .expect("pattern")
                .notes()
                .iter()
                .map(|n| i16::from(n.pitch.get()))
        })
        .collect();
    (rhythm, pitches.windows(2).map(|w| w[1] - w[0]).collect())
}
