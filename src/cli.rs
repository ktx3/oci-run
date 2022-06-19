// This is free and unencumbered software released into the public domain.

//! Command-line interface module.

use std::path::PathBuf;

use clap::Parser;

/// Command-line options.
#[derive(Debug, Parser)]
#[clap(about, long_about = None, version)]
pub struct Cli {
    /// Command to run inside the container.
    #[clap(last = true)]
    pub command: Vec<String>,
    /// Path to the config file.
    #[clap(long, parse(from_os_str), short, value_name = "PATH")]
    pub config_file: Option<PathBuf>,
    /// Enable additional debug logging.
    #[clap(long, short)]
    pub debug: bool,
    /// Profile path.
    #[clap(long, short, value_name = "PROFILE")]
    pub profile: Option<PathBuf>,
    /// Decrease logging verbosity.
    #[clap(long, parse(from_occurrences), short)]
    pub quiet: u64,
    /// Increase logging verbosity.
    #[clap(long, parse(from_occurrences), short)]
    pub verbose: u64,
}

impl Cli {
    /// Check if additional debug logging is enabled.
    #[must_use]
    pub fn debug(&self) -> bool {
        self.debug
    }

    /// Get logging verbosity.
    #[must_use]
    pub fn verbosity(&self) -> u64 {
        // Show warnings and above by default
        self.verbose.saturating_add(2).saturating_sub(self.quiet)
    }
}
