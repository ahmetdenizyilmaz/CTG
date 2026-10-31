//! Musical, validation and undo invariants for all composition adapters.
#![allow(clippy::expect_used, reason = "tests fail on invalid projects")]
use soundbit_app::{
    Command, Editor, Origin, PresetLibrary, Session,
    compose::{BassRhythm, ComposeConfig, Form, Role, Vibe, euclidean, generate, transform_notes},
    model::{Note, Pitch, Scale, ScaleKind, Song},
};

#[test]
fn every_style_scale_and_bass_mode_produces_valid_bounded_editable_music() {
    for vibe in [
        Vibe::Adventure,
        Vibe::Battle,
        Vibe::Dungeon,
        Vibe::TitleScreen,
        Vibe::Boss,
        Vibe::Funk,
    ] {
        for scale in [
            "major",
            "natural_minor",
            "harmonic_minor",
            "dorian",
            "mixolydian",
            "major_pentatonic",
            "minor_pentatonic",
        ] {
            for bass_rhythm in [
                BassRhythm::Mixed,
                BassRhythm::Template,
                BassRhythm::Euclidean,
            ] {
                let config = ComposeConfig {
                    vibe,
                    scale: scale.into(),
                    bass_rhythm,
                    form: Form::Short,
                    ..ComposeConfig::default()
                };
                let generated = generate(&config).expect("valid config");
                assert_eq!(generated.song.length_bars(), 16);
                assert_eq!(generated.song.profile_violations(), []);
                assert_eq!(generated.song.tracks().count(), 7);
                for (_, pattern) in generated.song.patterns() {
                    assert!(pattern.end_step() <= 16);
                    assert!(pattern.polyphony() <= 4);
                }
                for bar in &generated.report.bars {
                    let base = usize::from(config.bass_hits);
                    let expected = match bar.role {
                        Role::Bridge => (base * 7 + 5) / 10,
                        Role::Climax => (base * 12 + 5) / 10,
                        _ => base,
                    }
                    .min(16);
                    assert_eq!(bar.attacks[2], expected, "{vibe:?} {bass_rhythm:?}");
                }
            }
        }
    }
}

#[test]
fn recipes_are_repeatable_and_seeds_change_music() {
    let config = ComposeConfig::default();
    let one = generate(&config).expect("valid");
    assert_eq!(one.song, generate(&config).expect("valid").song);
    assert_ne!(
        one.song,
        generate(&ComposeConfig { seed: 43, ..config })
            .expect("valid")
            .song
    );
}

#[test]
fn final_cadence_ends_on_tonic_in_every_key() {
    for key in [
        "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
    ] {
        let result = generate(&ComposeConfig {
            key: key.into(),
            form: Form::Short,
            ..ComposeConfig::default()
        })
        .expect("valid");
        let root = i16::from(result.song.scale().root.get());
        let bar = result.report.bars.last().expect("last bar");
        assert!(bar.voicing.iter().any(|n| n % 12 == root));
        let (_, track) = result.song.tracks().next().expect("lead");
        let id = track.clip_at(15).expect("last lead");
        let last = result
            .song
            .pattern(id)
            .expect("pattern")
            .notes()
            .last()
            .expect("note");
        assert_eq!(i16::from(last.pitch.class().get()), root);
        assert_eq!(last.end(), 16);
    }
}

#[test]
fn euclidean_patterns_have_exact_counts_and_balanced_cyclic_gaps() {
    for steps in 1..=32 {
        for hits in 0..=steps {
            for rotation in 0..steps {
                let mask = euclidean(hits, steps, rotation);
                let starts: Vec<_> = mask
                    .iter()
                    .enumerate()
                    .filter(|(_, on)| **on)
                    .map(|(i, _)| i)
                    .collect();
                assert_eq!(starts.len(), hits);
                if hits > 0 {
                    let gaps: Vec<_> = (0..hits)
                        .map(|i| {
                            if i + 1 == hits {
                                steps + starts[0] - starts[i]
                            } else {
                                starts[i + 1] - starts[i]
                            }
                        })
                        .collect();
                    assert!(gaps.iter().max().expect("gap") - gaps.iter().min().expect("gap") <= 1);
                }
            }
        }
    }
}

#[test]
fn native_generation_undo_redo_restores_complete_projects() {
    let mut original = Song::new("Keep me");
    original.set_meta(soundbit_app::model::SongMeta {
        title: "Keep me".into(),
        author: "Human".into(),
        description: "Original notes".into(),
    });
    let composition = generate(&ComposeConfig::default()).expect("valid");
    let mut editor = Editor::new(original.clone());
    editor
        .execute(
            Command::ReplaceSong(Box::new(composition.song.clone())),
            Origin::Agent,
        )
        .expect("apply");
    assert_eq!(editor.undo_label(), Some("AI: Generate composition"));
    editor.undo().expect("undo");
    assert_eq!(editor.song(), &original);
    editor.redo().expect("redo");
    assert_eq!(editor.song(), &composition.song);
}

#[test]
fn all_added_packs_are_default_and_cached() {
    let first = PresetLibrary::builtin().expect("presets");
    let second = PresetLibrary::builtin().expect("presets");
    assert_eq!(first.all().len(), 86);
    assert!(std::ptr::eq(first.all().as_ptr(), second.all().as_ptr()));
    for id in [
        "aloboi_cello_solo",
        "osc_spiral_pluck",
        "sfx_zoom_in_riser",
        "pc98_opna_brass_lead",
        "trig_velvet_chord_pluck",
    ] {
        assert!(first.find(id).is_some());
    }
}

#[test]
fn invalid_inputs_are_rejected_and_zero_bass_stays_silent() {
    for json in [
        r#"{"bpm":0}"#,
        r#"{"swing":90}"#,
        r#"{"bass_hits":17}"#,
        r#"{"scale":"unknown"}"#,
        r#"{"key":"H"}"#,
        r#"{"instruments":{"lead":"missing"}}"#,
    ] {
        let config: ComposeConfig = serde_json::from_str(json).expect("recipe shape");
        assert!(generate(&config).is_err(), "{json}");
    }
    assert!(serde_json::from_str::<ComposeConfig>(r#"{"seed":-1}"#).is_err());
    assert!(serde_json::from_str::<ComposeConfig>(r#"{"typo":42}"#).is_err());
    let composition = generate(&ComposeConfig {
        bass_hits: 0,
        ..ComposeConfig::default()
    })
    .expect("valid");
    assert!(composition.report.bars.iter().all(|b| b.attacks[2] == 0));
}

#[test]
fn project_roundtrip_keeps_recipe_expression_and_all_notes() {
    let dir = tempfile::tempdir().expect("directory");
    let generated = generate(&ComposeConfig::default()).expect("valid");
    let mut session = Session::new(generated.song);
    let path = dir.path().join("generated.sbproj");
    session.save_as(&path).expect("save");
    assert_eq!(Session::open(path).expect("open").song(), session.song());
}

#[test]
fn inversion_twice_restores_diatonic_motif_and_expression() {
    let scale = Scale::new("C".parse().expect("key"), ScaleKind::Major);
    let notes: Vec<_> = [60, 64, 67, 62]
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            Note::new(
                Pitch::new(p).expect("pitch"),
                u16::try_from(i * 4).expect("small"),
                2,
            )
            .expect("note")
            .with_detune_cents(3)
            .with_slide_semitones(1)
        })
        .collect();
    let inverted = transform_notes(&notes, scale, 0, true).expect("transform");
    assert_eq!(
        transform_notes(&inverted, scale, 0, true).expect("transform"),
        notes
    );
    assert_ne!(inverted, notes);
}

#[test]
fn optional_harmony_tools_really_change_the_written_music() {
    let base = ComposeConfig {
        key: "C".into(),
        scale: "major".into(),
        form: Form::Standard,
        ..ComposeConfig::default()
    };
    let normal = generate(&base).expect("valid");
    let borrowed = generate(&ComposeConfig {
        modal_mixture: true,
        ..base.clone()
    })
    .expect("valid");
    assert!(
        borrowed
            .report
            .bars
            .iter()
            .any(|b| b.chord.contains("borrowed iv"))
    );
    assert_ne!(normal.song, borrowed.song);
    let secondary = generate(&ComposeConfig {
        secondary_dominants: true,
        ..base
    })
    .expect("valid");
    assert!(
        secondary
            .report
            .bars
            .iter()
            .any(|b| b.chord.contains("secondary dominant"))
    );
}

#[test]
fn bridge_suspensions_are_prepared_and_change_actual_notes_when_eligible() {
    let mut changed = 0;
    for seed in 0..64 {
        let config = ComposeConfig {
            seed,
            key: "C".into(),
            scale: "major".into(),
            form: Form::Short,
            ..ComposeConfig::default()
        };
        let with = generate(&config).expect("valid");
        let without = generate(&ComposeConfig {
            suspensions: false,
            ..config
        })
        .expect("valid");
        let left: Vec<_> = with
            .song
            .patterns()
            .map(|(_, p)| p.notes().to_vec())
            .collect();
        let right: Vec<_> = without
            .song
            .patterns()
            .map(|(_, p)| p.notes().to_vec())
            .collect();
        if left != right {
            changed += 1;
        }
    }
    assert!(
        changed > 0,
        "suspension control must change eligible bridge phrases"
    );
}
