//! Edits requested by the front end, translated into app commands.

mod instrument;
mod note;

use serde::Deserialize;
use soundbit_app::Command;
use soundbit_app::command::{
    InstrumentCommand, MetaChanges, PatternChanges, PatternCommand, ReverbChanges, SongCommand,
    TimingChanges, TrackChanges, TrackCommand,
};
use soundbit_app::model::{
    ChipProfile, Instrument, InstrumentId, ModelError, Note, Pattern, PatternId, Pitch, PitchClass,
    Scale, ScaleKind, Song, TrackId, TrackSettings, Voice,
};
use std::num::NonZeroU16;

use instrument::InstrumentUpdate;
use note::NoteToggle;

/// One edit from the UI. Mirrored by `Action` in `src/lib/ipc/types.ts`.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Action {
    UpdateSong(SongSettings),
    AddInstrument {
        name: String,
        voice: Voice,
    },
    AddPreset {
        preset: String,
    },
    UpdateInstrument(InstrumentUpdate),
    RemoveInstrument {
        id: u32,
    },
    CreatePattern {
        name: String,
        instrument: u32,
    },
    UpdatePattern {
        id: u32,
        name: Option<String>,
        instrument: Option<u32>,
    },
    DuplicatePattern {
        id: u32,
    },
    RemovePattern {
        id: u32,
    },
    TransposePattern {
        id: u32,
        semitones: i32,
    },
    ToggleNote(NoteToggle),
    SetNotes {
        pattern: u32,
        notes: Vec<soundbit_app::model::Note>,
    },
    AddTrack {
        name: String,
    },
    UpdateTrack(TrackUpdate),
    RemoveTrack {
        id: u32,
    },
    /// Places a pattern on a bar, or clears the bar when `pattern` is `null`.
    SetClip {
        track: u32,
        bar: u16,
        pattern: Option<u32>,
    },
}

/// Song-wide settings to change; unset fields keep their value.
#[derive(Debug, Deserialize)]
pub(crate) struct SongSettings {
    title: Option<String>,
    bpm: Option<u16>,
    steps_per_beat: Option<u8>,
    beats_per_bar: Option<u8>,
    swing: Option<u8>,
    chip_profile: Option<ChipProfile>,
    scale_root: Option<u8>,
    scale_kind: Option<ScaleKind>,
    transpose_notes: Option<bool>,
    semitone_offset: Option<i32>,
    reverb_size: Option<u8>,
    reverb_damping: Option<u8>,
}

/// Track properties to change.
#[derive(Debug, Deserialize)]
pub(crate) struct TrackUpdate {
    id: u32,
    name: Option<String>,
    muted: Option<bool>,
    volume: Option<u8>,
}

pub(super) fn ranged<T, R>(value: Option<R>) -> Result<Option<T>, String>
where
    T: TryFrom<R, Error = ModelError>,
{
    value
        .map(T::try_from)
        .transpose()
        .map_err(|error| error.to_string())
}

impl Action {
    pub(crate) fn into_command(self, song: &Song) -> Result<Command, String> {
        let pattern = PatternId::new;
        Ok(match self {
            Self::UpdateSong(settings) => settings.into_command(song)?,
            Self::AddInstrument { name, voice } => {
                InstrumentCommand::Add(Instrument::new(name, voice)).into()
            }
            Self::AddPreset { preset } => instrument::add_preset(&preset)?,
            Self::UpdateInstrument(update) => update.into_command()?,
            Self::RemoveInstrument { id } => {
                InstrumentCommand::Remove(InstrumentId::new(id)).into()
            }
            Self::CreatePattern { name, instrument } => {
                PatternCommand::Create(Pattern::new(name, InstrumentId::new(instrument))).into()
            }
            Self::UpdatePattern {
                id,
                name,
                instrument,
            } => {
                let changes = PatternChanges {
                    name,
                    instrument: instrument.map(InstrumentId::new),
                };
                PatternCommand::Update(pattern(id), changes).into()
            }
            Self::DuplicatePattern { id } => PatternCommand::Duplicate {
                source: pattern(id),
                name: None,
            }
            .into(),
            Self::RemovePattern { id } => PatternCommand::Remove(pattern(id)).into(),
            Self::TransposePattern { id, semitones } => {
                PatternCommand::Transpose(pattern(id), semitones).into()
            }
            Self::SetNotes { pattern: id, notes } => {
                PatternCommand::SetNotes(pattern(id), notes).into()
            }
            Self::ToggleNote(toggle) => toggle.into_command(song)?,
            Self::AddTrack { name } => TrackCommand::Add(TrackSettings::new(name)).into(),
            Self::UpdateTrack(update) => update.into_command()?,
            Self::RemoveTrack { id } => TrackCommand::Remove(TrackId::new(id)).into(),
            Self::SetClip {
                track,
                bar,
                pattern: Some(id),
            } => TrackCommand::PlaceClips {
                track: TrackId::new(track),
                pattern: pattern(id),
                bars: vec![bar],
            }
            .into(),
            Self::SetClip {
                track,
                bar,
                pattern: None,
            } => TrackCommand::ClearClips {
                track: TrackId::new(track),
                bars: vec![bar],
            }
            .into(),
        })
    }
}

impl SongSettings {
    fn into_command(self, song: &Song) -> Result<Command, String> {
        let mut commands: Vec<Command> = Vec::new();
        if self.title.is_some() {
            let changes = MetaChanges {
                title: self.title,
                ..MetaChanges::default()
            };
            commands.push(SongCommand::UpdateMeta(changes).into());
        }
        let timing = TimingChanges {
            bpm: ranged(self.bpm)?,
            steps_per_beat: ranged(self.steps_per_beat)?,
            beats_per_bar: ranged(self.beats_per_bar)?,
            swing: ranged(self.swing)?,
        };
        if timing != TimingChanges::default() {
            commands.push(SongCommand::UpdateTiming(timing).into());
        }
        if self.scale_root.is_some() || self.scale_kind.is_some() || self.semitone_offset.is_some()
        {
            let current = song.scale();
            let mut root = ranged::<PitchClass, _>(self.scale_root)?.unwrap_or(current.root);
            if let Some(offset) = self.semitone_offset
                && self.scale_root.is_none()
            {
                let shifted = (i32::from(root.get()) + offset).rem_euclid(12);
                if let Ok(shifted_u8) = u8::try_from(shifted)
                    && let Ok(pc) = PitchClass::new(shifted_u8)
                {
                    root = pc;
                }
            }
            let scale = Scale::new(root, self.scale_kind.unwrap_or(current.kind));
            if scale != current {
                commands.push(SongCommand::SetScale(scale).into());
            }
            if self.transpose_notes.unwrap_or(true) {
                commands.extend(remap_song_patterns(
                    song,
                    current,
                    scale,
                    self.semitone_offset.unwrap_or(0),
                ));
            }
        }
        if let Some(profile) = self.chip_profile {
            commands.push(SongCommand::SetChipProfile(profile).into());
        }
        let reverb = ReverbChanges {
            size: ranged(self.reverb_size)?,
            damping: ranged(self.reverb_damping)?,
        };
        if reverb != ReverbChanges::default() {
            commands.push(SongCommand::UpdateReverb(reverb).into());
        }
        Ok(Command::Batch {
            label: "Change song settings".to_owned(),
            commands,
        })
    }
}

impl TrackUpdate {
    fn into_command(self) -> Result<Command, String> {
        let changes = TrackChanges {
            name: self.name,
            muted: self.muted,
            volume: ranged(self.volume)?,
        };
        Ok(TrackCommand::Update(TrackId::new(self.id), changes).into())
    }
}

const DRUM_KEYWORDS: [&str; 17] = [
    "kick", "snare", "hat", "crash", "tom", "clap", "cymbal", "ride", "rim", "cowbell", "shaker",
    "tamb", "bongo", "conga", "drum", "perc", "sub pulse",
];

fn has_drum_keyword(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    DRUM_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

fn is_drum_pattern(song: &Song, pattern: &Pattern) -> bool {
    if has_drum_keyword(&pattern.name) {
        return true;
    }
    if let Some(inst) = song.instrument(pattern.instrument) {
        if matches!(inst.voice, Voice::Noise { .. }) || has_drum_keyword(&inst.name) {
            return true;
        }
    }
    false
}

fn clamp_transposed(pitch: Pitch, semitones: i32) -> Pitch {
    let mut target = i32::from(pitch.get()) + semitones;
    while target < 12 {
        target += 12;
    }
    while target > 119 {
        target -= 12;
    }
    u8::try_from(target.clamp(0, 127))
        .ok()
        .and_then(|v| Pitch::new(v).ok())
        .unwrap_or(pitch)
}

fn remap_note_pitch(pitch: Pitch, old_scale: Scale, new_scale: Scale, total_shift: i32) -> Pitch {
    let shifted = clamp_transposed(pitch, total_shift);
    if new_scale.kind == ScaleKind::Chromatic {
        return shifted;
    }
    let old_iv = old_scale.kind.intervals();
    let new_iv = new_scale.kind.intervals();
    let rel = u8::try_from(
        (i32::from(pitch.get()) - i32::from(old_scale.root.get())).rem_euclid(12),
    )
    .unwrap_or(0);
    if old_scale.kind != ScaleKind::Chromatic
        && !old_iv.is_empty()
        && !new_iv.is_empty()
        && let Some(deg_idx) = old_iv.iter().position(|&iv| iv == rel)
    {
        let mapped_idx = (deg_idx * new_iv.len()) / old_iv.len();
        let target_iv = i32::from(new_iv[mapped_idx]);
        let delta_iv = target_iv - i32::from(rel);
        return clamp_transposed(pitch, total_shift + delta_iv);
    }
    new_scale.snap(shifted)
}

fn sanitize_pattern_notes(mut notes: Vec<Note>) -> Vec<Note> {
    notes.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| a.pitch.cmp(&b.pitch))
            .then_with(|| b.velocity.cmp(&a.velocity))
            .then_with(|| b.length.cmp(&a.length))
    });
    notes.dedup_by(|a, b| a.start == b.start && a.pitch == b.pitch);
    let len = notes.len();
    for i in 0..len {
        let p = notes[i].pitch;
        let end = notes[i].end();
        for j in (i + 1)..len {
            if notes[j].pitch == p {
                let next_start = u32::from(notes[j].start);
                if next_start < end {
                    let new_len = u16::try_from(next_start.saturating_sub(u32::from(notes[i].start)))
                        .unwrap_or(1)
                        .max(1);
                    if let Some(nz) = NonZeroU16::new(new_len) {
                        notes[i].length = nz;
                    }
                }
                break;
            }
        }
    }
    notes
}

fn remap_song_patterns(
    song: &Song,
    old_scale: Scale,
    new_scale: Scale,
    semitone_offset: i32,
) -> Vec<Command> {
    let total_shift = if let Some(offset) = (semitone_offset != 0).then_some(semitone_offset) {
        offset
    } else {
        let mut root_diff = i32::from(new_scale.root.get()) - i32::from(old_scale.root.get());
        if root_diff > 6 {
            root_diff -= 12;
        } else if root_diff < -6 {
            root_diff += 12;
        }
        root_diff
    };
    if total_shift == 0 && old_scale.kind == new_scale.kind {
        return Vec::new();
    }
    let mut cmds = Vec::new();
    for (id, pattern) in song.patterns() {
        if pattern.notes().is_empty() || is_drum_pattern(song, pattern) {
            continue;
        }
        let remapped: Vec<Note> = pattern
            .notes()
            .iter()
            .map(|note| Note {
                pitch: remap_note_pitch(note.pitch, old_scale, new_scale, total_shift),
                ..*note
            })
            .collect();
        let cleaned = sanitize_pattern_notes(remapped);
        if cleaned != pattern.notes() {
            cmds.push(PatternCommand::SetNotes(id, cleaned).into());
        }
    }
    cmds
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "tests panic on unexpected errors by design"
    )]

    use super::*;

    #[test]
    fn actions_deserialize_from_front_end_json() {
        let json = r#"{"kind": "update_song", "bpm": 150, "chip_profile": "nes"}"#;
        let action: Action = serde_json::from_str(json).expect("valid action");
        assert!(matches!(
            action,
            Action::UpdateSong(SongSettings { bpm: Some(150), .. })
        ));
        let json = r#"{"kind": "add_instrument", "name": "Lead", "voice": {"type": "pulse", "duty": "quarter"}}"#;
        assert!(serde_json::from_str::<Action>(json).is_ok());
    }

    #[test]
    fn instrument_groups_and_presets_deserialize() {
        let json = r#"{"kind": "update_instrument", "id": 1,
            "filter": {"mode": "low_pass", "cutoff": 900, "resonance": 20, "envelope_depth": 12},
            "unison": {"voices": 3, "detune": 14}, "reverb_send": 40}"#;
        let action: Action = serde_json::from_str(json).expect("valid action");
        assert!(action.into_command(&Song::new("t")).is_ok());
        let json = r#"{"kind": "update_instrument", "id": 1, "unison": {"voices": 9}}"#;
        assert!(
            serde_json::from_str::<Action>(json).is_err(),
            "out of range"
        );
        let json = r#"{"kind": "add_preset", "preset": "kick"}"#;
        let action: Action = serde_json::from_str(json).expect("valid action");
        assert!(action.into_command(&Song::new("t")).is_ok());
    }

    #[test]
    fn changing_key_and_scale_transposes_pitched_patterns_and_skips_drums() {
        let mut song = Song::new("test");
        song.set_scale(Scale::new(PitchClass::default(), ScaleKind::Major));
        let lead_id = song
            .add_instrument(Instrument::new("Lead", Voice::Triangle))
            .expect("lead");
        let kick_id = song
            .add_instrument(Instrument::new("Kick Drum", Voice::Triangle))
            .expect("kick");
        let mut lead_pat = Pattern::new("Melody", lead_id);
        lead_pat
            .insert_note(Note::new("C4".parse().expect("c4"), 0, 2).expect("note"))
            .expect("insert c4");
        lead_pat
            .insert_note(Note::new("E4".parse().expect("e4"), 2, 2).expect("note"))
            .expect("insert e4");
        let lead_pat_id = song.add_pattern(lead_pat).expect("add lead pat");

        let mut kick_pat = Pattern::new("Kick 1", kick_id);
        kick_pat
            .insert_note(Note::new("C2".parse().expect("c2"), 0, 1).expect("note"))
            .expect("insert c2");
        let kick_pat_id = song.add_pattern(kick_pat).expect("add kick pat");

        // Change from C Major (0, Major) to D Natural Minor (2, NaturalMinor)
        let json = r#"{"kind": "update_song", "scale_root": 2, "scale_kind": "natural_minor"}"#;
        let action: Action = serde_json::from_str(json).expect("valid action");
        let cmd = action.into_command(&song).expect("into command");
        let mut session = soundbit_app::Session::new(song);
        session
            .editor_mut()
            .execute(cmd, soundbit_app::Origin::User)
            .expect("execute");

        let updated_lead = session
            .song()
            .pattern(lead_pat_id)
            .expect("lead pattern")
            .notes();
        // C4 (root) -> D4; E4 (major 3rd, +4) -> F4 (minor 3rd of D, +3 from D4 = 65)
        assert_eq!(updated_lead[0].pitch.to_string(), "D4");
        assert_eq!(updated_lead[1].pitch.to_string(), "F4");

        // Kick pattern should remain untouched at C2
        let updated_kick = session
            .song()
            .pattern(kick_pat_id)
            .expect("kick pattern")
            .notes();
        assert_eq!(updated_kick[0].pitch.to_string(), "C2");
    }
}
