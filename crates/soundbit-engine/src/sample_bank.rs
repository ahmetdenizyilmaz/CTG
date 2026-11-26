//! Bounded, shared procedural samples prepared before realtime playback.
use crate::{
    error::EngineError,
    patch::{Patch, SourcePatch},
    physical_model::{PhysicalModelKind, synthesize},
    schedule::TrackPlan,
};
use std::collections::HashMap;

const BANK_SAMPLE_RATE: u32 = 44_100;
const MAX_SAMPLE_FLOATS: usize = 48 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PhysKey {
    kind: PhysicalModelKind,
    pitch: u8,
    detune_cents: i16,
    detune_milli_hz: i32,
    vel_bucket: u8,
    dur_bucket: usize,
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "nonnegative validated envelope durations are bounded"
)]
pub(crate) fn compile(
    tracks: &mut [TrackPlan],
    patches: &[Patch],
    sample_rate: u32,
    song_end: u64,
) -> Result<Vec<Vec<f32>>, EngineError> {
    let mut zzfx_cache = HashMap::new();
    let mut phys_cache: HashMap<PhysKey, usize> = HashMap::new();
    let mut samples = Vec::new();
    let mut count = 0_usize;
    let sr_f = f64::from(sample_rate);

    // 1. Respect written note durations with a short natural release (no multi-second overlap mud)
    for track in tracks.iter_mut() {
        let len = track.events.len();
        for i in 0..len {
            let patch_idx = track.events[i].patch;
            let Some(patch) = patches.get(patch_idx) else {
                continue;
            };
            let SourcePatch::PhysicalModel(kind) = patch.source else {
                continue;
            };
            if !kind.is_percussive_sustain() {
                continue;
            }
            let on = track.events[i].on;
            let written_dur = track.events[i].off.saturating_sub(on);
            // Add at most a clean 120ms natural release tail, never overlapping the next note on the same track
            let tail = (0.12 * sr_f) as u64;
            let mut target_off = (track.events[i].off + tail).min(song_end.max(track.events[i].off));
            if let Some(next_ev) = track.events.get(i + 1)
                && next_ev.on > on
            {
                // If it's a short staccato/stab note or bassline, stop cleanly before the next attack
                let max_overlap = if written_dur <= (0.25 * sr_f) as u64 || kind == PhysicalModelKind::UprightBass {
                    (0.03 * sr_f) as u64
                } else {
                    (0.10 * sr_f) as u64
                };
                target_off = target_off.min(next_ev.on.saturating_add(max_overlap));
            }
            track.events[i].off = target_off;
        }
    }

    // 2. Synthesize and cache procedural ZzFX and PhysicalModel samples at 44.1 kHz
    for event in tracks.iter_mut().flat_map(|t| &mut t.events) {
        match patches[event.patch].source {
            SourcePatch::Zzfx(parameters) => {
                let key = (
                    parameters,
                    event.pitch,
                    event.detune_cents,
                    event.detune_milli_hz,
                    event.zzfx_effect,
                );
                if let Some(&index) = zzfx_cache.get(&key) {
                    event.sample = Some(index);
                    continue;
                }
                let mut p = parameters.values();
                if let Some(effect) = event.zzfx_effect {
                    effect.apply(&mut p);
                }
                p[2] *= 2_f64
                    .powf((f64::from(event.pitch) - 60. + f64::from(event.detune_cents) / 100.) / 12.);
                p[2] += f64::from(event.detune_milli_hz) / 1000.;
                let estimated = ((p[3] + p[4] + p[5] + p[16] + p[18]) * 44100. + 10.) as usize;
                count += estimated;
                if count > MAX_SAMPLE_FLOATS {
                    return Err(EngineError::SampleCacheLimit);
                }
                let data = crate::zzfx::build(p, u64::from(event.pitch) + event.patch as u64 * 131);
                event.sample = Some(samples.len());
                zzfx_cache.insert(key, samples.len());
                samples.push(data);
            }
            SourcePatch::PhysicalModel(kind) => {
                let dur_native = event.off.saturating_sub(event.on).max(64);
                let dur_44k = ((dur_native as f64) * f64::from(BANK_SAMPLE_RATE) / sr_f) as usize;
                // Quantize duration to 40ms buckets and velocity to 6 levels for high cache hit rate
                let dur_bucket = ((dur_44k + 882) / 1764).max(1) * 1764;
                let vel_bucket = (event.velocity * 6.0).round().clamp(1.0, 6.0) as u8;
                let key = PhysKey {
                    kind,
                    pitch: event.pitch,
                    detune_cents: event.detune_cents,
                    detune_milli_hz: event.detune_milli_hz,
                    vel_bucket,
                    dur_bucket,
                };
                if let Some(&index) = phys_cache.get(&key) {
                    event.sample = Some(index);
                    continue;
                }
                count += dur_bucket;
                if count > MAX_SAMPLE_FLOATS {
                    return Err(EngineError::SampleCacheLimit);
                }
                let vel_norm = f32::from(vel_bucket) / 6.0;
                let data = synthesize(
                    kind,
                    event.pitch,
                    event.detune_cents,
                    event.detune_milli_hz,
                    vel_norm,
                    dur_bucket,
                    BANK_SAMPLE_RATE,
                );
                let idx = samples.len();
                event.sample = Some(idx);
                phys_cache.insert(key, idx);
                samples.push(data);
            }
            _ => {}
        }
    }
    Ok(samples)
}
