//! `ZzFX` synthesis, ported from Frank Force's MIT-licensed `ZzFX` 1.3 algorithm.
//! Sounds are compiled off the audio thread; playback only reads sample buffers.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::too_many_lines,
    reason = "direct bounded DSP port using upstream parameter names"
)]
use std::f64::consts::TAU;

/// Deterministic `ZzFX` sound at 44.1 kHz. Randomness uses a stable per-sound seed.
pub(crate) fn build(p: [f64; 21], seed: u64) -> Vec<f32> {
    let [
        volume,
        randomness,
        frequency,
        attack,
        sustain,
        release,
        shape,
        curve,
        slide,
        delta_slide,
        pitch_jump,
        jump_time,
        repeat_time,
        noise,
        modulation,
        bit_crush,
        delay,
        sustain_volume,
        decay,
        tremolo,
        filter,
    ] = p;
    let sr = 44100.;
    let random = ((seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1) >> 11) as f64)
        / ((1_u64 << 53) as f64);
    let mut slide = slide * 500. * TAU / sr / sr;
    let start_slide = slide;
    let mut frequency = frequency * (1. + randomness * (2. * random - 1.)) * TAU / sr;
    let mut start_frequency = frequency;
    let attack = if attack == 0. { 9. } else { attack * sr };
    let (decay, sustain, release, delay) = (decay * sr, sustain * sr, release * sr, delay * sr);
    let delta_slide = delta_slide * 500. * TAU / sr.powi(3);
    let modulation = modulation * TAU / sr;
    let pitch_jump = pitch_jump * TAU / sr;
    let jump_time = jump_time * sr;
    let repeat_time = (repeat_time * sr) as usize;
    let crush_period = (bit_crush * 100.) as usize;
    let length = (attack + decay + sustain + release + delay) as usize;
    let volume = volume * 0.3; // ZzFX's default master volume.
    let mut b = Vec::<f64>::with_capacity(length);
    let (mut t, mut s, mut jump) = (0_f64, 0_f64, 1_usize);
    let w = TAU * filter.abs() * 2. / sr;
    let cos = w.cos();
    let alpha = w.sin() / 4.;
    let a0 = 1. + alpha;
    let sign = |v: f64| if v < 0. { -1. } else { 1. };
    let (a1, a2) = (-2. * cos / a0, (1. - alpha) / a0);
    let b0 = f64::midpoint(1., sign(filter) * cos) / a0;
    let b1 = -(sign(filter) + cos) / a0;
    let (mut x1, mut x2, mut y1, mut y2) = (0., 0., 0., 0.);
    for i in 0..length {
        let at = i as f64;
        if crush_period == 0 || (i + 1) % crush_period == 0 {
            s = match shape as u8 {
                0 => t.sin(),
                1 => 1. - 4. * ((t / TAU + 0.5).floor() - t / TAU).abs(),
                2 => 1. - ((2. * t / TAU % 2. + 2.) % 2.),
                3 => t.tan().clamp(-1., 1.),
                4 => t.powi(3).sin(),
                _ => {
                    if t / TAU % 1. < curve / 2. {
                        1.
                    } else {
                        -1.
                    }
                }
            };
            if shape <= 4. {
                s = sign(s) * s.abs().powf(curve);
            }
            if repeat_time > 0 {
                s *= 1. - tremolo + tremolo * (TAU * at / repeat_time as f64).sin();
            }
            s *= if at < attack {
                at / attack
            } else if at < attack + decay {
                1. - (at - attack) / decay * (1. - sustain_volume)
            } else if at < attack + decay + sustain {
                sustain_volume
            } else if at < length as f64 - delay {
                (length as f64 - at - delay) / release * sustain_volume
            } else {
                0.
            };
            if delay > 0. {
                s = s / 2.
                    + if delay > at || volume == 0. {
                        0.
                    } else {
                        let fade = if at < length as f64 - delay {
                            1.
                        } else {
                            (length as f64 - at) / delay
                        };
                        fade * b[(at - delay) as usize] / 2. / volume
                    };
            }
            if filter != 0. {
                let y = b0 * s + b1 * x1 + b0 * x2 - a1 * y1 - a2 * y2;
                x2 = x1;
                x1 = s;
                y2 = y1;
                y1 = y;
                s = y;
            }
        }
        slide += delta_slide;
        frequency += slide;
        let f = frequency * (modulation * at).cos();
        t += f + f * noise * at.powi(5).sin();
        if jump > 0 {
            jump += 1;
            if jump as f64 > jump_time {
                frequency += pitch_jump;
                start_frequency += pitch_jump;
                jump = 0;
            }
        }
        if repeat_time > 0 && (i + 1) % repeat_time == 0 {
            frequency = start_frequency;
            slide = start_slide;
            if jump == 0 {
                jump = 1;
            }
        }
        let value = s * volume;
        b.push(if value.is_finite() { value } else { 0. });
    }
    b.into_iter().map(|v| v as f32).collect()
}
