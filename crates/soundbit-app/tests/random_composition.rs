//! Discovery must be diverse, reproducible and safe to regenerate with new sounds.
#![allow(
    clippy::expect_used,
    reason = "tests panic on invalid generated projects"
)]
use soundbit_app::compose::{generate, random_recipe, randomize};
use std::collections::BTreeSet;

#[test]
fn discovery_is_diverse_and_preserves_musical_guardrails() {
    let mut keys = BTreeSet::new();
    let mut styles = BTreeSet::new();
    let mut palettes = BTreeSet::new();
    for seed in (0..256).chain([u32::MAX]) {
        let recipe = random_recipe(seed);
        assert_eq!(recipe, random_recipe(seed));
        assert!(recipe.motif_development && recipe.voice_leading && recipe.cadences);
        assert!((76..=160).contains(&recipe.bpm));
        assert!((4..=8).contains(&recipe.bass_hits));
        assert!(recipe.swing <= 24);
        assert!(!(recipe.modal_mixture && recipe.secondary_dominants));
        let song = generate(&recipe).expect("random recipe generates").song;
        assert!(song.profile_violations().is_empty());
        assert_eq!(song.tracks().count(), 7);
        assert!([16, 32].contains(&song.length_bars()));
        for (_, pattern) in song.patterns() {
            assert!(pattern.end_step() <= 16);
            assert!(pattern.polyphony() <= 4);
        }
        keys.insert(recipe.key.clone());
        styles.insert(format!("{:?}", recipe.vibe));
        palettes.insert(format!("{:?}", recipe.instruments));
    }
    assert_eq!(keys.len(), 12);
    assert_eq!(styles.len(), 6);
    assert!(palettes.len() > 100);
}

#[test]
fn instrument_discovery_preserves_notes_timing_and_recipe_settings() {
    let original = random_recipe(42);
    let original_song = generate(&original).expect("original").song;
    for seed in 0..32 {
        let changed = randomize(seed, Some(&original));
        assert_eq!(changed, randomize(seed, Some(&original)));
        let mut without_palette = changed.clone();
        without_palette.instruments = original.instruments.clone();
        assert_eq!(without_palette, original);
        let changed_song = generate(&changed).expect("new sounds").song;
        let notes = |song: &soundbit_app::model::Song| {
            song.patterns()
                .map(|(_, pattern)| pattern.notes().to_vec())
                .collect::<Vec<_>>()
        };
        assert_eq!(notes(&changed_song), notes(&original_song));
        assert_eq!(changed_song.timing(), original_song.timing());
    }
}
