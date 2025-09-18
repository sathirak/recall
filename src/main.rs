use clap::{Parser, Subcommand};
use std::process::exit;

mod commands;
mod db;
mod ui;

#[derive(Parser)]
#[command(name = "recall")]
#[command(about = "Command history manager")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Log {
        command: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Log { command }) => {
            if let Err(e) = commands::log_command(command).await {
                eprintln!("Error logging command: {}", e);
                exit(1);
            }
        }
        None => {
            if let Err(e) = commands::get_command_history().await {
                eprintln!("Error fetching command history: {}", e);
                exit(1);
            }
        }
    }
}
