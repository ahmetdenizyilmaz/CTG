//! Musical, validation and undo invariants for all composition adapters.
#![allow(clippy::expect_used, reason = "tests fail on invalid projects")]
use soundbit_app::{
    Command, Editor, Origin, PresetLibrary, Session,
    compose::{
        AccompanimentStyle, BassRhythm, ChordProgressionStyle, ComposeConfig, Form, GenreStyle,
        GrooveTemplate, MelodyContour, MotifForm, Role, SectionKind, SectionSpec, Vibe, euclidean,
        generate, transform_notes,
    },
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
    assert_eq!(first.all().len(), 253);
    assert!(std::ptr::eq(first.all().as_ptr(), second.all().as_ptr()));
    for id in [
        "acoustic_grand_piano",
        "bw_bright_pulse_lead",
        "aloboi_cello_solo",
        "osc_spiral_pluck",
        "sfx_zoom_in_riser",
        "pc98_opna_brass_lead",
        "trig_velvet_chord_pluck",
        "vocal_formant_chop",
        "devilfish_303_acid",
        "alpha_juno_hoover",
        "m1_house_piano",
        "m1_organ_bass",
        "uk_bassline_donk",
        "padsynth_lush_choir",
        "balkan_zurna_lead",
        "cyber_shamisen_pluck",
        "eurobeat_synth_brass",
        "eurobeat_octave_bass",
        "anime_rock_guitar",
        "uprising_trance_lead",
        "anime_sparkle_arp",
        "eurobeat_cowbell",
        "club_crash_cymbal",
    ] {
        assert!(first.find(id).is_some(), "missing preset {id}");
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

#[test]
fn neon_funk_recipe_has_original_zzfx_sounds_and_expressive_sections() {
    let config: ComposeConfig =
        serde_json::from_str(include_str!("../../../examples/neon-funk/recipe.json"))
            .expect("recipe");
    let composition = generate(&config).expect("Neon palette");
    assert_eq!(composition.report.design.groove, "syncopated");
    assert_eq!(composition.report.design.accompaniment, "offbeat_stabs");
    assert_eq!(composition.report.design.bass_motion, "octave");
    assert!(
        composition
            .song
            .instruments()
            .all(|(_, i)| matches!(i.voice, soundbit_app::model::Voice::Zzfx { .. }))
    );
    assert!(
        composition
            .report
            .bars
            .iter()
            .any(|b| b.role == Role::Bridge && b.attacks[0] == 0 && b.attacks[3] > 0)
    );
    assert!(
        composition
            .report
            .bars
            .iter()
            .any(|b| b.role == Role::Climax && b.attacks[0] > 0)
    );
    assert!(
        composition
            .song
            .patterns()
            .flat_map(|(_, p)| p.notes())
            .any(|n| n.slide_semitones != 0)
    );
    assert!(
        composition
            .song
            .patterns()
            .flat_map(|(_, p)| p.notes())
            .any(|n| n.glissando)
    );
    let project = soundbit_formats::to_project_json(&composition.song).expect("write");
    assert_eq!(
        composition.song,
        soundbit_formats::from_project_json(&project).expect("read")
    );
}

#[test]
fn all_ten_contours_genres_motif_forms_and_new_scales_generate_distinct_valid_music() {
    use std::collections::BTreeSet;
    let mut motifs = BTreeSet::new();
    for contour in MelodyContour::ALL {
        let result = generate(&ComposeConfig {
            melody_contour: contour,
            form: Form::Short,
            rest_density: 40,
            syncopation: 55,
            slide_probability: 50,
            ..ComposeConfig::default()
        })
        .expect("contour valid");
        assert_eq!(result.song.profile_violations(), []);
        assert_eq!(result.report.design.melody_contour, contour.as_str());
        motifs.insert(result.report.motif);
    }
    assert!(motifs.len() >= 8, "contours must vary motifs widely");

    let genres = [
        GenreStyle::EuphoricTrance,
        GenreStyle::AcidWarehouse303,
        GenreStyle::LiquidDnb,
        GenreStyle::UkGarage2Step,
        GenreStyle::FrenchTouchDisco,
        GenreStyle::MelbourneBounce,
        GenreStyle::MidtempoCyberpunk,
        GenreStyle::PianoHouse90s,
        GenreStyle::AfterlifeTechno,
        GenreStyle::BalkanFestivalBrass,
    ];
    let mut grooves = BTreeSet::new();
    for genre in genres {
        let result = generate(&ComposeConfig {
            genre_style: genre,
            form: Form::Short,
            ..ComposeConfig::default()
        })
        .expect("genre valid");
        assert_eq!(result.song.profile_violations(), []);
        grooves.insert(result.report.design.groove);
    }
    assert!(grooves.len() >= 8);

    for form in MotifForm::ALL {
        for scale in ["phrygian", "lydian", "blues", "melodic_minor"] {
            let result = generate(&ComposeConfig {
                motif_form: form,
                scale: scale.into(),
                groove_template: GrooveTemplate::AcidWarehouse,
                accompaniment_style: AccompanimentStyle::MinimalGapAnswer,
                form: Form::Short,
                ..ComposeConfig::default()
            })
            .expect("form and scale valid");
            assert_eq!(result.song.profile_violations(), []);
            let (_, track) = result.song.tracks().next().expect("lead");
            let id = track.clip_at(15).expect("final lead clip");
            let last = result
                .song
                .pattern(id)
                .expect("pattern")
                .notes()
                .last()
                .expect("last note");
            assert_eq!(last.pitch.class().get(), result.song.scale().root.get());
            assert_eq!(last.end(), 16);
        }
    }
}

#[test]
fn semi_pro_initial_d_and_anime_multi_section_songs_have_distinct_melodies_and_continuations() {
    use std::collections::BTreeSet;
    for genre in [
        GenreStyle::InitialDEurobeat,
        GenreStyle::AnimeOpeningTechno,
        GenreStyle::UprisingMelodicTechno,
        GenreStyle::HyperEurodanceRave,
    ] {
        let mut config = ComposeConfig {
            genre_style: genre,
            key: "D".into(),
            scale: "natural_minor".into(),
            bpm: 156,
            form: Form::Standard,
            key_change_finale: true,
            riser_builds: true,
            studio_layers: true,
            ..ComposeConfig::default()
        };
        config.instruments.lead = "eurobeat_synth_brass".into();
        config.instruments.harmony = "anime_grand_chords".into();
        config.instruments.bass = "eurobeat_octave_bass".into();
        config.instruments.emotional = "anime_rock_guitar".into();
        config.instruments.kick = "tr909_club_kick".into();
        config.instruments.snare = "tr909_club_snare".into();

        let generated = generate(&config).expect("semi-pro song generates");
        assert_eq!(generated.song.tracks().count(), 10);
        assert_eq!(generated.song.profile_violations(), []);
        assert_eq!(generated.song.length_bars(), 48);

        // Distinct section types must yield distinct written lead/solo bar patterns
        let mut distinct_bars = BTreeSet::new();
        for bar in 0..generated.song.length_bars() {
            for (_, trk) in generated.song.tracks().take(4) {
                if let Some(pid) = trk.clip_at(bar)
                    && let Some(pat) = generated.song.pattern(pid)
                {
                    let sig: Vec<_> = pat.notes().iter().map(|n| (n.start, n.pitch.get())).collect();
                    distinct_bars.insert(sig);
                }
            }
        }
        assert!(
            distinct_bars.len() >= 12,
            "semi-pro arrangement should produce diverse bars across sections (got {})",
            distinct_bars.len()
        );
    }

    // Custom multi-section builder with per-section theme, progression, energy & key shift
    let custom = ComposeConfig {
        genre_style: GenreStyle::InitialDEurobeat,
        bpm: 158,
        riser_builds: true,
        studio_layers: true,
        custom_sections: vec![
            SectionSpec {
                kind: SectionKind::IntroRiff,
                bars: 4,
                melody_theme: 0,
                progression: ChordProgressionStyle::EurobeatCircle,
                energy: 5,
                transpose_semitones: 0,
                seed_offset: 11,
            },
            SectionSpec {
                kind: SectionKind::VerseA,
                bars: 4,
                melody_theme: 1,
                progression: ChordProgressionStyle::UprisingAnthem,
                energy: 3,
                transpose_semitones: 0,
                seed_offset: 22,
            },
            SectionSpec {
                kind: SectionKind::VerseB,
                bars: 4,
                melody_theme: 2,
                progression: ChordProgressionStyle::KomuroDrive,
                energy: 4,
                transpose_semitones: 0,
                seed_offset: 33,
            },
            SectionSpec {
                kind: SectionKind::PreChorus,
                bars: 4,
                melody_theme: 3,
                progression: ChordProgressionStyle::DramaticMinor,
                energy: 4,
                transpose_semitones: 0,
                seed_offset: 44,
            },
            SectionSpec {
                kind: SectionKind::FinalChorus,
                bars: 4,
                melody_theme: 4,
                progression: ChordProgressionStyle::RoyalRoad,
                energy: 5,
                transpose_semitones: 2,
                seed_offset: 55,
            },
        ],
        ..ComposeConfig::default()
    };
    let custom_out = generate(&custom).expect("custom section song");
    assert_eq!(custom_out.song.length_bars(), 20);
    assert_eq!(custom_out.song.tracks().count(), 10);
}
