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
    Jp8000Supersaw,
    ReeseNeuroBass,
    Tr909ClubKick,
    Tr909ClubSnare,
    Tr909ClubClap,
    VocalFormantChop,
    DaftTalkboxLead,
    DevilFish303Acid,
    AlphaJunoHoover,
    M1HousePiano,
    M1OrganBass,
    InsomniaRavePizz,
    UkBasslineDonk,
    CzPhaseLead,
    WtModernPluck,
    PadSynthLushChoir,
    PadSynthEtherealStrings,
    BalkanZurnaLead,
    CyberShamisenPluck,
    CelticWhistleLead,
    ArabicOudPluck,
    EurobeatSynthBrass,
    EurobeatOctaveBass,
    AnimeRockGuitar,
    EurobeatCowbell,
    ClubCrashCymbal,
    AnimeSparkleArp,
    UprisingTranceLead,
}

impl PhysicalModelKind {
    /// Detects whether an instrument should be rendered by a physical or modern electronic model.
    pub(crate) fn from_instrument(instrument: &Instrument) -> Option<Self> {
        let name = instrument.name.trim().to_ascii_lowercase();
        // Match specific modern electronic, Eurobeat & world models before broad fallbacks.
        if name.contains("eurobeat brass")
            || name.contains("initial d brass")
            || name.contains("eurobeat synth brass")
        {
            return Some(Self::EurobeatSynthBrass);
        }
        if name.contains("eurobeat octave bass") || name.contains("uprising techno bass") {
            return Some(Self::EurobeatOctaveBass);
        }
        if name.contains("anime rock guitar") || name.contains("touge power guitar") {
            return Some(Self::AnimeRockGuitar);
        }
        if name.contains("eurobeat cowbell") {
            return Some(Self::EurobeatCowbell);
        }
        if name.contains("club crash") {
            return Some(Self::ClubCrashCymbal);
        }
        if name.contains("anime sparkle") {
            return Some(Self::AnimeSparkleArp);
        }
        if name.contains("uprising trance") || name.contains("anime sabi") {
            return Some(Self::UprisingTranceLead);
        }
        if name.contains("vocal formant") || name.contains("vocal chop") {
            return Some(Self::VocalFormantChop);
        }
        if name.contains("talkbox") || name.contains("daft talkbox") {
            return Some(Self::DaftTalkboxLead);
        }
        if name.contains("devilfish") || name.contains("303 acid") || name.contains("tb-303") {
            return Some(Self::DevilFish303Acid);
        }
        if name.contains("hoover") || name.contains("alpha juno") || name.contains("mentasm") {
            return Some(Self::AlphaJunoHoover);
        }
        if name.contains("m1 house piano") || name.contains("house piano") {
            return Some(Self::M1HousePiano);
        }
        if name.contains("m1 organ") || name.contains("organ 2 bass") || name.contains("organ bass")
        {
            return Some(Self::M1OrganBass);
        }
        if name.contains("insomnia") || name.contains("rave pizz") || name.contains("maxi pizz") {
            return Some(Self::InsomniaRavePizz);
        }
        if name.contains("bassline donk") || name.contains("uk donk") || name.contains("fm donk")
            || name.contains("offbeat donk")
        {
            return Some(Self::UkBasslineDonk);
        }
        if name.contains("cz phase") || name.contains("phase distortion") || name.contains("phase clav")
        {
            return Some(Self::CzPhaseLead);
        }
        if name.contains("wavetable pluck")
            || name.contains("wt modern pluck")
            || name.contains("serum pluck")
        {
            return Some(Self::WtModernPluck);
        }
        if name.contains("padsynth") && name.contains("choir") {
            return Some(Self::PadSynthLushChoir);
        }
        if name.contains("padsynth") || name.contains("ethereal strings") {
            return Some(Self::PadSynthEtherealStrings);
        }
        if name.contains("zurna") || name.contains("balkan") {
            return Some(Self::BalkanZurnaLead);
        }
        if name.contains("shamisen") {
            return Some(Self::CyberShamisenPluck);
        }
        if name.contains("celtic whistle") || name.contains("tin whistle") {
            return Some(Self::CelticWhistleLead);
        }
        if name.contains("arabic oud") || name.contains("oud pluck") {
            return Some(Self::ArabicOudPluck);
        }
        if name.contains("supersaw")
            || name.contains("aloboi warm lead")
            || name.contains("cs-80 brass")
            || name.contains("aloboi brass swell")
        {
            return Some(Self::Jp8000Supersaw);
        }
        if name.contains("aloboi gliding bass")
            || name.contains("cyber reese")
            || name.contains("reese")
        {
            return Some(Self::ReeseNeuroBass);
        }
        if name.contains("punch studio kick")
            || name == "fusion studio kick"
            || name.contains("909 club kick")
            || name.contains("tr-909 kick")
        {
            return Some(Self::Tr909ClubKick);
        }
        if name.contains("crisp snare clap")
            || name == "fusion studio snare"
            || name.contains("909 club snare")
            || name.contains("tr-909 snare")
        {
            return Some(Self::Tr909ClubSnare);
        }
        if name == "hand clap"
            || name == "clap"
            || name.contains("909 club clap")
            || name.contains("tr-909 clap")
        {
            return Some(Self::Tr909ClubClap);
        }
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
                | Self::Tr909ClubKick
                | Self::Tr909ClubSnare
                | Self::Tr909ClubClap
                | Self::M1HousePiano
                | Self::InsomniaRavePizz
                | Self::UkBasslineDonk
                | Self::WtModernPluck
                | Self::CyberShamisenPluck
                | Self::ArabicOudPluck
                | Self::EurobeatCowbell
                | Self::ClubCrashCymbal
                | Self::AnimeSparkleArp
        )
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

/// Polynomial band-limited step correction to eliminate digital aliasing in saw/pulse oscillators.
#[inline]
fn poly_blep(t: f64, dt: f64) -> f64 {
    if dt <= 0.0 || dt >= 0.5 {
        return 0.0;
    }
    if t < dt {
        let x = t / dt;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}

#[inline]
fn blep_saw(phase: f64, dt: f64) -> f64 {
    (2.0 * phase - 1.0) - poly_blep(phase, dt)
}

#[inline]
fn blep_pulse(phase: f64, dt: f64, pw: f64) -> f64 {
    let raw = if phase < pw { 1.0 } else { -1.0 };
    raw + poly_blep(phase, dt) - poly_blep((phase - pw).rem_euclid(1.0), dt)
}

/// Zero-delay-feedback 4-pole (24 dB/oct) TPT resonant lowpass filter with gentle saturation.
struct TptLadder {
    z: [f64; 4],
}

impl TptLadder {
    const fn new() -> Self {
        Self { z: [0.0; 4] }
    }

    fn process(&mut self, input: f64, cutoff_hz: f64, res: f64, sr: f64) -> f64 {
        let fc = cutoff_hz.clamp(35.0, sr * 0.24);
        let g = (PI * fc / sr).tan();
        let g1 = g / (1.0 + g);
        let k = res.clamp(0.0, 2.8);
        let fb = (k * self.z[3]).tanh();
        let mut x = (input - fb).tanh();
        for state in &mut self.z {
            let v = (x - *state) * g1;
            let y = v + *state;
            *state = y + v;
            x = y;
        }
        x
    }
}

/// One-pole mud-removing high-pass filter.
struct OnePoleHp {
    x1: f64,
    y1: f64,
    a: f64,
}

impl OnePoleHp {
    fn new(cutoff_hz: f64, sr: f64) -> Self {
        let w = (TAU * cutoff_hz / sr).clamp(0.001, 0.5);
        Self {
            x1: 0.0,
            y1: 0.0,
            a: (1.0 - w).clamp(0.85, 0.998),
        }
    }

    #[inline]
    fn process(&mut self, x: f64) -> f64 {
        let y = self.a * (self.y1 + x - self.x1);
        self.x1 = x;
        self.y1 = y;
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
        PhysicalModelKind::Jp8000Supersaw => synth_jp8000_supersaw(hz, vel, total_samples, sr),
        PhysicalModelKind::ReeseNeuroBass => synth_reese_neuro_bass(hz, vel, total_samples, sr),
        PhysicalModelKind::Tr909ClubKick => synth_tr909_kick(vel, total_samples, sr),
        PhysicalModelKind::Tr909ClubSnare => synth_tr909_snare(vel, total_samples, sr, seed),
        PhysicalModelKind::Tr909ClubClap => synth_tr909_clap(vel, total_samples, sr, seed),
        PhysicalModelKind::VocalFormantChop => {
            synth_vocal_formant(hz, vel, pitch, total_samples, sr, false)
        }
        PhysicalModelKind::DaftTalkboxLead => {
            synth_vocal_formant(hz, vel, pitch, total_samples, sr, true)
        }
        PhysicalModelKind::DevilFish303Acid => synth_devilfish_303(hz, vel, total_samples, sr),
        PhysicalModelKind::AlphaJunoHoover => synth_alpha_juno_hoover(hz, vel, total_samples, sr),
        PhysicalModelKind::M1HousePiano => synth_m1_house_piano(hz, vel, total_samples, sr, seed),
        PhysicalModelKind::M1OrganBass => synth_m1_organ_bass(hz, vel, total_samples, sr),
        PhysicalModelKind::InsomniaRavePizz => {
            synth_insomnia_rave_pizz(hz, vel, total_samples, sr, seed)
        }
        PhysicalModelKind::UkBasslineDonk => synth_uk_bassline_donk(hz, vel, total_samples, sr),
        PhysicalModelKind::CzPhaseLead => synth_cz_phase_lead(hz, vel, total_samples, sr),
        PhysicalModelKind::WtModernPluck => synth_wt_modern_pluck(hz, vel, total_samples, sr, seed),
        PhysicalModelKind::PadSynthLushChoir => {
            synth_padsynth_ensemble(hz, vel, total_samples, sr, seed, true)
        }
        PhysicalModelKind::PadSynthEtherealStrings => {
            synth_padsynth_ensemble(hz, vel, total_samples, sr, seed, false)
        }
        PhysicalModelKind::BalkanZurnaLead => synth_balkan_zurna(hz, vel, total_samples, sr, seed),
        PhysicalModelKind::CyberShamisenPluck => synth_cyber_shamisen(hz, vel, total_samples, sr, seed),
        PhysicalModelKind::CelticWhistleLead => synth_celtic_whistle(hz, vel, total_samples, sr, seed),
        PhysicalModelKind::ArabicOudPluck => synth_arabic_oud(hz, vel, total_samples, sr, seed),
        PhysicalModelKind::EurobeatSynthBrass => synth_eurobeat_brass(hz, vel, total_samples, sr),
        PhysicalModelKind::EurobeatOctaveBass => synth_eurobeat_octave_bass(hz, vel, total_samples, sr),
        PhysicalModelKind::AnimeRockGuitar => synth_anime_rock_guitar(hz, vel, total_samples, sr, seed),
        PhysicalModelKind::EurobeatCowbell => synth_eurobeat_cowbell(vel, total_samples, sr),
        PhysicalModelKind::ClubCrashCymbal => synth_club_crash_cymbal(vel, total_samples, sr, seed),
        PhysicalModelKind::AnimeSparkleArp => synth_anime_sparkle_arp(hz, vel, total_samples, sr),
        PhysicalModelKind::UprisingTranceLead => synth_uprising_trance_lead(hz, vel, total_samples, sr),
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

/// Roland JP-8000 7-Oscillator PolyBLEP Detuned Supersaw with 24dB TPT ladder sweep.
fn synth_jp8000_supersaw(f0: f64, vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    const DETUNES: [f64; 7] = [-0.115, -0.065, -0.022, 0.0, 0.022, 0.066, 0.116];
    const WEIGHTS: [f64; 7] = [0.11, 0.14, 0.17, 0.24, 0.17, 0.14, 0.11];
    let mut phases: [f64; 7] = [0.0, 0.142, 0.285, 0.428, 0.571, 0.714, 0.857];
    let freqs: [f64; 7] = DETUNES.map(|d| f0 * 2.0_f64.powf(d / 12.0));

    let mut hp = OnePoleHp::new(150.0, sr);
    let mut ladder = TptLadder::new();
    let mut pres = ModalBiquad::bandpass(2400.0, 0.9, sr);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.0045).clamp(0.0, 1.0);
        let pitch_punch = 1.0 + 0.005 * (-t * 135.0).exp();

        let mut sum = 0.0;
        for k in 0..7 {
            let dt = (freqs[k] * pitch_punch) / sr;
            phases[k] = (phases[k] + dt).fract();
            sum += blep_saw(phases[k], dt) * WEIGHTS[k];
        }
        let cutoff = 1200.0 + (7800.0 * vel) * (0.45 + 0.55 * (-t * 7.5).exp());
        let cleaned = hp.process(sum);
        let filtered = ladder.process(cleaned + pres.process(cleaned) * 0.18, cutoff, 0.45, sr);
        *out_sample = ((filtered * 1.18).tanh() * attack) as f32;
    }
    buf
}

/// 5-Voice PolyBLEP Detuned Reese / Neuro Bass with phase-locked sub & 24dB TPT growl.
fn synth_reese_neuro_bass(f0: f64, vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    const DETUNES: [f64; 4] = [-0.14, -0.05, 0.05, 0.15];
    let mut phases = [0.0, 0.25, 0.50, 0.75];
    let mut sub_phase = 0.0_f64;
    let freqs = DETUNES.map(|d| f0 * 2.0_f64.powf(d / 12.0));

    let mut ladder = TptLadder::new();
    let mut mid_bite = ModalBiquad::bandpass((f0 * 3.5).clamp(220.0, 1400.0), 1.9, sr);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.004).clamp(0.0, 1.0);
        sub_phase = (sub_phase + f0 / sr).fract();
        let sub = (TAU * sub_phase).sin() * 0.54;

        let mut saws = 0.0;
        for k in 0..4 {
            let dt = freqs[k] / sr;
            phases[k] = (phases[k] + dt).fract();
            saws += blep_saw(phases[k], dt) * 0.25;
        }
        let cutoff = 320.0 + 1350.0 * vel * (0.35 + 0.65 * (-t * 5.0).exp());
        let growl = ladder.process(saws, cutoff, 0.75, sr)
            + mid_bite.process(saws) * (0.28 * vel) * (-t * 4.5).exp();
        *out_sample = (((sub + growl * 0.78) * 1.25).tanh() * attack) as f32;
    }
    buf
}

/// Authentic TR-909 / Festival Club Kick: two-stage pitch drop + transient click + saturated sub body.
fn synth_tr909_kick(vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    let mut phase = 0.0_f64;
    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        // Two-stage 909 pitch envelope: 170 Hz -> 68 Hz (fast 15 ms punch) -> 46 Hz (sub body)
        let f_inst = 46.0 + 115.0 * (-t * 115.0).exp() + 28.0 * (-t * 22.0).exp();
        phase = (phase + f_inst / sr).fract();
        let body_env = (-t * 8.5).exp();
        let sub_body = (TAU * phase).sin() * body_env;
        // Transient beater click at 1.5 kHz in first 4 ms
        let click = (TAU * 1480.0 * t).sin() * (-t * 420.0).exp() * 0.36 * vel;
        *out_sample = ((sub_body * 1.35 + click).tanh()) as f32;
    }
    buf
}

/// Studio TR-909 / Club Snare: dual-mode drumhead body (185 Hz + 330 Hz) + crispy high-passed wires.
fn synth_tr909_snare(vel: f64, total_samples: usize, sr: f64, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    let mut wire_bp = ModalBiquad::bandpass(3400.0, 1.1, sr);
    let mut snap_bp = ModalBiquad::bandpass(1350.0, 1.6, sr);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let f1 = 185.0 + 95.0 * (-t * 140.0).exp();
        let f2 = 330.0 + 120.0 * (-t * 140.0).exp();
        let body = ((TAU * f1 * t).sin() * 0.62 + (TAU * f2 * t).sin() * 0.38) * (-t * 26.0).exp();
        let n = rng.next_f64();
        let wires = (wire_bp.process(n) * 0.72 + snap_bp.process(n) * 0.45) * (-t * 15.5).exp();
        *out_sample = (((body * 0.65 + wires * 0.85) * (0.7 + 0.4 * vel)).tanh()) as f32;
    }
    buf
}

/// Classic TR-909 4-Burst Micro-Staggered Handclap (0 ms, 11 ms, 22 ms, 33 ms) through 1180 Hz bandpass.
fn synth_tr909_clap(vel: f64, total_samples: usize, sr: f64, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    let mut clap_bp = ModalBiquad::bandpass(1180.0, 1.85, sr);
    let mut air_bp = ModalBiquad::bandpass(2850.0, 1.4, sr);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        // 3 micro-bursts separated by 11 ms followed by a decaying reverb tail at 33 ms
        let env = if t < 0.011 {
            (-t * 190.0).exp()
        } else if t < 0.022 {
            (-(t - 0.011) * 190.0).exp() * 0.92
        } else if t < 0.033 {
            (-(t - 0.022) * 190.0).exp() * 0.88
        } else {
            (-(t - 0.033) * 20.0).exp() * 0.82
        };
        let n = rng.next_f64();
        let shaped = clap_bp.process(n) * 0.75 + air_bp.process(n) * 0.45;
        *out_sample = ((shaped * env * (0.8 + 0.4 * vel)).tanh()) as f32;
    }
    buf
}

/// Smooth PolyBLEP 3-band morphing vocal formant & Daft talkbox synthesis.
fn synth_vocal_formant(
    f0: f64,
    vel: f64,
    pitch: u8,
    total_samples: usize,
    sr: f64,
    talkbox: bool,
) -> Vec<f32> {
    const VOWELS: [(f64, f64, f64); 5] = [
        (290.0, 2250.0, 2950.0),
        (520.0, 1820.0, 2480.0),
        (720.0, 1120.0, 2420.0),
        (560.0, 860.0, 2380.0),
        (320.0, 880.0, 2240.0),
    ];
    let mut buf = vec![0.0_f32; total_samples];
    let v_start = if talkbox {
        VOWELS[3]
    } else {
        VOWELS[usize::from(pitch) % VOWELS.len()]
    };
    let v_end = if talkbox {
        VOWELS[1]
    } else {
        VOWELS[(usize::from(pitch) + 2) % VOWELS.len()]
    };
    let dur_sec = (total_samples as f64 / sr).max(0.06);
    let mut b1 = ModalBiquad::bandpass(v_start.0, 3.8, sr);
    let mut b2 = ModalBiquad::bandpass(v_start.1, 4.6, sr);
    let mut b3 = ModalBiquad::bandpass(v_start.2, 5.2, sr);
    let mut ladder = TptLadder::new();
    let mut phase = 0.0_f64;

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let u = (t / dur_sec).clamp(0.0, 1.0);
        let morph = if talkbox {
            (u * 1.4).sin().abs().clamp(0.0, 1.0)
        } else {
            u
        };
        let f1 = v_start.0 + (v_end.0 - v_start.0) * morph;
        let f2 = v_start.1 + (v_end.1 - v_start.1) * morph;
        let f3 = v_start.2 + (v_end.2 - v_start.2) * morph;
        for (b, f, q) in [(&mut b1, f1, 3.8), (&mut b2, f2, 4.6), (&mut b3, f3, 5.2)] {
            let c = ModalBiquad::bandpass(f, q, sr);
            b.b0 = c.b0;
            b.b1 = c.b1;
            b.b2 = c.b2;
            b.a1 = c.a1;
            b.a2 = c.a2;
        }
        let scoop = if talkbox {
            0.10 * ((t - 0.08).max(0.0) * TAU * 5.6).sin()
        } else {
            -1.0 * (-t * 45.0).exp() + 0.08 * ((t - 0.12).max(0.0) * TAU * 5.8).sin()
        };
        let hz = (f0 * 2.0_f64.powf(scoop / 12.0)).clamp(20.0, sr * 0.4);
        let dt = hz / sr;
        phase = (phase + dt).fract();
        let saw = blep_saw(phase, dt);
        let pulse = blep_pulse(phase, dt, 0.38);
        let exc = if talkbox {
            (saw * 0.72 + (TAU * phase).sin() * 0.45) * (1.15 + 0.5 * vel)
        } else {
            saw * 0.60 + pulse * 0.40
        }
        .tanh();
        let formants =
            b1.process(exc) * 0.95 + b2.process(exc) * 0.78 + b3.process(exc) * 0.42;
        let warm_fund = (TAU * phase).sin() * 0.34;
        let attack = (t / 0.008).clamp(0.0, 1.0);
        let out = ladder.process(formants + warm_fund, (f0 * 8.0 + 2600.0).min(sr * 0.23), 0.3, sr);
        *out_sample = ((out * 1.2).tanh() * attack) as f32;
    }
    buf
}

/// Open303 / DevilFish TB-303 diode-ladder acid bassline with 2x oversampling & velocity accent.
fn synth_devilfish_303(f0: f64, vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    let sr2 = sr * 2.0;
    let accent = ((vel - 0.72) / 0.28).clamp(0.0, 1.0);
    let decay_tau = 0.22 - 0.10 * accent;
    let mut phase = 0.0_f64;
    let (mut z1, mut z2, mut z3, mut z4, mut s4) = (0.0_f64, 0.0, 0.0, 0.0, 0.0);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let mut acc = 0.0_f64;
        for sub in 0..2 {
            let t = (i as f64 + 0.5 * sub as f64) / sr;
            phase = (phase + f0 / sr2).fract();
            let saw = 2.0 * phase - 1.0;
            let square = if phase < 0.5 { 1.0 } else { -1.0 };
            let osc = saw * (1.0 - 0.25 * accent) + square * (0.25 * accent);

            let env = (-t / decay_tau).exp();
            let cutoff = (170.0 + (1850.0 + 3400.0 * accent) * env + 480.0 * vel)
                .clamp(85.0, sr2 * 0.21);
            let g = (PI * cutoff / sr2).tan().clamp(0.01, 0.72);
            let g_coeff = g / (1.0 + g);
            let k = (2.85 + 0.78 * accent).clamp(0.5, 3.62);

            let fb = (k * s4).tanh();
            let u = (osc * (1.15 + 0.45 * accent) - fb).tanh();
            let v1 = (u - z1) * g_coeff;
            let y1 = v1 + z1;
            z1 = y1 + v1;
            let v2 = (y1.tanh() - z2) * g_coeff;
            let y2 = v2 + z2;
            z2 = y2 + v2;
            let v3 = (y2 - z3) * g_coeff;
            let y3 = v3 + z3;
            z3 = y3 + v3;
            let v4 = (y3 - z4) * g_coeff;
            let y4 = v4 + z4;
            z4 = y4 + v4;
            s4 = y4;

            // Preserve bass weight under heavy resonance + asymmetric diode overdrive
            let sub_warmth = (TAU * phase).sin() * 0.26;
            acc += (y4 * 1.45 + sub_warmth) * 0.5;
        }
        let t = i as f64 / sr;
        let attack = (t / 0.0025).clamp(0.0, 1.0);
        let driven = (acc * (1.35 + 0.75 * accent)).tanh();
        *out_sample = (driven * attack) as f32;
    }
    buf
}

/// Roland Alpha Juno PolyBLEP "Hoover" / Mentasm: sub square + dual LFO PWM saws + swoop.
fn synth_alpha_juno_hoover(f0: f64, vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    let mut phases = [0.0_f64, 0.19, 0.47, 0.73];
    let ratios = [0.5, 0.9948, 1.0064, 2.008];
    let mut hp = OnePoleHp::new(135.0, sr);
    let mut ladder = TptLadder::new();

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let pitch_swoop = 2.1 * (-t * 18.0).exp();
        let lfo = (TAU * 6.2 * t).sin();
        let vib = 0.08 * ((t - 0.10).max(0.0) * TAU * 5.2).sin();
        let glide = 2.0_f64.powf((pitch_swoop + vib) / 12.0);
        let mut dts = [0.0_f64; 4];
        for (k, p) in phases.iter_mut().enumerate() {
            let dt = ((f0 * ratios[k] * glide) / sr).clamp(0.0, 0.45);
            dts[k] = dt;
            *p = (*p + dt).fract();
        }
        let pw = (0.5 + 0.34 * lfo).clamp(0.12, 0.88);
        let sub_sq = blep_pulse(phases[0], dts[0], 0.5);
        let pwm_saw_a = blep_saw(phases[1], dts[1]) - blep_pulse(phases[1], dts[1], pw) * 0.42;
        let pwm_saw_b = blep_saw(phases[2], dts[2]) + blep_pulse(phases[2], dts[2], 1.0 - pw) * 0.42;
        let hi_saw = blep_saw(phases[3], dts[3]);

        let mix = sub_sq * 0.32 + pwm_saw_a * 0.34 + pwm_saw_b * 0.34 + hi_saw * (0.20 + 0.10 * vel);
        let attack = (t / 0.008).clamp(0.0, 1.0);
        let filtered = ladder.process(hp.process(mix), (2800.0 + 3600.0 * vel).min(sr * 0.23), 0.7, sr);
        *out_sample = ((filtered * 1.15).tanh() * attack) as f32;
    }
    buf
}

/// 90s Korg M1 compressed bright house piano with stiff-string dispersion and club bite.
fn synth_m1_house_piano(f0: f64, vel: f64, total_samples: usize, sr: f64, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    let mut click_bp = ModalBiquad::bandpass(2350.0, 1.9, sr);
    let mut pres_bp = ModalBiquad::bandpass(2100.0, 1.1, sr);
    let nyquist = sr * 0.43;

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.0018).clamp(0.0, 1.0);
        let club_env = 0.28 * (-t * 1.8).exp() + 0.72 * (-t * 4.8).exp();
        let mut sum = 0.0;
        for h in 1..=18 {
            let n = h as f64;
            let fh = f0 * n * (1.0 + 0.00018 * n * n).sqrt();
            if fh >= nyquist {
                break;
            }
            let h_gain = match h {
                1 => 0.74,
                2 => 0.56,
                3 => 0.42,
                4 => 0.30 * (0.65 + 0.35 * vel),
                _ => (0.28 / (n - 2.0)) * (0.5 + 0.5 * vel),
            };
            let h_decay = (-t * (0.7 + 0.55 * n)).exp();
            sum += (TAU * fh * t).sin() * h_gain * h_decay;
        }
        let click = if t < 0.012 {
            click_bp.process(rng.next_f64()) * (-t * 320.0).exp() * 0.36 * vel
        } else {
            0.0
        };
        let tone = sum * club_env + pres_bp.process(sum) * 0.32 + click;
        *out_sample = ((tone * 1.42).tanh() * attack) as f32;
    }
    buf
}

/// Korg M1 "Organ 2" Club Bass: drawbar harmonics + sharp key-click transient.
fn synth_m1_organ_bass(f0: f64, vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    let mut punch_bp = ModalBiquad::bandpass(1150.0, 1.4, sr);
    let nyquist = sr * 0.43;

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.0022).clamp(0.0, 1.0);
        let env = 0.48 + 0.52 * (-t * 14.5).exp();
        let drawbars: [(f64, f64); 5] = [
            (1.0, 0.64),
            (2.0, 0.46),
            (3.0, 0.34),
            (4.0, 0.14 * vel),
            (6.0, 0.08 * vel),
        ];
        let mut tone = 0.0;
        for (mult, amp) in drawbars {
            let fh = f0 * mult;
            if fh < nyquist {
                tone += (TAU * fh * t).sin() * amp;
            }
        }
        let key_click = (TAU * (f0 * 5.0).min(nyquist) * t).sin() * (-t * 280.0).exp() * 0.36;
        let body = tone * env + punch_bp.process(tone) * 0.32 * (-t * 18.0).exp() + key_click;
        *out_sample = ((body * 1.32).tanh() * attack) as f32;
    }
    buf
}

/// Faithless "Insomnia" style detuned maxi-pizzicato rave stab.
fn synth_insomnia_rave_pizz(
    f0: f64,
    vel: f64,
    total_samples: usize,
    sr: f64,
    seed: u64,
) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    let mut pick_bp = ModalBiquad::bandpass(1850.0, 1.8, sr);
    const DETUNES: [f64; 3] = [-0.08, 0.0, 0.09];
    let nyquist = sr * 0.43;

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.0018).clamp(0.0, 1.0);
        let mut sum = 0.0;
        for &d in &DETUNES {
            let fd = f0 * 2.0_f64.powf(d / 12.0);
            for h in 1..=9 {
                let fh = fd * h as f64;
                if fh >= nyquist {
                    break;
                }
                let amp = (0.55 / (h as f64).powf(0.85)) * (-t * (5.8 + 2.2 * h as f64)).exp();
                sum += (TAU * fh * t).sin() * amp;
            }
        }
        let transient = if t < 0.014 {
            pick_bp.process(rng.next_f64()) * (-t * 240.0).exp() * 0.42 * vel
        } else {
            0.0
        };
        *out_sample = (((sum * 0.55 + transient) * 1.25).tanh() * attack) as f32;
    }
    buf
}

/// UK Bassline / Hardbounce 2-operator FM Donk bass.
fn synth_uk_bassline_donk(f0: f64, vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    let (mut p_car, mut p_mod) = (0.0_f64, 0.0_f64);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.002).clamp(0.0, 1.0);
        let pitch_bounce = 1.65 * (-t * 58.0).exp();
        let hz = (f0 * 2.0_f64.powf(pitch_bounce / 12.0)).clamp(20.0, sr * 0.35);
        p_car = (p_car + hz / sr).fract();
        p_mod = (p_mod + (hz * 3.0).min(sr * 0.42) / sr).fract();

        let fm_index = (2.6 + 1.9 * vel) * (-t * 25.0).exp();
        let fm_tone = (TAU * p_car + fm_index * (TAU * p_mod).sin()).sin();
        let sub_fund = (TAU * p_car).sin() * 0.55;
        let body_env = (-t * 5.2).exp();
        let mix = (fm_tone * 0.72 + sub_fund) * body_env;
        *out_sample = ((mix * 1.38).tanh() * attack) as f32;
    }
    buf
}

/// Casio CZ Phase Distortion resonant electro lead.
fn synth_cz_phase_lead(f0: f64, vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    let mut phase = 0.0_f64;
    let mut air_lp = ModalBiquad::lowpass((3800.0 + 3600.0 * vel).min(sr * 0.44), 1.1, sr);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.003).clamp(0.0, 1.0);
        phase = (phase + f0 / sr).fract();
        let env = 0.32 + 0.68 * (-t * 8.5).exp();
        let d = (0.06 + 0.40 * env * vel).clamp(0.05, 0.47);
        let warped = if phase < d {
            0.5 * phase / d
        } else {
            0.5 + 0.5 * (phase - d) / (1.0 - d)
        };
        let pd_wave = -(TAU * warped).cos();
        let res_window = (1.0 - phase) * (TAU * (2.5 + 3.5 * env * vel) * phase).sin() * 0.38;
        let filtered = air_lp.process(pd_wave * 0.82 + res_window);
        *out_sample = ((filtered * 1.15).tanh() * attack) as f32;
    }
    buf
}

/// Morphing digital wavetable pluck (bright odd partials morphing to warm bell fundamental).
fn synth_wt_modern_pluck(
    f0: f64,
    vel: f64,
    total_samples: usize,
    sr: f64,
    seed: u64,
) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    let mut click_bp = ModalBiquad::bandpass(3200.0, 1.6, sr);
    let mut phase = 0.0_f64;
    let nyquist = sr * 0.43;

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.002).clamp(0.0, 1.0);
        phase = (phase + f0 / sr).fract();
        let morph = (-t * 15.0).exp() * vel;
        let mut bright_tbl = 0.0;
        for k in 1..=6 {
            let mult = (2 * k - 1) as f64;
            if f0 * mult >= nyquist {
                break;
            }
            bright_tbl += (TAU * mult * phase).sin() / (k as f64 * 1.15);
        }
        let warm_tbl = (TAU * phase).sin() * 0.78
            + if f0 * 2.0 < nyquist {
                (TAU * 2.0 * phase).sin() * 0.32
            } else {
                0.0
            };
        let plucked = (bright_tbl * morph + warm_tbl * (1.0 - 0.55 * morph)) * (-t * 4.4).exp();
        let click = if t < 0.009 {
            click_bp.process(rng.next_f64()) * (-t * 360.0).exp() * 0.25 * vel
        } else {
            0.0
        };
        *out_sample = ((plucked + click).tanh() * attack) as f32;
    }
    buf
}

/// ZynAddSubFX PADsynth-inspired Gaussian harmonic bandwidth ensemble pad/choir.
fn synth_padsynth_ensemble(
    f0: f64,
    vel: f64,
    total_samples: usize,
    sr: f64,
    seed: u64,
    choir: bool,
) -> Vec<f32> {
    let mut rng = Rng::new(seed ^ if choir { 0x41 } else { 0x97 });
    let mut buf = vec![0.0_f32; total_samples];
    let nyquist = sr * 0.43;
    let harmonics = if choir { 10_usize } else { 12_usize };
    const OFFSETS: [f64; 5] = [-2.0, -1.0, 0.0, 1.0, 2.0];
    const GAUSS: [f64; 5] = [0.28, 0.72, 1.0, 0.72, 0.28];

    // Precompute deterministic ensemble oscillator bank (harmonic * 5 Gaussian sidebands)
    let mut partials: Vec<(f64, f64, f64)> = Vec::with_capacity(harmonics * 5);
    let bw_scale = if choir { 0.0044 } else { 0.0058 };
    for h in 1..=harmonics {
        let n = h as f64;
        let center = f0 * n;
        let bw = (center * bw_scale).max(0.45);
        let h_weight = (1.0 / n.powf(if choir { 0.92 } else { 0.85 }))
            * if choir && (2..=4).contains(&h) { 1.35 } else { 1.0 };
        for (idx, &off) in OFFSETS.iter().enumerate() {
            let fh = center + off * bw;
            if fh >= 20.0 && fh < nyquist {
                let p0 = rng.next_f64() * PI;
                partials.push((TAU * fh, p0, h_weight * GAUSS[idx]));
            }
        }
    }

    let mut f1 = ModalBiquad::bandpass(540.0, 4.0, sr);
    let mut f2 = ModalBiquad::bandpass(1040.0, 4.8, sr);
    let mut f3 = ModalBiquad::bandpass(2460.0, 6.2, sr);
    let mut silk_lp = ModalBiquad::lowpass((4200.0 + 2600.0 * vel).min(sr * 0.43), 0.75, sr);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.045).clamp(0.0, 1.0);
        let mut sum = 0.0;
        for &(w, p0, amp) in &partials {
            sum += (w * t + p0).sin() * amp;
        }
        let body = if choir {
            let vowel = f1.process(sum) * 0.75 + f2.process(sum) * 0.65 + f3.process(sum) * 0.38;
            sum * 0.42 + vowel * 0.95
        } else {
            silk_lp.process(sum)
        };
        *out_sample = ((body * 0.34).tanh() * attack) as f32;
    }
    buf
}

/// Balkan conical double-reed zurna lead with grace pitch scoop & nasal reed formants.
fn synth_balkan_zurna(
    f0: f64,
    vel: f64,
    total_samples: usize,
    sr: f64,
    seed: u64,
) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    let mut f_nasal1 = ModalBiquad::bandpass(1380.0, 3.4, sr);
    let mut f_nasal2 = ModalBiquad::bandpass(2720.0, 4.2, sr);
    let mut warmth_lp = ModalBiquad::lowpass((4400.0 + 1800.0 * vel).min(sr * 0.42), 0.9, sr);
    let mut phase = 0.0_f64;

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.012).clamp(0.0, 1.0);
        let vib_onset = ((t - 0.12) / 0.14).clamp(0.0, 1.0);
        let scoop = -1.6 * (-t * 48.0).exp() + vib_onset * 0.18 * (TAU * 6.8 * t).sin();
        let hz = (f0 * 2.0_f64.powf(scoop / 12.0)).clamp(20.0, sr * 0.4);
        phase = (phase + hz / sr).fract();
        let reed_pulse = if phase < 0.28 { 1.0 } else { -0.65 };
        let reed_saw = 2.0 * phase - 1.0;
        let breath = rng.next_f64() * 0.04;
        let raw = (reed_pulse * 0.58 + reed_saw * 0.42 + breath).tanh();
        let formant = raw * 0.35 + f_nasal1.process(raw) * 0.88 + f_nasal2.process(raw) * 0.52;
        *out_sample = ((warmth_lp.process(formant) * 1.25).tanh() * attack) as f32;
    }
    buf
}

/// Celtic tin whistle with crisp Irish cut grace ornament and airy breath chiff.
fn synth_celtic_whistle(
    f0: f64,
    vel: f64,
    total_samples: usize,
    sr: f64,
    seed: u64,
) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    let mut chiff_bp = ModalBiquad::bandpass((f0 * 2.4).clamp(700.0, 4600.0), 3.6, sr);
    let mut phase = 0.0_f64;

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.014).clamp(0.0, 1.0);
        let cut = 2.0 * (-t * 68.0).exp();
        let vib = 0.12 * ((t - 0.14).max(0.0) * TAU * 5.8).sin();
        let hz = (f0 * 2.0_f64.powf((cut + vib) / 12.0)).clamp(20.0, sr * 0.42);
        phase = (phase + hz / sr).fract();
        let w = TAU * phase;
        let tube = w.sin() * 0.72 + (2.0 * w).sin() * (0.28 * vel) + (3.0 * w).sin() * 0.11;
        let chiff = chiff_bp.process(rng.next_f64()) * ((-t * 42.0).exp() * 0.24 + 0.025) * vel;
        *out_sample = ((tube + chiff) * attack * 0.9) as f32;
    }
    buf
}

/// Shared Karplus-Strong plucked waveguide kernel for Cyber Shamisen & Arabic Oud.
fn synth_world_waveguide(
    f0: f64,
    vel: f64,
    total_samples: usize,
    sr: f64,
    seed: u64,
    shamisen: bool,
) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    let delay_len_f = (sr / f0).clamp(8.0, 4096.0);
    let delay_len = delay_len_f.floor() as usize;
    let frac = delay_len_f - delay_len as f64;
    let ap_coeff = (1.0 - frac) / (1.0 + frac);
    let mut line = vec![0.0_f64; delay_len + 2];
    let pick_pos = if shamisen { 0.15 } else { 0.26 };
    for (idx, slot) in line.iter_mut().enumerate().take(delay_len) {
        let u = idx as f64 / delay_len as f64;
        let tri = if u < pick_pos {
            u / pick_pos
        } else {
            (1.0 - u) / (1.0 - pick_pos)
        };
        *slot = tri * 0.6 + rng.next_f64() * (if shamisen { 0.55 } else { 0.34 }) * vel;
    }
    let feedback = if shamisen { 0.9962 } else { 0.9974 };
    let damp = if shamisen { 0.18 } else { 0.32 - 0.08 * vel };
    let mut ptr = 0_usize;
    let mut prev = 0.0_f64;
    let (mut ap_x1, mut ap_y1) = (0.0_f64, 0.0_f64);

    let mut b1 = ModalBiquad::bandpass(if shamisen { 390.0 } else { 115.0 }, 2.6, sr);
    let mut b2 = ModalBiquad::bandpass(if shamisen { 1420.0 } else { 255.0 }, 2.4, sr);
    let mut b3 = ModalBiquad::bandpass(if shamisen { 2900.0 } else { 520.0 }, 2.2, sr);
    // Secondary course string phase for Arabic Oud unison shimmer
    let mut oud_phase = 0.0_f64;
    let oud_w = TAU * f0 * 2.0_f64.powf(4.5 / 1200.0) / sr;

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let raw = line[ptr];
        let lp = (1.0 - damp) * raw + damp * prev;
        prev = raw;
        let ap = ap_coeff * lp + ap_x1 - ap_coeff * ap_y1;
        ap_x1 = lp;
        ap_y1 = ap;
        line[ptr] = ap * feedback;
        ptr = (ptr + 1) % delay_len;

        let string = if shamisen {
            // Sawari buzz asymmetry + sharp bachi skin pop
            (ap + 0.24 * ap.abs() * ap).tanh()
        } else {
            oud_phase += oud_w;
            ap * 0.72 + oud_phase.sin() * 0.30 * (-t * 2.6).exp()
        };
        let body = b1.process(string) * 0.34 + b2.process(string) * 0.28 + b3.process(string) * 0.18;
        *out_sample = ((string * 0.72 + body) * 1.15).tanh() as f32;
    }
    buf
}

fn synth_cyber_shamisen(f0: f64, vel: f64, total_samples: usize, sr: f64, seed: u64) -> Vec<f32> {
    synth_world_waveguide(f0, vel, total_samples, sr, seed, true)
}

fn synth_arabic_oud(f0: f64, vel: f64, total_samples: usize, sr: f64, seed: u64) -> Vec<f32> {
    synth_world_waveguide(f0, vel, total_samples, sr, seed, false)
}

/// Initial D / Super Eurobeat Synth Brass: 4 PolyBLEP saws + FM lip bite + 24dB TPT filter punch.
fn synth_eurobeat_brass(f0: f64, vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    const DETUNES: [f64; 4] = [-0.11, -0.03, 0.04, 0.12];
    let mut phases = [0.0_f64, 0.24, 0.51, 0.76];
    let mut fm_phase = 0.0_f64;
    let freqs = DETUNES.map(|d| f0 * 2.0_f64.powf(d / 12.0));
    let mut hp = OnePoleHp::new(165.0, sr);
    let mut ladder = TptLadder::new();
    let mut pres = ModalBiquad::bandpass(2200.0, 1.0, sr);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.005).clamp(0.0, 1.0);
        // Classic brass lip pitch drop (+18 cents settling in 22 ms)
        let punch = 2.0_f64.powf((0.18 * (-t * 115.0).exp()) / 12.0);
        let mut saw_mix = 0.0;
        for k in 0..4 {
            let dt = ((freqs[k] * punch) / sr).clamp(0.0, 0.45);
            phases[k] = (phases[k] + dt).fract();
            saw_mix += blep_saw(phases[k], dt) * 0.25;
        }
        // Metallic FM brass attack transient (ratio 2:1, decays in ~36 ms)
        fm_phase = (fm_phase + (f0 * punch) / sr).fract();
        let fm_idx = (1.8 + 1.4 * vel) * (-t * 28.0).exp();
        let fm_bite = (TAU * fm_phase + fm_idx * (TAU * 2.0 * fm_phase).sin()).sin()
            * (-t * 18.0).exp()
            * 0.42;

        // Snappy brass filter bark opening fast and settling into warm sustain
        let f_open = (t / 0.010).clamp(0.0, 1.0);
        let f_decay = (-t * 4.5).exp();
        let cutoff = 720.0 + (6200.0 * vel) * f_open * (0.42 + 0.58 * f_decay);
        let body = hp.process(saw_mix + fm_bite);
        let filtered = ladder.process(body + pres.process(body) * 0.25, cutoff, 0.65, sr);
        *out_sample = ((filtered * 1.32).tanh() * attack) as f32;
    }
    buf
}

/// Driving Initial D / Hi-NRG Eurobeat saw+sub octave bass with snappy TPT filter pluck.
fn synth_eurobeat_octave_bass(f0: f64, vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    let (mut p_sub, mut p_a, mut p_b) = (0.0_f64, 0.15_f64, 0.65_f64);
    let f_a = f0 * 2.0_f64.powf(-0.04 / 12.0);
    let f_b = f0 * 2.0_f64.powf(0.04 / 12.0);
    let mut ladder = TptLadder::new();

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.002).clamp(0.0, 1.0);
        let dt_sub = (f0 / sr).clamp(0.0, 0.45);
        let dt_a = (f_a / sr).clamp(0.0, 0.45);
        let dt_b = (f_b / sr).clamp(0.0, 0.45);
        p_sub = (p_sub + dt_sub).fract();
        p_a = (p_a + dt_a).fract();
        p_b = (p_b + dt_b).fract();

        let sub = (TAU * p_sub).sin() * 0.58;
        let saws = (blep_saw(p_a, dt_a) + blep_saw(p_b, dt_b)) * 0.5;
        let cutoff = 160.0 + (2650.0 * vel) * (-t * 13.0).exp();
        let filtered_saws = ladder.process(saws, cutoff, 0.85, sr);
        let amp_env = 0.48 + 0.52 * (-t * 7.5).exp();
        *out_sample = (((sub + filtered_saws * 0.72) * 1.38 * amp_env).tanh() * attack) as f32;
    }
    buf
}

/// Overdriven Initial D / Anime Opening Rock Guitar: waveguide string + tube saturation + 4x12 cab.
fn synth_anime_rock_guitar(
    f0: f64,
    vel: f64,
    total_samples: usize,
    sr: f64,
    seed: u64,
) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    let delay_len_f = (sr / f0).clamp(8.0, 4096.0);
    let delay_len = delay_len_f.floor() as usize;
    let frac = delay_len_f - delay_len as f64;
    let ap_coeff = (1.0 - frac) / (1.0 + frac);
    let mut line = vec![0.0_f64; delay_len + 2];
    for (idx, slot) in line.iter_mut().enumerate().take(delay_len) {
        let u = idx as f64 / delay_len as f64;
        let slope = if u < 0.16 { u / 0.16 } else { (1.0 - u) / 0.84 };
        *slot = slope * 0.62 + rng.next_f64() * 0.48 * vel;
    }
    let mut ptr = 0_usize;
    let mut prev = 0.0_f64;
    let (mut ap_x1, mut ap_y1) = (0.0_f64, 0.0_f64);
    let mut saw_phase = 0.0_f64;
    let mut hp_tight = OnePoleHp::new(130.0, sr);
    let mut cab_mid = ModalBiquad::bandpass(940.0, 1.2, sr);
    let mut cab_pres = ModalBiquad::bandpass(2450.0, 1.5, sr);
    let mut cab_lp1 = ModalBiquad::lowpass(4400.0, 0.85, sr);
    let mut cab_lp2 = ModalBiquad::lowpass(4800.0, 0.80, sr);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.003).clamp(0.0, 1.0);
        let raw = line[ptr];
        let lp = 0.80 * raw + 0.20 * prev;
        prev = raw;
        let ap = ap_coeff * lp + ap_x1 - ap_coeff * ap_y1;
        ap_x1 = lp;
        ap_y1 = ap;
        line[ptr] = ap * 0.9975;
        ptr = (ptr + 1) % delay_len;

        let vib = 0.12 * ((t - 0.12).max(0.0) * TAU * 5.8).sin();
        let dt = ((f0 * 2.0_f64.powf(vib / 12.0)) / sr).clamp(0.0, 0.45);
        saw_phase = (saw_phase + dt).fract();
        let sustain_core = blep_saw(saw_phase, dt) * 0.32 + (TAU * saw_phase).sin() * 0.22;

        // Asymmetric dual-stage tube preamp overdrive (even + odd harmonics without fizzy aliasing)
        let pre = hp_tight.process(ap * 0.85 + sustain_core);
        let stage1 = (pre * (3.0 + 1.6 * vel) + 0.14).tanh() - 0.12;
        let stage2 = (stage1 * 2.2).tanh();

        // 4x12 guitar speaker cabinet curve
        let cab = stage2 * 0.55 + cab_mid.process(stage2) * 0.34 + cab_pres.process(stage2) * 0.26;
        let smoothed = cab_lp2.process(cab_lp1.process(cab));
        *out_sample = ((smoothed * 1.12).tanh() * attack) as f32;
    }
    buf
}

/// Iconic TR-808/707 Eurobeat offbeat cowbell (540 Hz + 800 Hz square oscillators through 740 Hz BP).
fn synth_eurobeat_cowbell(vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    let mut bp = ModalBiquad::bandpass(740.0, 2.8, sr);
    let (mut p1, mut p2) = (0.0_f64, 0.0_f64);
    let dt1 = 540.0 / sr;
    let dt2 = 800.0 / sr;

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.001).clamp(0.0, 1.0);
        p1 = (p1 + dt1).fract();
        p2 = (p2 + dt2).fract();
        let sq = blep_pulse(p1, dt1, 0.5) * 0.55 + blep_pulse(p2, dt2, 0.5) * 0.45;
        let env = 0.58 * (-t * 68.0).exp() + 0.42 * (-t * 14.0).exp();
        let tone = bp.process(sq) + sq * 0.20;
        *out_sample = ((tone * env * (0.8 + 0.4 * vel) * 1.3).tanh() * attack) as f32;
    }
    buf
}

/// 6-oscillator TR-808 metallic crash cymbal + airy high-frequency shimmer.
fn synth_club_crash_cymbal(vel: f64, total_samples: usize, sr: f64, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut buf = vec![0.0_f32; total_samples];
    const FREQS: [f64; 6] = [205.3, 304.4, 369.6, 522.7, 540.0, 800.0];
    let mut phases = [0.0_f64; 6];
    let mut hp = OnePoleHp::new(3600.0, sr);
    let mut air_bp = ModalBiquad::bandpass(7800.0, 0.9, sr);

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.002).clamp(0.0, 1.0);
        let mut metallic = 0.0;
        for (k, &f) in FREQS.iter().enumerate() {
            let dt = (f / sr).clamp(0.0, 0.45);
            phases[k] = (phases[k] + dt).fract();
            metallic += blep_pulse(phases[k], dt, 0.5) * (1.0 / 6.0);
        }
        let raw = metallic * 0.48 + rng.next_f64() * 0.52;
        let high = hp.process(raw) * 0.65 + air_bp.process(raw) * 0.55;
        let env = 0.35 * (-t * 14.0).exp() + 0.65 * (-t * 2.4).exp();
        *out_sample = ((high * env * (0.75 + 0.35 * vel) * 1.3).tanh() * attack) as f32;
    }
    buf
}

/// Crystal PolyBLEP sparkle arpeggio pluck for anime opening & Eurobeat choruses.
fn synth_anime_sparkle_arp(f0: f64, vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    let mut p1 = 0.0_f64;
    let mut p2 = 0.33_f64;
    let mut hp = OnePoleHp::new(240.0, sr);
    let mut ladder = TptLadder::new();

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.0015).clamp(0.0, 1.0);
        let dt1 = (f0 / sr).clamp(0.0, 0.45);
        let dt2 = ((f0 * 2.002) / sr).clamp(0.0, 0.45);
        p1 = (p1 + dt1).fract();
        p2 = (p2 + dt2).fract();
        let tone = blep_pulse(p1, dt1, 0.25) * 0.65
            + blep_saw(p2, dt2) * 0.28 * (-t * 10.0).exp()
            + (TAU * p1).sin() * 0.25;
        let cutoff = 1100.0 + 7500.0 * vel * (-t * 9.5).exp();
        let env = 0.20 + 0.80 * (-t * 6.8).exp();
        let out = ladder.process(hp.process(tone), cutoff, 0.45, sr) * env;
        *out_sample = ((out * 1.2).tanh() * attack) as f32;
    }
    buf
}

/// Uprising PolyBLEP 7-voice supersaw + octave sparkle lead built for soaring sabi choruses.
fn synth_uprising_trance_lead(f0: f64, vel: f64, total_samples: usize, sr: f64) -> Vec<f32> {
    let mut buf = vec![0.0_f32; total_samples];
    const DETUNES: [f64; 7] = [-0.12, -0.07, -0.025, 0.0, 0.025, 0.07, 0.12];
    const WEIGHTS: [f64; 7] = [0.10, 0.13, 0.17, 0.22, 0.17, 0.13, 0.10];
    let mut phases = [0.0, 0.14, 0.29, 0.43, 0.57, 0.71, 0.86];
    let mut oct_phase = 0.0_f64;
    let freqs = DETUNES.map(|d| f0 * 2.0_f64.powf(d / 12.0));
    let mut hp = OnePoleHp::new(180.0, sr);
    let mut ladder = TptLadder::new();

    for (i, out_sample) in buf.iter_mut().enumerate() {
        let t = i as f64 / sr;
        let attack = (t / 0.004).clamp(0.0, 1.0);
        let scoop = 2.0_f64.powf((-0.25 * (-t * 95.0).exp()) / 12.0);
        let mut sum = 0.0;
        for k in 0..7 {
            let dt = ((freqs[k] * scoop) / sr).clamp(0.0, 0.45);
            phases[k] = (phases[k] + dt).fract();
            sum += blep_saw(phases[k], dt) * WEIGHTS[k];
        }
        let dt_oct = ((f0 * 2.0 * scoop) / sr).clamp(0.0, 0.45);
        oct_phase = (oct_phase + dt_oct).fract();
        sum += blep_saw(oct_phase, dt_oct) * (0.20 * vel);
        let cutoff = 1800.0 + (7800.0 * vel) * (0.55 + 0.45 * (-t * 6.0).exp());
        let out = ladder.process(hp.process(sum), cutoff, 0.5, sr);
        *out_sample = ((out * 1.2).tanh() * attack) as f32;
    }
    buf
}
