//! Expand Studio's logical channels into independent effect and drum lanes.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "format limits validate indices, MIDI pitches and normalized levels before conversion"
)]
use serde_json::{Value, json};
use soundbit_core::{ZzfxEffect, ZzfxParameters};
use std::collections::HashMap;

type VariantKey = (usize, usize, u8, Option<ZzfxEffect>);

#[allow(
    clippy::too_many_lines,
    reason = "three ordered passes mirror the Studio channel expansion algorithm"
)]
pub(super) fn expand(data: &Value) -> Result<Value, String> {
    let order = data["patternOrder"]
        .as_array()
        .ok_or("logical Studio project needs patternOrder")?;
    if order.is_empty() || order.len() > 512 {
        return Err("invalid patternOrder length".into());
    }
    let source = data["instruments"]
        .as_array()
        .ok_or("missing instruments")?;
    if source.is_empty() || source.len() > 128 {
        return Err("expected 1-128 instruments".into());
    }
    let mut instruments = Vec::new();
    for values in source {
        let raw = values.as_array().ok_or("invalid instrument")?;
        if raw.len() > 21 {
            return Err("too many ZzFX parameters".into());
        }
        let mut p = vec![0.; 21];
        for (i, v) in raw.iter().enumerate() {
            p[i] = v.as_f64().ok_or("instrument parameter must be numeric")?;
        }
        instruments.push(ZzfxParameters::try_from(p)?.values().to_vec());
    }
    let mut originals = Vec::<Vec<Vec<f64>>>::new();
    let mut effect_maps = Vec::new();
    let mut rows = 32;
    for (i, label) in order.iter().enumerate() {
        let label = label.as_str().ok_or("invalid pattern label")?;
        let pattern = if data["patterns"].is_object() {
            &data["patterns"][label]
        } else {
            &data["patterns"][i]
        };
        let channels: Vec<Vec<f64>> =
            serde_json::from_value(pattern.clone()).map_err(|e| e.to_string())?;
        if channels.len() != 4 {
            return Err("logical Studio project needs four channels".into());
        }
        for ch in &channels {
            let inst = ch.first().copied().unwrap_or(0.);
            if inst < 0. || inst.fract() != 0. || inst >= source.len() as f64 {
                return Err("invalid channel instrument index".into());
            }
            if ch.iter().skip(2).any(|n| *n > 0. && *n < 1.) {
                return Err("volume-only tracker rows are not supported".into());
            }
            rows = rows.max(ch.len().saturating_sub(2));
        }
        originals.push(channels);
        let effects = if data["patternEffects"].is_object() {
            &data["patternEffects"][label]
        } else {
            &data["patternEffects"][i]
        };
        let mut map = HashMap::new();
        for c in 0..4 {
            if let Some(fx) = effects[c].as_array() {
                for (r, e) in fx.iter().enumerate() {
                    if e.is_null() {
                        continue;
                    }
                    let row = e["r"].as_u64().map_or(r, |v| v as usize);
                    let effect:ZzfxEffect=serde_json::from_value(json!({"code":e.get("c").or(e.get("code")),"value":e.get("v").or(e.get("value"))})).map_err(|e|e.to_string())?;
                    map.insert((c, row), effect);
                }
            }
        }
        effect_maps.push(map);
    }
    if rows > 64 {
        return Err("logical pattern exceeds 64 rows".into());
    }
    let mut variants: HashMap<VariantKey, (usize, usize)> = HashMap::new();
    let mut channels = 4;
    for (pi, pattern) in originals.iter().enumerate() {
        for (c, ch) in pattern.iter().enumerate() {
            for row in 0..rows {
                let note = ch.get(row + 2).copied().unwrap_or(0.);
                if note <= 0. {
                    continue;
                }
                let effect = effect_maps[pi].get(&(c, row)).copied();
                let drum = if c != 3 {
                    0
                } else if note <= 6. {
                    1
                } else if note <= 22. {
                    2
                } else {
                    3
                };
                if effect.is_none() && drum == 0 {
                    continue;
                }
                let inst = ch.first().copied().unwrap_or(0.) as usize;
                let key = (c, inst, drum, effect);
                if variants.contains_key(&key) {
                    continue;
                }
                let base = instruments.get(inst).ok_or("unknown instrument")?.clone();
                let mut p: [f64; 21] = base.try_into().map_err(|_| "invalid parameter count")?;
                drum_patch(&mut p, drum);
                if let Some(effect) = effect {
                    effect.apply(&mut p);
                }
                variants.insert(key, (instruments.len(), channels));
                instruments.push(p.to_vec());
                channels += 1;
                if channels > 128 || instruments.len() > 128 {
                    return Err("expanded project exceeds 128 sound lanes".into());
                }
            }
        }
    }
    let mut patterns = Vec::new();
    for (pi, pattern) in originals.iter().enumerate() {
        let mut physical = vec![vec![0.; rows + 2]; channels];
        for (c, ch) in pattern.iter().enumerate() {
            physical[c][0] = ch.first().copied().unwrap_or(0.);
            physical[c][1] = ch.get(1).copied().unwrap_or(0.);
        }
        for (&(c, _, _, _), &(_, lane)) in &variants {
            physical[lane][1] = pattern[c].get(1).copied().unwrap_or(0.);
        }
        for (c, ch) in pattern.iter().enumerate() {
            for row in 0..rows {
                let note = ch.get(row + 2).copied().unwrap_or(0.);
                if note <= 0. {
                    continue;
                }
                let effect = effect_maps[pi].get(&(c, row)).copied();
                let drum = if c != 3 {
                    0
                } else if note <= 6. {
                    1
                } else if note <= 22. {
                    2
                } else {
                    3
                };
                let inst = ch.first().copied().unwrap_or(0.) as usize;
                if let Some(&(instrument, lane)) = variants.get(&(c, inst, drum, effect)) {
                    physical[lane][0] = instrument as f64;
                    physical[lane][row + 2] = note;
                } else {
                    physical[c][row + 2] = note;
                }
            }
        }
        patterns.push(physical);
    }
    Ok(
        json!({"instruments":instruments,"patterns":patterns,"sequence":data["sequence"],"config":data["config"],"patternOrder":data["patternOrder"],"patternRoles":data["patternRoles"],"expanded":true,"author":data["author"]}),
    )
}

fn drum_patch(p: &mut [f64; 21], voice: u8) {
    match voice {
        1 => {
            p[6] = 0.;
            p[2] *= 0.55;
            p[8] = 0.;
            p[10] = -0.25 * p[2];
            p[11] = 0.012;
            p[0] = (p[0] * 1.1).min(1.);
        }
        2 => {
            p[13] = (p[13] * 1.25).min(1.);
            p[15] = p[15].min(0.02);
        }
        3 => {
            p[2] *= 2.6;
            p[4] *= 0.4;
            p[5] *= 0.35;
            p[18] *= 0.35;
            p[13] = (p[13] * 1.5).min(1.);
            p[0] *= 0.6;
            p[15] = p[15].min(0.02);
        }
        _ => return,
    }
    let minimum = if voice == 3 { 0.036 } else { 0.06 };
    let body = p[18] + p[4] + p[5];
    if body > 0. && body < minimum {
        for i in [18, 4, 5] {
            p[i] *= minimum / body;
        }
    }
}
