//! Safe `ZzFX` Studio JSON / metadata import. `JavaScript` is never executed.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "format limits validate indices, MIDI pitches and normalized levels before conversion"
)]
use crate::FormatError;
use serde_json::Value;
use soundbit_core::{
    BeatsPerBar, Bpm, Instrument, Note, Pan, Pattern, Pitch, Song, SongMeta, TimeSettings,
    TrackSettings, Velocity, Voice, Volume, ZzfxEffect, ZzfxParameters,
};
use std::collections::HashMap;

fn fail(message: impl Into<String>) -> FormatError {
    FormatError::Zzfx(message.into())
}
fn model(error: impl std::fmt::Display) -> FormatError {
    fail(error.to_string())
}
fn array<'a>(value: &'a Value, name: &str) -> Result<&'a Vec<Value>, FormatError> {
    value
        .as_array()
        .ok_or_else(|| fail(format!("expected {name} array")))
}

/// Import a `ZzFX` Studio song JSON, metadata-bearing `.zzfx`/`.js`, or `ZzFXM` JSON tuple.
///
/// # Errors
/// Returns an error on malformed data, unsupported effects, or unrepresentable song limits.
#[allow(
    clippy::too_many_lines,
    reason = "ordered validation and conversion of one external song document"
)]
pub fn from_zzfx(text: &str) -> Result<Song, FormatError> {
    if text.len() > 16 * 1024 * 1024 {
        return Err(fail("project exceeds 16 MiB"));
    }
    let metadata = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("// @zzfx-studio "));
    let mut data: Value =
        if text.contains("const instruments =") && text.contains("const patterns =") {
            let mut d = serde_json::json!({
                "instruments":crate::zzfx_literal::assignment(text,"instruments").map_err(fail)?,
                "patterns":crate::zzfx_literal::assignment(text,"patterns").map_err(fail)?,
                "sequence":crate::zzfx_literal::assignment(text,"sequence").map_err(fail)?,
                "bpm":crate::zzfx_literal::assignment(text,"BPM").map_err(fail)?,
                "expanded":true
            });
            if let Some(meta) = metadata {
                let m: Value = serde_json::from_str(meta)?;
                d["config"] = m["config"].clone();
                d["patternOrder"] = m["patternOrder"].clone();
                d["patternRoles"] = m["patternRoles"].clone();
            }
            d
        } else {
            serde_json::from_str(metadata.unwrap_or(text)).map_err(|_| {
            fail("Use ZzFX Studio JSON or exported literal arrays; JavaScript is never executed.")
        })?
        };
    if let Some(tuple) = data.as_array() {
        if tuple.len() < 3 {
            return Err(fail(
                "ZzFXM tuple needs instruments, patterns, sequence and optional BPM",
            ));
        }
        data = serde_json::json!({"instruments":tuple[0],"patterns":tuple[1],"sequence":tuple[2],"bpm":tuple.get(3).unwrap_or(&Value::from(125))});
    }
    if data.get("config").is_some() && data["expanded"] != true {
        data = crate::zzfx_expand::expand(&data).map_err(fail)?;
    }
    let instruments = array(&data["instruments"], "instruments")?;
    if instruments.is_empty() || instruments.len() > 128 {
        return Err(fail("expected 1-128 instruments"));
    }
    let order: Vec<String> = data["patternOrder"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| v.as_str().unwrap_or("").to_owned())
                .collect()
        })
        .unwrap_or_default();
    let patterns: Vec<&Value> = if data["patterns"].is_object() {
        if order.is_empty() {
            return Err(fail("patternOrder is required for named patterns"));
        }
        order.iter().map(|key| &data["patterns"][key]).collect()
    } else {
        array(&data["patterns"], "patterns")?.iter().collect()
    };
    if patterns.is_empty() || patterns.len() > 512 {
        return Err(fail("expected 1–512 patterns"));
    }
    let logical = data.get("config").is_some() && data["expanded"] != true;
    let mut rows = if logical { 32 } else { 0 };
    let mut channels = 0;
    for pat in &patterns {
        let ch = array(pat, "pattern channels")?;
        channels = channels.max(ch.len());
        for channel in ch {
            rows = rows.max(array(channel, "channel")?.len().saturating_sub(2));
        }
    }
    if channels == 0 || channels > 128 || rows == 0 || rows > 64 || rows % 4 != 0 {
        return Err(fail(
            "import supports up to 128 channels and uniform 4–64-row patterns in multiples of four",
        ));
    }
    if !logical
        && patterns.iter().any(|p| {
            p.as_array().is_some_and(|ch| {
                ch.iter()
                    .map(|c| c.as_array().map_or(0, Vec::len).saturating_sub(2))
                    .max()
                    .unwrap_or(0)
                    != rows
            })
        })
    {
        return Err(fail(
            "variable pattern row counts are not supported; export uniform pattern lengths",
        ));
    }
    let tempo = data["config"]["bpm"]
        .as_f64()
        .or(data["bpm"].as_f64())
        .unwrap_or(125.)
        .round() as u64;
    let tempo = u16::try_from(tempo).map_err(model)?;
    let title = data["config"]["name"]
        .as_str()
        .or(data["name"].as_str())
        .unwrap_or("Imported ZzFX song");
    let mut song = Song::new(title);
    song.set_meta(SongMeta {title:title.into(),author:data["author"].as_str().unwrap_or("").into(),description:"Imported editable ZzFX Studio score. Original mastering and extra audio mixed outside the project are not part of the score.".into()});
    song.set_timing(TimeSettings {
        bpm: Bpm::new(tempo).map_err(model)?,
        beats_per_bar: BeatsPerBar::new((rows / 4) as u8).map_err(model)?,
        ..TimeSettings::default()
    })
    .map_err(model)?;
    if let (Some(key), Some(kind)) = (
        data["config"]["key"].as_str(),
        data["config"]["scale"].as_str(),
    ) {
        let key = key.replace('♯', "#").replace('♭', "b");
        if let Ok(pitch) = format!("{key}4").parse::<Pitch>()
            && let Ok(scale) =
                serde_json::from_value(serde_json::json!({"root":pitch.get()%12,"kind":kind}))
        {
            song.set_scale(scale);
        }
    }
    let sequence = array(&data["sequence"], "sequence")?;
    if sequence.is_empty() || sequence.len() > 512 {
        return Err(fail("sequence must contain 1-512 patterns"));
    }
    let sequence: Vec<usize> = sequence
        .iter()
        .map(|v| {
            v.as_f64()
                .filter(|v| v.fract() == 0. && *v >= 0. && *v < patterns.len() as f64)
                .map(|v| v as usize)
                .ok_or_else(|| fail("invalid sequence index"))
        })
        .collect::<Result<_, _>>()?;
    for pat in &patterns {
        for ch in array(pat, "channels")? {
            let ch = array(ch, "channel")?;
            for value in ch {
                if !value.is_null() && !value.is_number() {
                    return Err(fail("channel values must be numeric"));
                }
            }
            if ch
                .iter()
                .skip(2)
                .any(|v| v.as_f64().is_some_and(|n| n > 0. && n < 1.))
            {
                return Err(fail("volume-only tracker rows are not supported"));
            }
            let index = ch.first().and_then(Value::as_f64).unwrap_or(0.);
            if index < 0. || index.fract() != 0. || index >= instruments.len() as f64 {
                return Err(fail("invalid channel instrument index"));
            }
        }
    }
    let mut tracks = Vec::new();
    for c in 0..channels {
        tracks.push(
            song.add_track(TrackSettings {
                volume: Volume::new(100).map_err(model)?,
                ..TrackSettings::new(format!("ZzFX channel {}", c + 1))
            })
            .map_err(model)?,
        );
    }
    let mut patches = HashMap::new();
    let mut native_patterns = Vec::new();
    for (bar, &pi) in sequence.iter().enumerate() {
        let pat = patterns[pi];
        let label = order
            .get(pi)
            .cloned()
            .unwrap_or_else(|| format!("{}", pi + 1));
        let mut ids = Vec::new();
        for (ci, channel) in array(pat, "channels")?.iter().enumerate() {
            let channel = array(channel, "channel")?;
            if !channel
                .iter()
                .skip(2)
                .any(|v| v.as_f64().is_some_and(|n| n >= 1.))
            {
                continue;
            }
            let inst = channel.first().and_then(Value::as_f64).unwrap_or(0.) as usize;
            let pan = channel.get(1).and_then(Value::as_f64).unwrap_or(0.);
            if !(-1. ..=1.).contains(&pan) {
                return Err(fail("channel pan must be between -1 and 1"));
            }
            let raw = array(
                instruments
                    .get(inst)
                    .ok_or_else(|| fail("unknown instrument index"))?,
                "instrument parameters",
            )?;
            let mut params = Vec::new();
            let defaults = Vec::<f64>::from(ZzfxParameters::try_from(Vec::new()).map_err(model)?);
            for (i, v) in raw.iter().enumerate() {
                params.push(if v.is_null() {
                    *defaults.get(i).ok_or_else(|| fail("too many parameters"))?
                } else {
                    v.as_f64()
                        .ok_or_else(|| fail("instrument parameters must be numbers"))?
                });
            }
            // Studio metadata trims trailing zero parameters, not upstream defaults.
            if params.len() > 21 {
                return Err(fail("too many ZzFX parameters"));
            }
            if logical {
                params.resize(21, 0.);
            }
            let parameters = ZzfxParameters::try_from(params).map_err(model)?;
            let pan_percent = (pan * 100.).round() as i8;
            let key = (inst, pan_percent);
            let id = if let Some(&id) = patches.get(&key) {
                id
            } else {
                let mut instrument = Instrument::new(
                    format!("ZzFX {} / channel {}", inst + 1, ci + 1),
                    Voice::Zzfx { parameters },
                );
                instrument.pan = Pan::new(pan_percent).map_err(model)?;
                // Balance-pan equivalent of ZzFXM's (1 +/- pan)/2 law.
                instrument.volume =
                    Volume::new(((1. + pan.abs()) * 50.).round() as u8).map_err(model)?;
                instrument.pan =
                    Pan::new((2. * pan / (1. + pan.abs()) * 100.).round() as i8).map_err(model)?;
                let id = song.add_instrument(instrument).map_err(model)?;
                patches.insert(key, id);
                id
            };
            let role = data["patternRoles"][&label].as_str().unwrap_or("phrase");
            let mut pattern = Pattern::new(format!("{label} / {role} / {}", ci + 1), id);
            let effects = if data["patternEffects"].is_object() {
                &data["patternEffects"][&label][ci]
            } else {
                &data["patternEffects"][pi][ci]
            };
            let mut row_effects = HashMap::new();
            if let Some(list) = effects.as_array() {
                for (row, fx) in list.iter().enumerate() {
                    if fx.is_null() {
                        continue;
                    }
                    let row = fx["r"].as_u64().map_or(row, |r| r as usize);
                    let code = fx
                        .get("c")
                        .or(fx.get("code"))
                        .ok_or_else(|| fail("missing tracker effect code"))?;
                    let value = fx
                        .get("v")
                        .or(fx.get("value"))
                        .ok_or_else(|| fail("missing tracker effect value"))?;
                    let effect: ZzfxEffect =
                        serde_json::from_value(serde_json::json!({"code":code,"value":value}))
                            .map_err(|e| fail(format!("unsupported effect: {e}")))?;
                    row_effects.insert(row, effect);
                }
            }
            for row in 0..rows {
                let value = channel.get(row + 2).and_then(Value::as_f64).unwrap_or(0.);
                if value <= 0. {
                    continue;
                }
                if value < 1. {
                    return Err(fail(
                        "volume-only tracker rows are not supported; bake them into note velocities",
                    ));
                }
                let midi = value.trunc() + 48.;
                if midi > 127. {
                    return Err(fail("ZzFX note is outside CTG's MIDI range"));
                }
                let end = (row + 1..rows)
                    .find(|&r| channel.get(r + 2).and_then(Value::as_f64).unwrap_or(0.) != 0.)
                    .unwrap_or(rows);
                let mut note = Note::new(
                    Pitch::new(midi as u8).map_err(model)?,
                    row as u16,
                    (end - row) as u16,
                )
                .map_err(model)?;
                if end == rows {
                    let mut tail = 0;
                    'future: for &next in &sequence[bar + 1..] {
                        let next = &patterns[next][ci];
                        let next_inst = next[0].as_f64().unwrap_or(0.) as usize;
                        if next_inst != inst {
                            break;
                        }
                        for step in 0..rows {
                            if next[step + 2].as_f64().unwrap_or(0.) != 0. {
                                break 'future;
                            }
                            tail += 1;
                        }
                    }
                    note.tail_steps = tail;
                }
                note.velocity =
                    Velocity::new(((1. - value.fract()) * 127.).round().clamp(1., 127.) as u8)
                        .map_err(model)?;
                note.zzfx_effect = row_effects.get(&row).copied();
                pattern.insert_note(note).map_err(model)?;
            }
            let existing = song
                .patterns()
                .find_map(|(id, p)| (p == &pattern).then_some(id));
            ids.push((
                ci,
                if let Some(id) = existing {
                    id
                } else {
                    song.add_pattern(pattern).map_err(model)?
                },
            ));
        }
        native_patterns.push(ids);
    }
    for (bar, ids) in native_patterns.iter().enumerate() {
        for (channel, id) in ids {
            song.place_clip(tracks[*channel], bar as u16, *id)
                .map_err(model)?;
        }
    }
    Ok(song)
}
