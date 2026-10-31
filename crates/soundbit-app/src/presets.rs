//! The built-in instrument presets.
//!
//! Presets are data, not code: they live as JSON files in `assets/presets/` and are embedded
//! into the binary at compile time, so every front end offers the same library without
//! having to find files at run time.

use soundbit_formats::{Preset, parse_presets};
use std::sync::{Arc, OnceLock};

use crate::AppError;

/// The preset files, by name.
const FILES: [(&str, &str); 12] = [
    (
        "zzfx.json",
        include_str!("../../../assets/presets/zzfx.json"),
    ),
    (
        "legacy_songs.json",
        include_str!("../../../assets/presets/legacy_songs.json"),
    ),
    (
        "ctg_alternatives.json",
        include_str!("../../../assets/presets/ctg_alternatives.json"),
    ),
    (
        "orchestral.json",
        include_str!("../../../assets/presets/orchestral.json"),
    ),
    (
        "drums.json",
        include_str!("../../../assets/presets/drums.json"),
    ),
    (
        "chip.json",
        include_str!("../../../assets/presets/chip.json"),
    ),
    ("fm.json", include_str!("../../../assets/presets/fm.json")),
    (
        "aloboi.json",
        include_str!("../../../assets/presets/aloboi.json"),
    ),
    (
        "oscilloscope.json",
        include_str!("../../../assets/presets/oscilloscope.json"),
    ),
    (
        "motion_sfx.json",
        include_str!("../../../assets/presets/motion_sfx.json"),
    ),
    (
        "pc98_opna.json",
        include_str!("../../../assets/presets/pc98_opna.json"),
    ),
    (
        "generalized_triggers.json",
        include_str!("../../../assets/presets/generalized_triggers.json"),
    ),
];

/// A searchable collection of presets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetLibrary {
    pub(crate) presets: Arc<[Preset]>,
    pub(crate) warnings: Arc<[String]>,
}

impl PresetLibrary {
    /// The presets shipped with SoundBit.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Presets`] if a built-in file is invalid, which tests rule out.
    pub fn builtin() -> Result<Self, AppError> {
        static BUILTIN: OnceLock<Arc<[Preset]>> = OnceLock::new();
        if let Some(presets) = BUILTIN.get() {
            return Ok(Self {
                presets: Arc::clone(presets),
                warnings: Arc::from([]),
            });
        }
        let mut presets = Vec::new();
        for (file, text) in FILES {
            presets
                .extend(parse_presets(text).map_err(|source| AppError::Presets { file, source })?);
        }
        Ok(Self {
            presets: Arc::clone(BUILTIN.get_or_init(|| presets.into())),
            warnings: Arc::from([]),
        })
    }

    /// Every preset, grouped by file and in file order.
    pub fn all(&self) -> &[Preset] {
        &self.presets
    }

    /// Saved files that could not be loaded. Factory presets remain available.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// The preset with this id.
    pub fn find(&self, id: &str) -> Option<&Preset> {
        self.presets.iter().find(|preset| preset.id == id)
    }

    /// Ids of all presets, for error messages that list the choices.
    pub fn ids(&self) -> Vec<&str> {
        self.presets
            .iter()
            .map(|preset| preset.id.as_str())
            .collect()
    }
}
