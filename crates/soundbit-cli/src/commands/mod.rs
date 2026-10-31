//! Subcommand implementations.

mod compose;
mod info;
mod mcp;
mod play;
mod render;

use crate::cli::Command;

pub(crate) fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Compose {
            config,
            output,
            seed,
            wav,
            report,
            force,
        } => compose::run(
            config.as_deref(),
            &output,
            seed,
            wav.as_deref(),
            report.as_deref(),
            force,
        ),
        Command::ComposeConfig => compose::defaults(),
        Command::RandomConfig {
            seed,
            instruments_for,
        } => compose::random_config(seed, instruments_for.as_deref()),
        Command::Presets => compose::presets(),
        Command::Render {
            input,
            output,
            sample_rate,
        } => render::run(&input, output.as_deref(), sample_rate),
        Command::Info { input } => info::run(&input),
        Command::Play {
            input,
            from_bar,
            looping,
        } => play::run(&input, from_bar, looping),
        Command::Mcp { root, open, http } => mcp::run(root, open, http),
    }
}
