//! Physical sound models for realistic acoustic and studio instruments.
//!
//! Synthesizes sample-accurate waveforms off the audio thread for [`crate::sample_bank`]:
//! - **Stiff-string inharmonic piano models** (`GrandPiano`, `FeltPiano`, `UprightPiano`):
//!   Fletcher's stiff-string dispersion ($f_n = n f_0 \sqrt{1 + B n^2}$), trichord 3-string
//!   detuning with Weinreich two-stage bridge decay (prompt sound + singing aftersound),
//!   velocity-dependent nonlinear felt-hammer strike at $x_0 = 1/7.4$, wooden keybed thud,
//!   and spruce soundboard modal resonators.
//! - **Digital waveguide plucked strings** (`NylonGuitar`, `SteelGuitar`, `ConcertHarp`,
//!   `UprightBass`): Extended Karplus-Strong delay lines with pick-position comb filtering,
//!   all-pass string dispersion, two-stage bridge damping, and wooden body cavity biquads.
//! - **Helmholtz bowed strings with fixed bridge formants** (`SoloViolin`, `SoloCello`,
//!   `StringQuartet`): Stick-slip bowed harmonics + rosin bow friction filtered through
//!   fixed Cremonese body formants so pitch vibrato produces authentic timbral shimmer.
//! - **Euler-Bernoulli beam & reed models** (`RhodesMark1`, `Wurlitzer200A`, `Vibraphone`,
//!   `Marimba`, `Celesta`) and **acoustic woodwinds & brush percussion** (`ConcertFlute`,
//!   `WarmClarinet`, `BrushSnare`).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::too_many_lines,
    reason = "physical-modelling DSP equations operate on bounded audio buffers and harmonic indices"
)]

use std::f64::consts::{PI, TAU};

use soundbit_core::Instrument;
use soundbit_dsp::math::note_frequency;

/// Acoustic and electro-acoustic physical sound models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PhysicalModelKind {
    GrandPiano,
    FeltPiano,
    UprightPiano,
    Celesta,
    RhodesMark1,
    Wurlitzer200A,
    NylonGuitar,
    SteelGuitar,
    ConcertHarp,
    Vibraphone,
    Marimba,
    SoloViolin,
    SoloCello,
    StringQuartet,
    ConcertFlute,
    WarmClarinet,
    UprightBass,
    BrushSnare,
}

impl PhysicalModelKind {
    /// Detects whether an instrument should be rendered by a physical sound model.
    pub(crate) fn from_instrument(instrument: &Instrument) -> Option<Self> {
        let name = instrument.name.trim().to_ascii_lowercase();
        if name.contains("felt piano") {
            return Some(Self::FeltPiano);
        }
        if name.contains("upright piano") || name.contains("honky") {
            return Some(Self::UprightPiano);
        }
        if name.contains("grand piano") || name == "piano" || name == "acoustic piano" {
            return Some(Self::GrandPiano);
        }
        if name.contains("celesta") || name.contains("music box") {
            return Some(Self::Celesta);
        }
        if name.contains("rhodes")
            || name.contains("vintage rhodes")
            || name == "e. piano"
            || name == "ctg mellow ep"
            || name == "fusion studio rhodes ep"
        {
            return Some(Self::RhodesMark1);
        }
        if name.contains("wurlitzer") || name.contains("reed ep") {
            return Some(Self::Wurlitzer200A);
        }
        if name.contains("nylon guitar") || name.contains("classical guitar") {
            return Some(Self::NylonGuitar);
        }
        if name.contains("steel guitar") || name.contains("acoustic guitar") {
            return Some(Self::SteelGuitar);
        }
        if name.contains("concert harp") || name == "harp" || name == "acoustic harp" {
            return Some(Self::ConcertHarp);
        }
        if name.contains("vibraphone") || name == "vibes" {
            return Some(Self::Vibraphone);
        }
        if name.contains("marimba") || name.contains("xylophone") {
            return Some(Self::Marimba);
        }
        if name.contains("solo violin") || name == "violin" || name == "acoustic violin" {
            return Some(Self::SoloViolin);
        }
        if name.contains("solo cello")
            || name.contains("acoustic solo cello")
            || name == "aloboi solo cello"
            || name == "fusion tape cello"
        {
            return Some(Self::SoloCello);
        }
        if name.contains("string quartet")
            || name.contains("chamber strings")
            || name == "strings"
            || name == "fm strings"
        {
            return Some(Self::StringQuartet);
        }
        if name.contains("concert flute")
            || name == "flute"
            || name.contains("nostalgic ocarina")
            || name.contains("fusion ocarina")
        {
            return Some(Self::ConcertFlute);
        }
        if name.contains("clarinet") || name.contains("oboe") {
            return Some(Self::WarmClarinet);
        }
        if name.contains("upright bass")
            || name.contains("double bass")
            || name.contains("fretless")
            || name.contains("fingered p-bass")
        {
            return Some(Self::UprightBass);
        }
        if name.contains("brush snare") {
            return Some(Self::BrushSnare);
        }
        None
    }

    /// Whether this instrument is struck or plucked and therefore rings out on a sustain pedal
    /// or natural soundboard after a short tracker step ends.
    pub(crate) const fn is_percussive_sustain(self) -> bool {
        matches!(
            self,
            Self::GrandPiano
                | Self::FeltPiano
                | Self::UprightPiano
                | Self::Celesta
                | Self::RhodesMark1
                | Self::Wurlitzer200A
                | Self::NylonGuitar
                | Self::SteelGuitar
                | Self::ConcertHarp
                | Self::Vibraphone
                | Self::Marimba
                | Self::UprightBass
                | Self::BrushSnare
        )
    }

    /// Minimum natural sustain duration in seconds when a note is struck in a tracker pattern.
    pub(crate) const fn min_ring_seconds(self) -> f64 {
        match self {
            Self::GrandPiano | Self::FeltPiano => 2.4,
            Self::UprightPiano | Self::ConcertHarp | Self::Vibraphone => 2.0,
            Self::RhodesMark1 | Self::Wurlitzer200A | Self::Celesta => 1.7,
            Self::NylonGuitar | Self::SteelGuitar => 1.6,
            Self::Marimba | Self::UprightBass => 1.1,
            Self::BrushSnare => 0.35,
            Self::SoloViolin
            | Self::SoloCello
            | Self::StringQuartet
            | Self::ConcertFlute
            | Self::WarmClarinet => 0.0,
        }
    }
}

/// Deterministic xorshift64 PRNG returning `[-1.0, 1.0]`.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next_f64(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        ((self.0 >> 11) as f64) * (2.0 / ((1u64 << 53) as f64)) - 1.0
    }
}

/// Second-order peaking/bandpass resonator for wooden soundboards and violin/cello bridges.
#[derive(Clone, Copy)]
struct ModalBiquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    z1: f64,
    z2: f64,
}

impl ModalBiquad {
    fn bandpass(freq_hz: f64, q: f64, sample_rate: f64) -> Self {
        let f = freq_hz.clamp(25.0, sample_rate * 0.45);
        let w0 = TAU * f / sample_rate;
        let (sin_w0, cos_w0) = w0.sin_cos();
        let alpha = sin_w0 / (2.0 * q.max(0.4));
        let a0 = 1.0 + alpha;
        Self {
            b0: (alpha / a0),
            b1: 0.0,
            b2: (-alpha / a0),
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    fn lowpass(freq_hz: f64, q: f64, sample_rate: f64) -> Self {
        let f = freq_hz.clamp(30.0, sample_rate * 0.46);
        let w0 = TAU * f / sample_rate;
        let (sin_w0, cos_w0) = w0.sin_cos();
        let alpha = sin_w0 / (2.0 * q.max(0.5));
        let b1 = 1.0 - cos_w0;
        let b0 = b1 * 0.5;
        let a0 = 1.0 + alpha;
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b0 / a0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

/// Synthesizes one note of `kind` at `sample_rate` Hz.
pub(crate) fn synthesize(
    kind: PhysicalModelKind,
    pitch: u8,
    detune_cents: i16,
    detune_milli_hz: i32,
    velocity: f32,
    duration_samples: usize,
    sample_rate: u32,
) -> Vec<f32> {
    let sr = f64::from(sample_rate);
    let base_hz = note_frequency(pitch) * 2.0_f64.powf(f64::from(detune_cents) / 1200.0)
        + f64::from(detune_milli_hz) / 1000.0;
    let hz = base_hz.clamp(22.0, sr * 0.42);
    let vel = f64::from(velocity).clamp(0.15, 1.0);
    let seed = (u64::from(pitch) << 16) ^ ((kind as u64) * 0x517C_C1B7);

    let total_samples = duration_samples.clamp(64, (sr * 12.0) as usize);
    let release_sec = if kind.is_percussive_sustain() {
        0.28
    } else {
        0.18
    };
    let release_samples = ((release_sec * sr) as usize).min(total_samples / 3).max(16);
    let sustain_end = total_samples.saturating_sub(release_samples);

    let mut out = match kind {
        PhysicalModelKind::GrandPiano => synth_piano(hz, vel, total_samples, sr, seed, PianoType::Grand),
        PhysicalModelKind::FeltPiano => synth_piano(hz, vel, total_samples, sr, seed, PianoType::Felt),
        PhysicalModelKind::UprightPiano => synth_piano(hz, vel, total_samples, sr, seed, PianoType::Upright),
        PhysicalModelKind::Celesta => synth_mallet_or_tine(hz, vel, total_samples, sr, seed, BeamType::Celesta),
        PhysicalModelKind::RhodesMark1 => synth_mallet_or_tine(hz, vel, total_samples, sr, seed, BeamType::Rhodes),
        PhysicalModelKind::Wurlitzer200A => synth_mallet_or_tine(hz, vel, total_samples, sr, seed, BeamType::Wurlitzer),
        PhysicalModelKind::Vibraphone => synth_mallet_or_tine(hz, vel, total_samples, sr, seed, BeamType::Vibraphone),
        PhysicalModelKind::Marimba => synth_mallet_or_tine(hz, vel, total_samples, sr, seed, BeamType::Marimba),
        PhysicalModelKind::NylonGuitar => synth_waveguide_pluck(hz, vel, total_samples, sr, seed, PluckType::NylonGuitar),
        PhysicalModelKind::SteelGuitar => synth_waveguide_pluck(hz, vel, total_samples, sr, seed, PluckType::SteelGuitar),
        PhysicalModelKind::ConcertHarp => synth_waveguide_pluck(hz, vel, total_samples, sr, seed, PluckType::ConcertHarp),
        PhysicalModelKind::UprightBass => synth_waveguide_pluck(hz, vel, total_samples, sr, seed, PluckType::UprightBass),
        PhysicalModelKind::SoloViolin => synth_bowed_string(hz, vel, sustain_end, total_samples, sr, seed, BowedType::Violin),
        PhysicalModelKind::SoloCello => synth_bowed_string(hz, vel, sustain_end, total_samples, sr, seed, BowedType::Cello),
        PhysicalModelKind::StringQuartet => synth_bowed_string(hz, vel, sustain_end, total_samples, sr, seed, BowedType::Quartet),
        PhysicalModelKind::ConcertFlute => synth_woodwind(hz, vel, sustain_end, total_samples, sr, seed, WindType::Flute),
        PhysicalModelKind::WarmClarinet => synth_woodwind(hz, vel, sustain_end, total_samples, sr, seed, WindType::Clarinet),
        PhysicalModelKind::BrushSnare => synth_brush_snare(vel, total_samples, sr, seed),
    };

    // Apply smooth damper release envelope over the final `release_samples` and normalize safely
    let mut peak = 0.0_f32;
    for (i, sample) in out.iter_mut().enumerate() {
        if i >= sustain_end {
            let rel_u = ((i - sustain_end) as f64 / release_samples as f64).clamp(0.0, 1.0);
            let damper = 0.5 * (1.0 + (PI * rel_u).cos());
            *sample *= damper as f32;
        }
        let abs = sample.abs();
        if abs > peak {
            peak = abs;
        }
    }

    // Target peak 0.85 (voice.rs scales amplitude by velocity, while `vel` above shapes hammer/bow timbre)
    let target_peak = 0.85_f32;
    if peak > 1e-4 {
        let scale = target_peak / peak;
        for sample in &mut out {
            *sample = (*sample * scale).clamp(-0.98, 0.98);
        }
    }
    out
}

#[derive(Clone, Copy)]
enum PianoType {
    Grand,
    Felt,
    Upright,
}

/// Physical model of a struck stiff piano string trichord coupled to a spruce soundboard.
fn synth_piano(
    f0: f64,
    vel: f64,
    total_samples: usize,
    sr: f64,
    seed: u64,
    piano_type: PianoType,
) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];

    // 1. Fletcher's stiff-string inharmonicity coefficient B(f0)
    // Higher in treble (short stiff wire) and low wound bass strings
    let b_base = match piano_type {
        PianoType::Grand => 0.000_11,
        PianoType::Felt => 0.000_09,
        PianoType::Upright => 0.000_19, // shorter upright strings = slightly higher B
    };
    let b_coeff = (b_base * (f0 / 261.63).powf(1.18) + if f0 < 110.0 { 0.000_16 } else { 0.0 })
        .clamp(0.000_04, 0.0025);

    // 2. Number of audible partials below Nyquist
    let max_partials = match piano_type {
        PianoType::Grand => 22,
        PianoType::Felt => 11,
        PianoType::Upright => 18,
    };
    let nyquist = sr * 0.44;

    // 3. Trichord detuning in cents (3 strings per key above 95 Hz, 2 strings in bass)
    let detune_cents: [f64; 3] = match piano_type {
        PianoType::Grand => [-1.15, 0.0, 1.35],
        PianoType::Felt => [-1.6, 0.0, 1.8],
        PianoType::Upright => [-2.6, 0.0, 2.9],
    };

    // 4. Weinreich two-stage decay time constants (seconds)
    // Prompt sound (all 3 strings in phase) decays ~6x faster than the singing aftersound
    let pitch_scale = (261.63 / f0).powf(0.42).clamp(0.28, 2.2);
    let tau_prompt_base = match piano_type {
        PianoType::Grand => 0.24 * pitch_scale,
        PianoType::Felt => 0.30 * pitch_scale,
        PianoType::Upright => 0.19 * pitch_scale,
    };
    let tau_after_base = match piano_type {
        PianoType::Grand => 2.65 * pitch_scale,
        PianoType::Felt => 2.20 * pitch_scale,
        PianoType::Upright => 1.95 * pitch_scale,
    };

    // Hammer hardness exponent: soft notes roll off high partials rapidly; loud notes excite them
    let hardness_exp = match piano_type {
        PianoType::Grand => 2.15 - 0.85 * vel,
        PianoType::Felt => 2.85 - 0.65 * vel,
        PianoType::Upright => 2.00 - 0.80 * vel,
    };

    // Hammer strike position along string (x0 ≈ 1/7.4 suppresses 7th/8th partial harshness)
    let strike_pos = 1.0 / 7.4;

    // Precompute per-partial oscillator state to avoid calling sin()/exp() per sample!
    struct PartialState {
        re1: f64,
        im1: f64,
        d_re1: f64,
        d_im1: f64,
        re2: f64,
        im2: f64,
        d_re2: f64,
        d_im2: f64,
        re3: f64,
        im3: f64,
        d_re3: f64,
        d_im3: f64,
        env_prompt: f64,
        decay_prompt: f64,
        env_after: f64,
        decay_after: f64,
    }

    let mut partials = Vec::with_capacity(max_partials);
    for n_idx in 1..=max_partials {
        let n = n_idx as f64;
        // Stiff-string inharmonic frequency
        let fn_hz = n * f0 * (1.0 + b_coeff * n * n).sqrt();
        if fn_hz >= nyquist {
            break;
        }

        // Spatial hammer comb + velocity rolloff + subtle wood soundboard boost on n=1,2,3
        let spatial = (n * PI * strike_pos).sin().abs() * 0.82 + 0.18;
        let body_weight = if n_idx == 1 {
            1.15
        } else if n_idx == 2 {
            0.95
        } else if n_idx == 3 {
            0.72
        } else {
            1.0
        };
        let amp = (spatial * body_weight) / n.powf(hardness_exp);

        // Higher partials lose energy to the bridge and air much faster
        let loss_factor = 1.0 + 0.24 * (n - 1.0).powf(1.65);
        let tau_p = (tau_prompt_base / loss_factor).max(0.012);
        let tau_a = (tau_after_base / loss_factor).max(0.025);

        let decay_prompt = (-1.0 / (tau_p * sr)).exp();
        let decay_after = (-1.0 / (tau_a * sr)).exp();

        // Prompt stage carries ~62% of initial strike energy; aftersound carries ~38%
        let env_prompt = amp * 0.62;
        let env_after = amp * 0.38;

        // 3 trichord string frequencies for this partial
        // Higher partials have slightly reduced cents spread so they don't warble artificially
        let spread_atten = 1.0 / n.powf(0.45);
        let f_s1 = fn_hz * 2.0_f64.powf(detune_cents[0] * spread_atten / 1200.0);
        let f_s2 = fn_hz;
        let f_s3 = fn_hz * 2.0_f64.powf(detune_cents[2] * spread_atten / 1200.0);

        let w1 = TAU * f_s1 / sr;
        let w2 = TAU * f_s2 / sr;
        let w3 = TAU * f_s3 / sr;

        // Slight initial phase offsets on strings 1 & 3 from hammer felt curvature
        let p1 = 0.08 * n;
        let p2 = 0.0_f64;
        let p3 = -0.11 * n;

        partials.push(PartialState {
            re1: p1.cos(),
            im1: p1.sin(),
            d_re1: w1.cos(),
            d_im1: w1.sin(),
            re2: p2.cos(),
            im2: p2.sin(),
            d_re2: w2.cos(),
            d_im2: w2.sin(),
            re3: p3.cos(),
            im3: p3.sin(),
            d_re3: w3.cos(),
            d_im3: w3.sin(),
            env_prompt,
            decay_prompt,
            env_after,
            decay_after,
        });
    }

    // 5. Synthesize wooden hammer + keybed mechanical strike impulse (first 22 ms)
    let mut wood_low = ModalBiquad::bandpass(135.0, 2.2, sr);
    let mut wood_mid = ModalBiquad::bandpass((f0 * 1.5).clamp(240.0, 1400.0), 3.0, sr);
    let mut felt_soft = ModalBiquad::lowpass(
        match piano_type {
            PianoType::Felt => 1100.0,
            PianoType::Grand => 2600.0,
            PianoType::Upright => 2200.0,
        },
        0.8,
        sr,
    );

    // Spruce soundboard body warmth filter
    let mut soundboard_body = ModalBiquad::bandpass(210.0, 1.4, sr);
    let attack_samples = (0.0022 * sr).max(6.0);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        // Fast nonlinear felt-hammer compression attack (0..2.2 ms)
        let attack_env = if (i as f64) < attack_samples {
            let u = (i as f64) / attack_samples;
            u * u * (3.0 - 2.0 * u)
        } else {
            1.0
        };

        let mut string_sum = 0.0;
        for p in &mut partials {
            // Coupled 3-string sum: prompt stage is in-phase, aftersound has slight bridge differential
            let s_val = p.im2 * 0.44 + (p.im1 + p.im3) * 0.28;
            string_sum += s_val * (p.env_prompt + p.env_after);

            // Advance phasors via complex multiplication
            let nr1 = p.re1 * p.d_re1 - p.im1 * p.d_im1;
            p.im1 = p.re1 * p.d_im1 + p.im1 * p.d_re1;
            p.re1 = nr1;

            let nr2 = p.re2 * p.d_re2 - p.im2 * p.d_im2;
            p.im2 = p.re2 * p.d_im2 + p.im2 * p.d_re2;
            p.re2 = nr2;

            let nr3 = p.re3 * p.d_re3 - p.im3 * p.d_im3;
            p.im3 = p.re3 * p.d_im3 + p.im3 * p.d_re3;
            p.re3 = nr3;

            p.env_prompt *= p.decay_prompt;
            p.env_after *= p.decay_after;
        }

        // Hammer & keybed wooden knock transient (0..24 ms)
        let hammer_transient = if t < 0.028 {
            let h_env = (-t * 190.0).exp();
            let noise = felt_soft.process(rng.next_f64());
            let knock = (TAU * 125.0 * t).sin() * (-t * 240.0).exp();
            let w1 = wood_low.process(knock + noise * 0.5);
            let w2 = wood_mid.process(noise);
            let transient_gain = match piano_type {
                PianoType::Felt => 0.26,
                PianoType::Grand => 0.18,
                PianoType::Upright => 0.22,
            };
            (w1 * 0.65 + w2 * 0.35) * h_env * transient_gain * (0.5 + 0.5 * vel)
        } else {
            0.0
        };

        let body_res = soundboard_body.process(string_sum) * 0.18;
        *out_sample = ((string_sum + body_res) * attack_env + hammer_transient) as f32;
    }

    buf
}

#[derive(Clone, Copy)]
enum PluckType {
    NylonGuitar,
    SteelGuitar,
    ConcertHarp,
    UprightBass,
}

/// Extended Digital Waveguide (Karplus-Strong + all-pass dispersion + wooden body resonators).
fn synth_waveguide_pluck(
    f0: f64,
    vel: f64,
    total_samples: usize,
    sr: f64,
    seed: u64,
    pluck_type: PluckType,
) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];

    let delay_len_f = (sr / f0).clamp(8.0, 4096.0);
    let delay_len = delay_len_f.floor() as usize;
    let frac = delay_len_f - delay_len as f64;
    // First-order all-pass fractional delay coefficient
    let ap_coeff = (1.0 - frac) / (1.0 + frac);

    let mut line = vec![0.0_f64; delay_len + 2];
    let pick_pos = match pluck_type {
        PluckType::NylonGuitar => 0.22,
        PluckType::SteelGuitar => 0.14,
        PluckType::ConcertHarp => 0.28,
        PluckType::UprightBass => 0.25,
    };
    let pick_offset = ((delay_len as f64 * pick_pos) as usize).clamp(1, delay_len.saturating_sub(1));

    // Initialize waveguide with physical pluck displacement (triangle slope + finger/pick texture)
    let mut lp_init = 0.0;
    let init_smooth = match pluck_type {
        PluckType::NylonGuitar => 0.32,
        PluckType::SteelGuitar => 0.62,
        PluckType::ConcertHarp => 0.26,
        PluckType::UprightBass => 0.20,
    };
    for i in 0..delay_len {
        let u = i as f64 / delay_len as f64;
        let tri = if u < pick_pos {
            u / pick_pos
        } else {
            (1.0 - u) / (1.0 - pick_pos)
        };
        let n = rng.next_f64();
        lp_init += init_smooth * (n - lp_init);
        line[i] = tri * 0.65 + lp_init * (0.35 + 0.25 * vel);
    }
    // Comb filter for pick position + remove DC offset
    let copy = line.clone();
    let mut mean = 0.0;
    for i in 0..delay_len {
        let delayed = copy[(i + delay_len - pick_offset) % delay_len];
        line[i] -= 0.45 * delayed;
        mean += line[i];
    }
    mean /= delay_len as f64;
    for item in line.iter_mut().take(delay_len) {
        *item -= mean;
    }

    // Loop loss & damping parameters
    let (feedback, damp) = match pluck_type {
        PluckType::NylonGuitar => (0.9968, 0.30 - 0.10 * vel),
        PluckType::SteelGuitar => (0.9978, 0.16 - 0.07 * vel),
        PluckType::ConcertHarp => (0.9982, 0.34 - 0.10 * vel),
        PluckType::UprightBass => (0.9958, 0.42 - 0.12 * vel),
    };

    // Coupled wooden body resonators (Helmholtz air mode + top plate + back plate)
    let (f_air, f_top, f_back) = match pluck_type {
        PluckType::NylonGuitar => (98.0, 205.0, 410.0),
        PluckType::SteelGuitar => (104.0, 225.0, 460.0),
        PluckType::ConcertHarp => (130.0, 280.0, 560.0),
        PluckType::UprightBass => (64.0, 138.0, 275.0),
    };
    let mut body_air = ModalBiquad::bandpass(f_air, 3.2, sr);
    let mut body_top = ModalBiquad::bandpass(f_top, 2.8, sr);
    let mut body_back = ModalBiquad::bandpass(f_back, 2.4, sr);

    let mut ptr = 0_usize;
    let mut prev_out = 0.0_f64;
    let mut ap_x1 = 0.0_f64;
    let mut ap_y1 = 0.0_f64;
    let mut phase1 = 0.0_f64;
    let mut phase2 = 0.0_f64;
    let w1 = TAU * f0 / sr;
    let w2 = TAU * f0 * 2.0006 / sr;

    let body_tau = match pluck_type {
        PluckType::ConcertHarp => 1.65,
        PluckType::NylonGuitar | PluckType::SteelGuitar => 1.25,
        PluckType::UprightBass => 0.85,
    };

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let raw = line[ptr];
        // One-pole bridge low-pass damping
        let lp = (1.0 - damp) * raw + damp * prev_out;
        prev_out = raw;
        // Fractional-delay all-pass filter
        let ap = ap_coeff * lp + ap_x1 - ap_coeff * ap_y1;
        ap_x1 = lp;
        ap_y1 = ap;

        line[ptr] = ap * feedback;
        ptr = (ptr + 1) % delay_len;

        // Warm fundamental & octave body coupling
        phase1 += w1;
        phase2 += w2;
        let warm_fund = (phase1.sin() * 0.34 + phase2.sin() * 0.14) * (-t / body_tau).exp();

        let signal = ap + warm_fund;
        let b_res = body_air.process(signal) * 0.28
            + body_top.process(signal) * 0.24
            + body_back.process(signal) * 0.16;

        *out_sample = (signal * 0.72 + b_res) as f32;
    }

    buf
}

#[derive(Clone, Copy)]
enum BowedType {
    Violin,
    Cello,
    Quartet,
}

/// Helmholtz stick-slip bowed string model filtered through fixed Cremonese wooden body formants.
fn synth_bowed_string(
    f0: f64,
    vel: f64,
    note_samples: usize,
    total_samples: usize,
    sr: f64,
    seed: u64,
    bowed_type: BowedType,
) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];

    // Fixed wooden body formants (Hz, Q, gain): as vibrato moves harmonic frequencies across
    // these fixed bridge resonances, each partial modulates in amplitude like a real violin/cello!
    let formants: [(f64, f64, f64); 4] = match bowed_type {
        BowedType::Violin => [
            (285.0, 3.4, 0.38),  // A0 air cavity resonance
            (530.0, 3.0, 0.42),  // B1+ main spruce top plate mode
            (1180.0, 2.5, 0.28), // Bridge transition mode
            (2550.0, 2.8, 0.46), // Cremonese "Bridge Hill" brilliance formant
        ],
        BowedType::Cello => [
            (132.0, 3.2, 0.45),  // Cello air cavity
            (265.0, 2.8, 0.44),  // Main chest wood mode
            (610.0, 2.6, 0.36),  // Warm baritone formant
            (1380.0, 2.8, 0.32), // Upper bridge singing formant
        ],
        BowedType::Quartet => [
            (180.0, 2.2, 0.38),
            (420.0, 2.2, 0.40),
            (1050.0, 2.0, 0.34),
            (2250.0, 2.2, 0.34),
        ],
    };

    let mut f1 = ModalBiquad::bandpass(formants[0].0, formants[0].1, sr);
    let mut f2 = ModalBiquad::bandpass(formants[1].0, formants[1].1, sr);
    let mut f3 = ModalBiquad::bandpass(formants[2].0, formants[2].1, sr);
    let mut f4 = ModalBiquad::bandpass(formants[3].0, formants[3].1, sr);
    let mut warmth_lp = ModalBiquad::lowpass(
        match bowed_type {
            BowedType::Violin => 4400.0,
            BowedType::Cello => 2800.0,
            BowedType::Quartet => 3600.0,
        },
        0.75,
        sr,
    );
    let mut rosin_bp = ModalBiquad::bandpass((f0 * 3.0).clamp(600.0, 3200.0), 2.5, sr);

    // Number of bowed string players (1 for solo violin/cello with subtle room double, 4 for quartet)
    let voices: &[(f64, f64, f64)] = match bowed_type {
        // (detune_cents, vibrato_rate_hz, vibrato_depth_semitones)
        BowedType::Violin => &[(-1.2, 5.6, 0.14), (1.2, 5.8, 0.12)],
        BowedType::Cello => &[(-1.0, 5.1, 0.13), (1.0, 5.3, 0.11)],
        BowedType::Quartet => &[
            (-4.5, 5.3, 0.13),
            (-1.5, 5.7, 0.14),
            (1.8, 5.1, 0.12),
            (4.8, 5.5, 0.13),
        ],
    };

    let mut phases = [0.0_f64, 0.23, 0.51, 0.77];
    let attack_sec = match bowed_type {
        BowedType::Violin => 0.065,
        BowedType::Cello => 0.080,
        BowedType::Quartet => 0.110,
    };
    let note_dur_sec = (note_samples as f64 / sr).max(0.08);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;

        // Expressive bow pressure envelope: smooth bow attack + gentle mid-note swell
        let attack = if t < attack_sec {
            let u = t / attack_sec;
            u * u * (3.0 - 2.0 * u)
        } else {
            1.0
        };
        let bow_swell = 0.88 + 0.12 * (PI * (t / note_dur_sec).clamp(0.0, 1.0)).sin();
        // Delayed human vibrato onset (blooms smoothly after 160 ms)
        let vib_onset = ((t - 0.14) / 0.22).clamp(0.0, 1.0);

        let mut raw_bow = 0.0;
        for (v_idx, &(det_c, vib_rate, vib_depth)) in voices.iter().enumerate() {
            let vib_semi = vib_onset * vib_depth * (TAU * vib_rate * t + v_idx as f64 * 1.1).sin();
            let inst_hz = f0 * 2.0_f64.powf((det_c / 100.0 + vib_semi) / 12.0);
            phases[v_idx] = (phases[v_idx] + inst_hz / sr).fract();
            let p = phases[v_idx];

            // Helmholtz stick-slip waveform: flyback phase (~12% of period) vs sticking phase (~88%)
            // Produces rich bowed string harmonics without harsh digital sawtooth buzz
            let flyback = 0.12 + 0.04 * (1.0 - vel);
            let helmholtz = if p < 1.0 - flyback {
                2.0 * (p / (1.0 - flyback)) - 1.0
            } else {
                1.0 - 2.0 * ((p - (1.0 - flyback)) / flyback)
            };
            // Reinforce warm fundamental and 2nd harmonic
            let s = helmholtz * 0.62 + (TAU * p).sin() * 0.28 + (2.0 * TAU * p).sin() * 0.10;
            raw_bow += s;
        }
        raw_bow /= voices.len() as f64;

        // Subtle bow-hair rosin friction at bow attack (first 70 ms)
        let rosin = if t < 0.075 {
            rosin_bp.process(rng.next_f64()) * (-t * 42.0).exp() * 0.14 * vel
        } else {
            0.0
        };

        let excited = raw_bow + rosin;
        // Pass through fixed Cremonese wooden body formants + warm low-pass
        let body = f1.process(excited) * formants[0].2
            + f2.process(excited) * formants[1].2
            + f3.process(excited) * formants[2].2
            + f4.process(excited) * formants[3].2;
        let warm = warmth_lp.process(excited * 0.42 + body * 0.85);

        *out_sample = (warm * attack * bow_swell) as f32;
    }

    buf
}

#[derive(Clone, Copy)]
enum BeamType {
    Celesta,
    Rhodes,
    Wurlitzer,
    Vibraphone,
    Marimba,
}

/// Euler-Bernoulli tuned bar, tine & reed modal synthesis.
fn synth_mallet_or_tine(
    f0: f64,
    vel: f64,
    total_samples: usize,
    sr: f64,
    seed: u64,
    beam_type: BeamType,
) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];

    // Modal frequency ratios (f_n / f_0), relative amplitudes, and decay rates (1/tau)
    // Arch-carved vibraphone/marimba bars are tuned to 1 : 4 : 10; Rhodes tines have 1 : 2 : 6.27
    let modes: &[(f64, f64, f64)] = match beam_type {
        BeamType::Vibraphone => &[
            (1.0, 1.00, 0.65),
            (4.0, 0.36 * vel, 2.4),
            (10.0, 0.12 * vel, 11.0),
        ],
        BeamType::Marimba => &[
            (1.0, 1.00, 3.2),
            (4.0, 0.48 * vel, 9.5),
            (9.2, 0.22 * vel, 26.0),
            (2.0, 0.14, 5.5),
        ],
        BeamType::Celesta => &[
            (1.0, 1.00, 1.25),
            (2.0, 0.32, 2.8),
            (4.0, 0.28 * vel, 5.2),
            (6.75, 0.14 * vel, 14.0),
        ],
        BeamType::Rhodes => &[
            (1.0, 1.00, 0.72),
            (2.0, 0.42 * vel, 2.1),
            (3.0, 0.18 * vel * vel, 4.5),
            (6.27, 0.24 * vel, 9.0),  // Classic Fender Rhodes tine chime
            (11.4, 0.08 * vel, 22.0), // Hammer strike transient
        ],
        BeamType::Wurlitzer => &[
            (1.0, 1.00, 0.90),
            (2.0, 0.55 * vel, 2.2),
            (3.0, 0.38 * vel, 3.8),
            (4.0, 0.22 * vel * vel, 6.5),
            (5.85, 0.16 * vel, 12.0),
        ],
    };

    let mut mallet_bp = ModalBiquad::bandpass((f0 * 2.5).clamp(300.0, 3500.0), 2.0, sr);
    let nyquist = sr * 0.44;

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.0025).clamp(0.0, 1.0);

        let mut sum = 0.0;
        for &(ratio, amp, decay_rate) in modes {
            let fh = f0 * ratio;
            if fh < nyquist {
                sum += (TAU * fh * t).sin() * amp * (-t * decay_rate).exp();
            }
        }

        // Motor resonator tremolo on Vibraphone; subtle stereo chorus warmth on Rhodes
        let tremolo = match beam_type {
            BeamType::Vibraphone => 0.84 + 0.16 * (TAU * 5.2 * t).sin(),
            BeamType::Rhodes => 0.92 + 0.08 * (TAU * 3.6 * t).sin(),
            _ => 1.0,
        };

        // Soft tube saturation for Wurlitzer / Rhodes bark on high velocity
        let shaped = match beam_type {
            BeamType::Wurlitzer => (sum * (1.1 + 0.7 * vel)).tanh(),
            BeamType::Rhodes => (sum * (1.0 + 0.35 * vel)).tanh(),
            _ => sum,
        };

        let mallet_click = if t < 0.015 {
            mallet_bp.process(rng.next_f64()) * (-t * 260.0).exp() * 0.16 * vel
        } else {
            0.0
        };

        *out_sample = ((shaped * tremolo * attack) + mallet_click) as f32;
    }

    buf
}

#[derive(Clone, Copy)]
enum WindType {
    Flute,
    Clarinet,
}

/// Physical bore woodwind model (cylindrical silver flute & odd-harmonic grenadilla clarinet).
fn synth_woodwind(
    f0: f64,
    vel: f64,
    note_samples: usize,
    total_samples: usize,
    sr: f64,
    seed: u64,
    wind_type: WindType,
) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    let mut chiff_bp = ModalBiquad::bandpass((f0 * 2.0).clamp(500.0, 3800.0), 3.2, sr);
    let mut bore_lp = ModalBiquad::lowpass(
        match wind_type {
            WindType::Flute => 4200.0,
            WindType::Clarinet => 2900.0,
        },
        0.8,
        sr,
    );

    let mut phase = 0.0_f64;
    let note_dur_sec = (note_samples as f64 / sr).max(0.08);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.045).clamp(0.0, 1.0);
        let breath_swell = 0.90 + 0.10 * (PI * (t / note_dur_sec).clamp(0.0, 1.0)).sin();
        let vib_onset = ((t - 0.16) / 0.20).clamp(0.0, 1.0);
        let vib_semi = vib_onset * 0.11 * (TAU * 5.4 * t).sin();
        let hz = f0 * 2.0_f64.powf(vib_semi / 12.0);
        phase = (phase + hz / sr).fract();
        let w = TAU * phase;

        let tone = match wind_type {
            // Silver flute: pure fundamental + singing 2nd/3rd overblown tube harmonics
            WindType::Flute => {
                w.sin() * 0.72 + (2.0 * w).sin() * (0.24 * vel) + (3.0 * w).sin() * 0.08
            }
            // Cylindrical bore clarinet: predominantly odd harmonics (1, 3, 5, 7)
            WindType::Clarinet => {
                w.sin() * 0.62
                    + (3.0 * w).sin() * (0.30 * vel)
                    + (5.0 * w).sin() * (0.15 * vel)
                    + (2.0 * w).sin() * 0.06
                    + (7.0 * w).sin() * 0.06
            }
        };

        // Embouchure breath chiff transient + subtle continuous air
        let chiff_env = (-t * 36.0).exp() * 0.14 + 0.022;
        let breath = chiff_bp.process(rng.next_f64()) * chiff_env * vel;
        let filtered = bore_lp.process(tone + breath);

        *out_sample = (filtered * attack * breath_swell) as f32;
    }

    buf
}

/// Studio wire-brush snare sweep on a coated acoustic drumhead.
fn synth_brush_snare(vel: f64, total_samples: usize, sr: f64, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    let mut wire_bp = ModalBiquad::bandpass(2650.0, 1.4, sr);
    let mut shell_bp = ModalBiquad::bandpass(195.0, 3.0, sr);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.008).clamp(0.0, 1.0);
        let wire_env = (-t * 11.5).exp() * attack;
        let shell_env = (-t * 28.0).exp();
        let n = rng.next_f64();
        let wires = wire_bp.process(n) * wire_env * 0.78;
        let shell = (shell_bp.process(n) + (TAU * 188.0 * t).sin() * 0.35) * shell_env * 0.32;
        *out_sample = ((wires + shell) * vel) as f32;
    }
    buf
}
