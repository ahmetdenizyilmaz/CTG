//! Headless, repeatable composition and native preset discovery.
use anyhow::{Context, Result, bail};
use soundbit_app::{
    PresetLibrary, RenderOptions, Session,
    compose::{ComposeConfig, generate, randomize},
};
use std::path::Path;

pub(crate) fn run(
    config: Option<&Path>,
    output: &Path,
    seed: Option<u32>,
    wav: Option<&Path>,
    report: Option<&Path>,
    force: bool,
) -> Result<()> {
    for path in [Some(output), wav, report].into_iter().flatten() {
        if path.exists() && !force {
            bail!(
                "{} exists; choose another path or use --force",
                path.display()
            );
        }
    }
    let mut recipe: ComposeConfig = match config {
        Some(path) => serde_json::from_str(&std::fs::read_to_string(path).context("read recipe")?)
            .context("parse recipe")?,
        None => ComposeConfig::default(),
    };
    if let Some(seed) = seed {
        recipe.seed = seed;
    }
    let composition = generate(&recipe).map_err(anyhow::Error::msg)?;
    let mut session = Session::new(composition.song);
    session.save_as(output)?;
    if let Some(path) = wav {
        session.export_wav(path, &RenderOptions::default())?;
    }
    let json = serde_json::to_string_pretty(&composition.report)?;
    if let Some(path) = report {
        std::fs::write(path, &json)?;
    }
    println!("{json}");
    Ok(())
}

pub(crate) fn defaults() -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&ComposeConfig::default())?
    );
    Ok(())
}
pub(crate) fn random_config(seed: u32, instruments_for: Option<&Path>) -> Result<()> {
    let config = instruments_for
        .map(|path| -> Result<ComposeConfig> {
            serde_json::from_str(&std::fs::read_to_string(path).context("read recipe")?)
                .context("parse recipe")
        })
        .transpose()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&randomize(seed, config.as_ref()))?
    );
    Ok(())
}
pub(crate) fn presets() -> Result<()> {
    let library = PresetLibrary::available().map_err(anyhow::Error::msg)?;
    println!("{}", serde_json::to_string_pretty(library.all())?);
    Ok(())
}
