//! Materialize composition decisions as ordinary editable project objects.
use super::{ComposeConfig, Role};
use crate::PresetLibrary;
use soundbit_core::{
    Bpm, InstrumentId, Note, Pattern, Scale, Song, Swing, TimeSettings, TrackId, TrackSettings,
    Volume,
};
const NAMES: [&str; 7] = [
    "Hook",
    "Harmony",
    "Bass",
    "Emotional theme",
    "Kick",
    "Snare",
    "Hi-hat",
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
    let preset_ids = [
        &p.lead,
        &p.harmony,
        &p.bass,
        &p.emotional,
        &p.kick,
        &p.snare,
        &p.hat,
    ];
    let names = [
        "Hook",
        "Harmony",
        "Bass",
        "Emotional theme",
        "Kick",
        "Snare",
        "Hi-hat",
    ];
    let levels = [72, 48, 78, 68, 85, 62, 40];
    let mut instruments = Vec::new();
    let mut tracks = Vec::new();
    for (i, id) in preset_ids.iter().enumerate() {
        let preset = library
            .find(id)
            .ok_or_else(|| format!("unknown preset '{id}'; use preset_list or ctg presets"))?;
        instruments.push(
            song.add_instrument(preset.instrument.clone())
                .map_err(|e| e.to_string())?,
        );
        let mut settings = TrackSettings::new(names[i]);
        settings.volume = Volume::new(levels[i]).map_err(|e| e.to_string())?;
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
    lanes: [Vec<Note>; 7],
) -> Result<(), String> {
    for (i, notes) in lanes.into_iter().enumerate() {
        if notes.is_empty() {
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
