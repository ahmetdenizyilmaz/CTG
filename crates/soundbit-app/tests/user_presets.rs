//! Saved sounds survive reload and cannot damage the factory library.
#![allow(clippy::expect_used, reason = "tests fail on unexpected I/O errors")]
use soundbit_app::{PresetLibrary, save_user_preset};

#[test]
fn saving_is_lossless_persistent_and_never_overwrites_another_copy() {
    let directory = tempfile::tempdir().expect("temp");
    let library = PresetLibrary::builtin().expect("factory");
    let mut sound = library
        .find("ctg_round_fm_bass")
        .expect("bass")
        .instrument
        .clone();
    sound.name = "My edited bass".into();
    let (first, first_path) = save_user_preset(directory.path(), &sound).expect("save");
    sound.pan = soundbit_app::model::Pan::new(15).expect("pan");
    let (second, _) = save_user_preset(directory.path(), &sound).expect("second copy");
    assert_ne!(first.id, second.id);
    let loaded = PresetLibrary::with_user_directory(directory.path()).expect("reload");
    assert_eq!(loaded.all().len(), library.all().len() + 2);
    assert_eq!(
        loaded.find(&first.id).expect("first").instrument,
        first.instrument
    );
    assert_eq!(loaded.find(&second.id).expect("second").instrument, sound);
    assert!(first_path.exists());
    assert_eq!(loaded.warnings(), &[] as &[String]);
    assert!(
        std::fs::read_dir(directory.path())
            .expect("files")
            .all(|e| e
                .expect("entry")
                .path()
                .extension()
                .is_some_and(|e| e == "json"))
    );
}

#[test]
fn corrupt_custom_files_warn_but_leave_valid_sounds_available() {
    let directory = tempfile::tempdir().expect("temp");
    std::fs::write(directory.path().join("broken.json"), "invalid").expect("write");
    let loaded = PresetLibrary::with_user_directory(directory.path()).expect("load");
    assert_eq!(loaded.all().len(), 86);
    assert_eq!(loaded.warnings().len(), 1);
    assert!(loaded.warnings()[0].contains("broken.json"));
}

#[test]
fn a_blank_name_does_not_create_a_preset() {
    let directory = tempfile::tempdir().expect("temp");
    let mut sound = PresetLibrary::builtin()
        .expect("factory")
        .find("ctg_deep_sub")
        .expect("sub")
        .instrument
        .clone();
    sound.name = "  ".into();
    assert!(save_user_preset(directory.path(), &sound).is_err());
    assert_eq!(
        std::fs::read_dir(directory.path()).expect("files").count(),
        0
    );
}
