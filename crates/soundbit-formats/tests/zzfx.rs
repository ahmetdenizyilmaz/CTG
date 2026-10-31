//! Import is data-only, preserves notes/patches, and rejects unsupported input.
#![allow(clippy::expect_used, reason = "test fixtures")]
use soundbit_core::Voice;
use soundbit_formats::{from_project_json, from_zzfx, to_project_json};

const EXPORT: &str = r"
const instruments = [[.5,0,261.625565,.01,.2,.15,1,1]];
const patterns = [[[0,.25,12.25,0,0,0]],[[0,.25,0,0,24,0]]];
const sequence = [0,1];
const BPM = 120;
";

#[test]
fn literal_export_preserves_pitch_pan_velocity_and_cross_bar_hold() {
    let song = from_zzfx(EXPORT).expect("valid export");
    assert_eq!(song.length_bars(), 2);
    assert_eq!(song.timing().steps_per_bar(), 4);
    let (_, track) = song.tracks().next().expect("track");
    let pattern = song
        .pattern(track.clip_at(0).expect("clip"))
        .expect("pattern");
    let note = &pattern.notes()[0];
    assert_eq!(
        (
            note.pitch.get(),
            note.start,
            note.length.get(),
            note.tail_steps
        ),
        (60, 0, 4, 2)
    );
    assert_eq!(note.velocity.get(), 95);
    let (_, instrument) = song.instruments().next().expect("instrument");
    assert_eq!(instrument.pan.get(), 40);
    let Voice::Zzfx { parameters } = instrument.voice else {
        panic!("wrong synth")
    };
    assert!((parameters.values()[2] - 261.625_565).abs() < 1e-8);
    let text = to_project_json(&song).expect("serialize");
    assert_eq!(song, from_project_json(&text).expect("round trip"));
}

#[test]
fn cuts_and_patch_changes_stop_held_notes() {
    let cut = from_zzfx(&EXPORT.replace("0,0,24,0", "0,-1,24,0")).expect("cut");
    assert_eq!(
        cut.patterns().next().expect("pattern").1.notes()[0].tail_steps,
        1
    );
    let changed = EXPORT
        .replace(
            "[[.5,0,261.625565,.01,.2,.15,1,1]]",
            "[[.5,0,261.625565,.01,.2,.15,1,1],[.4,0,220]]",
        )
        .replace("[[0,.25,0,0,24,0]]", "[[1,.25,0,0,24,0]]");
    let song = from_zzfx(&changed).expect("patch change");
    assert_eq!(
        song.patterns().next().expect("pattern").1.notes()[0].tail_steps,
        0
    );
}

#[test]
fn rejects_expressions_bad_indices_and_volume_only_rows() {
    for value in ["alert(1)", "12+3", "NaN", "Infinity", "\"12\""] {
        assert!(
            from_zzfx(&EXPORT.replace("12.25", value)).is_err(),
            "{value}"
        );
    }
    for change in [
        EXPORT.replace("12.25", ".5"),
        EXPORT.replace("[0,.25", "[.5,.25"),
        EXPORT.replace("[0,1]", "[0,9]"),
    ] {
        assert!(from_zzfx(&change).is_err());
    }
    assert!(from_zzfx("fetch('https://example.com')").is_err());
}

#[test]
fn omitted_parameters_use_upstream_defaults() {
    let song = from_zzfx(&EXPORT.replace("[.5,0,261.625565,.01,.2,.15,1,1]", "[.5,,220]"))
        .expect("omitted parameters");
    let Voice::Zzfx { parameters } = song.instruments().next().expect("instrument").1.voice else {
        panic!("wrong voice")
    };
    assert!((parameters.values()[1] - 0.05).abs() < 1e-9);
    assert!((parameters.values()[17] - 1.).abs() < 1e-9);
}

#[test]
fn existing_neon_funk_and_97_lane_glissando_are_editable() {
    for (source, lanes) in [
        (
            include_str!("../../../examples/library/zzfx-neon-pixel-funk/original.zzfx"),
            14,
        ),
        (
            include_str!("../../../examples/library/zzfx-gravity-glissando/original.zzfx"),
            97,
        ),
    ] {
        let song = from_zzfx(source).expect("original export");
        assert_eq!(song.tracks().count(), lanes);
        assert!(song.patterns().flat_map(|(_, p)| p.notes()).count() > 200);
        let saved = to_project_json(&song).expect("save");
        assert_eq!(song, from_project_json(&saved).expect("reopen"));
    }
    let logical = include_str!("../../../examples/library/zzfx-neon-pixel-funk/original-song.json");
    let song = from_zzfx(logical).expect("logical Studio project");
    assert_eq!(song.tracks().count(), 14);
    assert_eq!(song.meta().title, "Neon Pixel Funk");
}
