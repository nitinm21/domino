use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "domino-recorder", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start a recording session. Writes session info to stdout as JSON.
    Start {
        /// Override default output directory (~/.domino/recordings).
        #[arg(long)]
        out_dir: Option<PathBuf>,
    },
    /// Stop the currently active recording session.
    Stop,
    /// Print active session info as JSON, or "{}" if none.
    Status,
    /// Print diagnostic info about permissions, devices, OS version.
    Doctor,
}
