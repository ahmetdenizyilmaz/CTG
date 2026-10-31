//! Validated `ZzFX` sound parameters and tracker effects.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "validated fixed-point magnitudes are below 2^53"
)]
use serde::{Deserialize, Serialize};

/// `ZzFX`'s 21 parameters in upstream order, stored as fixed point for exact project equality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "Vec<f64>", into = "Vec<f64>")]
pub struct ZzfxParameters([i64; 21]);

impl Default for ZzfxParameters {
    fn default() -> Self {
        Self::try_from(vec![0.5, 0.0, 261.625_565, 0.004, 0.2, 0.15, 1.0, 1.0])
            .unwrap_or(Self([0; 21]))
    }
}

impl ZzfxParameters {
    /// Values in upstream `ZzFX` order. Frequency is the frequency at MIDI C4.
    pub fn values(self) -> [f64; 21] {
        self.0.map(|v| v as f64 / 1_000_000_000.0)
    }
}

impl From<ZzfxParameters> for Vec<f64> {
    fn from(value: ZzfxParameters) -> Self {
        value.values().to_vec()
    }
}

impl TryFrom<Vec<f64>> for ZzfxParameters {
    type Error = String;
    fn try_from(values: Vec<f64>) -> Result<Self, Self::Error> {
        if values.len() > 21 {
            return Err("ZzFX accepts at most 21 parameters".into());
        }
        let mut p = [
            1.0, 0.05, 220.0, 0.0, 0.0, 0.1, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            1.0, 0.0, 0.0, 0.0,
        ];
        p[..values.len()].copy_from_slice(&values);
        let bounds = [
            (0., 4.),
            (0., 1.),
            (0., 20000.),
            (0., 10.),
            (0., 10.),
            (0., 10.),
            (0., 5.),
            (0., 10.),
            (-1000., 1000.),
            (-1000., 1000.),
            (-20000., 20000.),
            (0., 10.),
            (0., 10.),
            (0., 50.),
            (-20000., 20000.),
            (0., 10.),
            (0., 10.),
            (0., 2.),
            (0., 10.),
            (0., 1.),
            (-20000., 20000.),
        ];
        for (i, (&v, &(min, max))) in p.iter().zip(&bounds).enumerate() {
            if !v.is_finite() || v < min || v > max || (i == 6 && v.fract() != 0.) {
                return Err(format!(
                    "ZzFX parameter {i} must be finite and between {min} and {max}"
                ));
            }
        }
        if p[3] + p[4] + p[5] + p[16] + p[18] > 15.0 {
            return Err("ZzFX sound must be at most 15 seconds".into());
        }
        Ok(Self(p.map(|v| (v * 1_000_000_000.0).round() as i64)))
    }
}

/// `ZzFX` Studio tracker effect, applied to a single note's sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZzfxEffect {
    /// Effect command.
    pub code: ZzfxEffectCode,
    /// Tracker byte (0–255).
    pub value: u8,
}

/// Supported `ZzFX` Studio commands. VB retains the original repeat/tremolo behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZzfxEffectCode {
    /// Slide up.
    SU,
    /// Slide down.
    SD,
    /// Original Studio wobble.
    VB,
    /// Duty/shape curve.
    DT,
    /// Shortened sustain and release.
    ST,
    /// Delayed downward pitch jump.
    PD,
    /// Sample-and-hold bit crush.
    BC,
    /// Tremolo.
    TR,
}

impl ZzfxEffect {
    /// Apply the upstream Studio mapping without altering the reusable base instrument.
    pub fn apply(self, p: &mut [f64; 21]) {
        let v = f64::from(self.value);
        match self.code {
            ZzfxEffectCode::SU => p[8] = v / 255. * 0.8,
            ZzfxEffectCode::SD => p[8] = -v / 255. * 0.8,
            ZzfxEffectCode::VB | ZzfxEffectCode::TR => {
                let speed = f64::from((self.value >> 4).max(1));
                let depth = f64::from((self.value & 15).max(1));
                if self.code == ZzfxEffectCode::VB {
                    p[12] = 0.05 + (16. - speed) * 0.02;
                    p[19] = depth / 15. * 0.5;
                } else {
                    p[12] = 0.04 + (16. - speed) * 0.015;
                    p[19] = depth / 15. * 0.7;
                }
            }
            ZzfxEffectCode::DT => {
                p[7] = match self.value {
                    1 => 0.25,
                    2 => 0.5,
                    _ => 1.,
                }
            }
            ZzfxEffectCode::ST => {
                p[4] *= 1. - v / 255. * 0.85;
                p[5] *= 1. - v / 255. * 0.85;
            }
            ZzfxEffectCode::PD => {
                p[10] = -v / 255. * 40.;
                p[11] = 0.005 + v / 255. * 0.02;
            }
            ZzfxEffectCode::BC => p[15] = v / 255.,
        }
    }
}
