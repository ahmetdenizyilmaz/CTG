//! Command-line arguments.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// CTG (`ChipTuneGenerator` MCP): 8- and 16-bit music for humans and AI agents.
#[derive(Debug, Parser)]
#[command(name = "ctg", version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Import a `ZzFX` Studio project as an editable CTG project.
    Import {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Generate an editable native composition. Print defaults with compose-config.
    Compose {
        /// JSON recipe; omitted fields use defaults.
        #[arg(long)]
        config: Option<PathBuf>,
        /// Native project output (.sbproj).
        #[arg(short, long)]
        output: PathBuf,
        /// Override the recipe seed.
        #[arg(long)]
        seed: Option<u32>,
        /// Also render a WAV file.
        #[arg(long)]
        wav: Option<PathBuf>,
        /// Also write the musical analysis as JSON.
        #[arg(long)]
        report: Option<PathBuf>,
        /// Replace existing output files.
        #[arg(long)]
        force: bool,
    },
    /// Print the default composition recipe as JSON.
    ComposeConfig,
    /// Print a constrained random recipe; use it with compose --config.
    RandomConfig {
        /// Discovery seed, saved in a fully random recipe for repeatability.
        #[arg(long)]
        seed: u32,
        /// Change only the sounds in this existing JSON recipe.
        #[arg(long)]
        instruments_for: Option<PathBuf>,
    },
    /// List all built-in instruments as JSON.
    Presets,
    /// Render a project to a 16-bit WAV file.
    Render {
        /// Project file (.sbproj).
        input: PathBuf,
        /// Output WAV file. Defaults to the input path with a .wav extension.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Sample rate in hertz.
        #[arg(long, default_value_t = 48_000)]
        sample_rate: u32,
    },
    /// Play a project through the default audio device.
    Play {
        /// Project file (.sbproj).
        input: PathBuf,
        /// Bar to start from (counted from 0).
        #[arg(long, default_value_t = 0)]
        from_bar: u16,
        /// Repeat the loop region (or the whole song) until interrupted.
        #[arg(long = "loop")]
        looping: bool,
    },
    /// Show a summary of a project and any chip-profile problems.
    Info {
        /// Project file (.sbproj).
        input: PathBuf,
    },
    /// Run the MCP server on standard input/output, so AI agents can compose music.
    Mcp {
        /// Directory the agent may read and write projects and exports in.
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Project to open at start-up. Defaults to a new, empty song.
        #[arg(long)]
        open: Option<PathBuf>,
        /// Serve streamable HTTP on 127.0.0.1 at this port instead of stdio.
        #[arg(long, value_name = "PORT")]
        http: Option<u16>,
    },
}
