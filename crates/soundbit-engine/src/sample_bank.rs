//! Bounded, shared procedural samples prepared before realtime playback.
use crate::{
    error::EngineError,
    patch::{Patch, SourcePatch},
    schedule::TrackPlan,
};
use std::collections::HashMap;

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "nonnegative validated envelope durations are at most 15 seconds"
)]
pub(crate) fn compile(
    tracks: &mut [TrackPlan],
    patches: &[Patch],
) -> Result<Vec<Vec<f32>>, EngineError> {
    let mut cache = HashMap::new();
    let mut samples = Vec::new();
    let mut count = 0;
    for event in tracks.iter_mut().flat_map(|t| &mut t.events) {
        let SourcePatch::Zzfx(parameters) = patches[event.patch].source else {
            continue;
        };
        let key = (
            parameters,
            event.pitch,
            event.detune_cents,
            event.detune_milli_hz,
            event.zzfx_effect,
        );
        if let Some(&index) = cache.get(&key) {
            event.sample = Some(index);
            continue;
        }
        let mut p = parameters.values();
        if let Some(effect) = event.zzfx_effect {
            effect.apply(&mut p);
        }
        p[2] *=
            2_f64.powf((f64::from(event.pitch) - 60. + f64::from(event.detune_cents) / 100.) / 12.);
        p[2] += f64::from(event.detune_milli_hz) / 1000.;
        let estimated = ((p[3] + p[4] + p[5] + p[16] + p[18]) * 44100. + 10.) as usize;
        count += estimated;
        if count > 32 * 1024 * 1024 {
            return Err(EngineError::SampleCacheLimit);
        }
        let data = crate::zzfx::build(p, u64::from(event.pitch) + event.patch as u64 * 131);
        event.sample = Some(samples.len());
        cache.insert(key, samples.len());
        samples.push(data);
    }
    Ok(samples)
}
