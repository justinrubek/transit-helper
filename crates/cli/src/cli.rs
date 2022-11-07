use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Store the position data over time
    LogPositionData {
        /// The path to the file to store the data in
        #[arg(short, long)]
        path: PathBuf,
    },
}
