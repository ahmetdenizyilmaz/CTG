//! Persistent user sounds, separate from the immutable factory library.
use crate::PresetLibrary;
use soundbit_core::Instrument;
use soundbit_formats::{Preset, parse_presets};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);

/// Location shared by desktop, CLI and AI. An override supports portable libraries.
///
/// # Errors
/// Returns an error if no home directory is available.
pub fn user_preset_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("CTG_PRESET_DIR") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or("Cannot locate the user preset directory; set CTG_PRESET_DIR")?;
    Ok(PathBuf::from(home)
        .join("Music")
        .join("CTG")
        .join("presets"))
}

/// Save a new copy; never overwrite another sound. Returns its preset and file path.
///
/// # Errors
/// Returns an error for a blank/long name or failed filesystem writes.
pub fn save_user_preset(
    directory: &Path,
    instrument: &Instrument,
) -> Result<(Preset, PathBuf), String> {
    if instrument.name.trim().is_empty() || instrument.name.chars().count() > 160 {
        return Err("Give the instrument a name of 1-160 characters before saving".into());
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let id = format!("user_{stamp}_{}", SERIAL.fetch_add(1, Ordering::Relaxed));
    let preset = Preset {
        id,
        category: "saved".into(),
        description: "Your saved instrument settings".into(),
        source: "CTG Studio user preset".into(),
        instrument: instrument.clone(),
    };
    let text = serde_json::to_vec_pretty(
        &serde_json::json!({"format":"soundbit-presets", "format_version":1,
        "source":preset.source, "presets":[preset]}),
    )
    .map_err(|e| e.to_string())?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let path = directory.join(format!("{}.json", preset.id));
    // Write a private temporary file, then publish the complete JSON by a hard link.
    // Readers only see .json files; a failed save cannot truncate an existing preset.
    let temp = directory.join(format!("{}.tmp", preset.id));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(&text)?;
        file.sync_all()?;
        drop(file);
        fs::hard_link(&temp, &path)
    })();
    let _ = fs::remove_file(&temp);
    result.map_err(|e| format!("Could not save {}: {e}", path.display()))?;
    Ok((preset, path))
}

impl PresetLibrary {
    /// Factory presets plus the user's saved sounds. Unreadable custom files are
    /// reported by `warnings` while the remaining library stays usable.
    ///
    /// # Errors
    /// Returns an error if factory presets or the home directory cannot be loaded.
    pub fn available() -> Result<Self, String> {
        Self::with_user_directory(&user_preset_dir()?)
    }

    /// Load a specific saved-sound directory; also used for portable libraries.
    ///
    /// # Errors
    /// Returns an error if the embedded factory library is invalid.
    pub fn with_user_directory(directory: &Path) -> Result<Self, String> {
        let mut library = Self::builtin().map_err(|e| e.to_string())?;
        let mut presets = library.all().to_vec();
        let mut warnings = Vec::new();
        let files = match fs::read_dir(directory) {
            Ok(files) => files,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(library),
            Err(error) => {
                library.warnings = vec![format!("{}: {error}", directory.display())].into();
                return Ok(library);
            }
        };
        let mut paths: Vec<_> = files
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        for path in paths {
            let result = fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|s| {
                    parse_presets(s.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())
                });
            match result {
                Ok(saved) => {
                    for preset in saved {
                        if presets.iter().any(|p| p.id == preset.id) {
                            warnings.push(format!(
                                "{}: duplicate preset id {}",
                                path.display(),
                                preset.id
                            ));
                        } else {
                            presets.push(preset);
                        }
                    }
                }
                Err(error) => warnings.push(format!("{}: {error}", path.display())),
            }
        }
        library.presets = presets.into();
        library.warnings = warnings.into();
        Ok(library)
    }
}
