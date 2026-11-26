//! Materialize composition decisions as ordinary editable project objects.
use super::{ComposeConfig, Role};
use crate::PresetLibrary;
use soundbit_core::{
    Bpm, InstrumentId, Note, Pattern, Scale, Song, Swing, TimeSettings, TrackId, TrackSettings,
    Volume,
};
const NAMES: [&str; 10] = [
    "Hook",
    "Harmony",
    "Bass",
    "Emotional theme",
    "Kick",
    "Snare",
    "Hi-hat",
    "Sparkle Arp",
    "Cowbell",
    "Crash",
];
pub(super) struct Lanes {
    instruments: Vec<InstrumentId>,
    tracks: Vec<TrackId>,
}
pub(super) fn prepare(config: &ComposeConfig, scale: Scale) -> Result<(Song, Lanes), String> {
    let library = PresetLibrary::available()?;
    let mut song = Song::new(&config.title);
    song.set_scale(scale);
    song.set_timing(TimeSettings {
        bpm: Bpm::new(config.bpm).map_err(|e| e.to_string())?,
        swing: Swing::new(config.swing).map_err(|e| e.to_string())?,
        ..TimeSettings::default()
    })
    .map_err(|e| e.to_string())?;
    let p = &config.instruments;
    let studio = config.studio_layers
        || config.genre_style.is_semi_pro()
        || !config.custom_sections.is_empty();
    let mut lane_defs: Vec<(&str, &str, u8)> = vec![
        (&p.lead, NAMES[0], 72),
        (&p.harmony, NAMES[1], 50),
        (&p.bass, NAMES[2], 78),
        (&p.emotional, NAMES[3], 70),
        (&p.kick, NAMES[4], 88),
        (&p.snare, NAMES[5], 64),
        (&p.hat, NAMES[6], 40),
    ];
    if studio {
        lane_defs.extend([
            (p.arp.as_str(), NAMES[7], 50),
            (p.cowbell.as_str(), NAMES[8], 64),
            (p.crash.as_str(), NAMES[9], 62),
        ]);
    }
    let mut instruments = Vec::new();
    let mut tracks = Vec::new();
    for (id, name, level) in lane_defs {
        let preset = library
            .find(id)
            .ok_or_else(|| format!("unknown preset '{id}'; use preset_list or ctg presets"))?;
        instruments.push(
            song.add_instrument(preset.instrument.clone())
                .map_err(|e| e.to_string())?,
        );
        let mut settings = TrackSettings::new(name);
        settings.volume = Volume::new(level).map_err(|e| e.to_string())?;
        tracks.push(song.add_track(settings).map_err(|e| e.to_string())?);
    }
    Ok((
        song,
        Lanes {
            instruments,
            tracks,
        },
    ))
}
pub(super) fn place(
    song: &mut Song,
    lanes_index: &Lanes,
    role: Role,
    local_bar: usize,
    bar: usize,
    lanes: Vec<Vec<Note>>,
) -> Result<(), String> {
    for (i, notes) in lanes.into_iter().enumerate() {
        if notes.is_empty() || i >= lanes_index.tracks.len() {
            continue;
        }
        let mut pattern = Pattern::new(
            format!("{:?} {} · {}", role, local_bar + 1, NAMES[i]),
            lanes_index.instruments[i],
        );
        pattern.set_notes(notes).map_err(|e| e.to_string())?;
        // Identical repetitions share an editable pattern instead of bloating the project.
        let existing = song
            .patterns()
            .find(|(_, p)| **p == pattern)
            .map(|(id, _)| id);
        let id = match existing {
            Some(id) => id,
            None => song.add_pattern(pattern).map_err(|e| e.to_string())?,
        };
        song.place_clip(
            lanes_index.tracks[i],
            u16::try_from(bar).map_err(|e| e.to_string())?,
            id,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
