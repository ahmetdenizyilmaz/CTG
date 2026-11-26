//! Constrained discovery: vary recipes, while retaining the musical guardrails.
use super::{BassRhythm, ComposeConfig, Form, Palette, Vibe, rhythm::Rng};

fn choose<'a>(rng: &mut Rng, choices: &'a [&str]) -> &'a str {
    choices[rng.pick(choices.len())]
}

/// Choose a repeatable, role-appropriate palette. Sweeps, drops and untuned SFX
/// are excluded from pitched lanes; slower pads are reserved for the bridge.
pub fn random_palette(seed: u32, vibe: Vibe) -> Palette {
    let mut rng = Rng::new(seed ^ 0xa17e_381d);
    let leads: &[&str] = match vibe {
        Vibe::Dungeon | Vibe::TitleScreen => &[
            "acoustic_grand_piano",
            "studio_felt_piano",
            "acoustic_solo_violin",
            "acoustic_concert_flute",
            "aloboi_warm_lead",
            "flute",
            "aloboi_nostalgic_pluck",
            "fm_electric_piano",
        ],
        Vibe::Funk => &[
            "acoustic_upright_piano",
            "vintage_rhodes_mark1",
            "warm_wurlitzer_200a",
            "m1_house_piano",
            "vocal_formant_chop",
            "daft_talkbox_lead",
            "cz_phase_lead",
            "ctg_singing_pulse",
            "zzfx_neon_lead",
            "ctg_soft_ep",
            "aloboi_warm_lead",
            "square_pluck",
            "pc98_opna_brass_lead",
            "fm_electric_piano",
        ],
        _ => &[
            "acoustic_grand_piano",
            "acoustic_solo_violin",
            "acoustic_nylon_guitar",
            "jp8000_festival_supersaw",
            "alpha_juno_hoover",
            "acid_303_lead",
            "insomnia_rave_pizz",
            "balkan_zurna_lead",
            "celtic_whistle_lead",
            "pulse_lead",
            "aloboi_warm_lead",
            "pc98_ssg_pulse_hook",
            "pc98_opna_brass_lead",
        ],
    };
    let basses: &[&str] = match vibe {
        Vibe::Funk => &[
            "acoustic_upright_bass",
            "m1_organ_bass",
            "uk_bassline_donk",
            "garage_sub_sine",
            "pc98_opna_slap_bass",
            "zzfx_neon_bass",
            "fm_bass",
            "triangle_bass",
            "ctg_rubber_bass",
            "ctg_picked_bass",
            "ctg_metal_slap_bass",
        ],
        _ => &[
            "acoustic_upright_bass",
            "devilfish_303_acid",
            "cyber_reese_neuro_bass",
            "melbourne_offbeat_bass",
            "triangle_bass",
            "fm_bass",
            "trig_minimal_sub_pulse",
            "ctg_finger_bass",
            "ctg_round_fm_bass",
            "ctg_warm_saw_bass",
        ],
    };
    Palette {
        lead: choose(&mut rng, leads).into(),
        harmony: choose(
            &mut rng,
            &[
                "acoustic_grand_piano",
                "studio_felt_piano",
                "m1_house_piano",
                "vintage_rhodes_mark1",
                "french_disco_clav",
                "wt_modern_pluck",
                "afterlife_techno_pluck",
                "cyber_shamisen_pluck",
                "arabic_oud_pluck",
                "acoustic_concert_harp",
                "acoustic_nylon_guitar",
                "acoustic_string_quartet",
                "trig_velvet_chord_pluck",
                "ctg_soft_ep",
                "zzfx_neon_chords",
                "ctg_wood_pluck",
                "ctg_bell_arp",
                "fm_electric_piano",
                "aloboi_dream_arp",
                "pc98_opna_glass_arp",
                "pizzicato",
                "square_pluck",
            ],
        )
        .into(),
        bass: choose(&mut rng, basses).into(),
        emotional: choose(
            &mut rng,
            &[
                "acoustic_solo_cello",
                "acoustic_solo_violin",
                "padsynth_lush_choir",
                "padsynth_ethereal_strings",
                "studio_felt_piano",
                "acoustic_concert_flute",
                "celtic_whistle_lead",
                "arabic_oud_pluck",
                "acoustic_warm_clarinet",
                "acoustic_vibraphone",
                "aloboi_cello_solo",
                "ctg_velvet_reed",
                "zzfx_midnight_keys",
                "flute",
                "fm_strings",
                "aloboi_warm_lead",
                "strings_ensemble",
            ],
        )
        .into(),
        kick: choose(&mut rng, &["kick", "tr909_club_kick"]).into(),
        snare: choose(&mut rng, &["snare", "clap", "tr909_club_snare", "tr909_club_clap", "acoustic_brush_snare"]).into(),
        hat: "hihat_closed".into(),
        ..Palette::default()
    }
}

/// Create a varied recipe with style-compatible tempo, scale, density and sounds.
/// Hook development, smooth voice leading and cadences are always enabled.
/// This is musical constraint sampling, not a subjective quality guarantee.
pub fn random_recipe(seed: u32) -> ComposeConfig {
    let mut rng = Rng::new(seed);
    let vibe = [
        Vibe::Funk,
        Vibe::Adventure,
        Vibe::TitleScreen,
        Vibe::Dungeon,
        Vibe::Battle,
        Vibe::Boss,
    ][rng.pick(6)];
    let (scales, low, range, hits): (&[&str], u16, usize, u8) = match vibe {
        Vibe::Funk => (&["dorian", "mixolydian", "minor_pentatonic"], 108, 25, 6),
        Vibe::Adventure => (&["major", "dorian", "major_pentatonic"], 112, 29, 5),
        Vibe::TitleScreen => (&["major", "mixolydian", "natural_minor"], 88, 25, 4),
        Vibe::Dungeon => (&["natural_minor", "dorian", "minor_pentatonic"], 76, 25, 4),
        Vibe::Battle => (&["natural_minor", "harmonic_minor", "dorian"], 128, 25, 6),
        Vibe::Boss => (&["harmonic_minor", "natural_minor"], 132, 29, 6),
    };
    let scale = choose(&mut rng, scales).to_owned();
    let key = choose(
        &mut rng,
        &[
            "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
        ],
    )
    .into();
    let bpm = low + u16::try_from(rng.pick(range)).unwrap_or(0);
    let swing = if vibe == Vibe::Funk {
        8 + u8::try_from(rng.pick(17)).unwrap_or(0)
    } else if matches!(vibe, Vibe::Battle | Vibe::Boss) {
        0
    } else {
        u8::try_from(rng.pick(4) * 4).unwrap_or(0)
    };
    let modal_mixture = matches!(scale.as_str(), "major" | "major_pentatonic") && rng.chance(40);
    let secondary_dominants = !modal_mixture && rng.chance(20);
    ComposeConfig {
        title: title(&mut rng),
        seed,
        vibe,
        key,
        scale,
        bpm,
        swing,
        form: if rng.chance(30) {
            Form::Short
        } else {
            Form::Standard
        },
        bass_rhythm: [
            BassRhythm::Mixed,
            BassRhythm::Template,
            BassRhythm::Euclidean,
        ][rng.pick(3)],
        bass_hits: hits + u8::try_from(rng.pick(3)).unwrap_or(0),
        chord_colors: !matches!(vibe, Vibe::Battle | Vibe::Boss) || rng.chance(25),
        modal_mixture,
        secondary_dominants,
        instruments: random_palette(seed, vibe),
        ..ComposeConfig::default()
    }
}

/// Randomize a whole recipe, or only the instruments of a supplied recipe.
/// Instrument-only discovery preserves the musical seed, notes and all settings
/// when the resulting recipe is generated again.
pub fn randomize(seed: u32, instruments_for: Option<&ComposeConfig>) -> ComposeConfig {
    instruments_for.map_or_else(
        || random_recipe(seed),
        |original| ComposeConfig {
            instruments: random_palette(seed, original.vibe),
            ..original.clone()
        },
    )
}

fn title(rng: &mut Rng) -> String {
    format!(
        "{} {}",
        choose(
            rng,
            &[
                "Velvet", "Paper", "Lunar", "Amber", "Neon", "Silver", "Midnight", "Electric"
            ]
        ),
        choose(
            rng,
            &[
                "Satellites",
                "Daydream",
                "Orbit",
                "Footsteps",
                "Garden",
                "Signals",
                "Carousel",
                "Tides"
            ]
        )
    )
}
