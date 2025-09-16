use clap::{Parser, Subcommand};
use std::process::exit;

mod db;
mod commands;

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
    Install {
        #[arg(short, long, default_value = "bash")]
        shell: String,
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
        Some(Commands::Install { shell }) => {
            if let Err(e) = commands::install_shell_integration(shell) {
                eprintln!("Error installing shell integration: {}", e);
                exit(1);
            }
        }
        None => {
            println!("Recall - Terminal Command Logger");
            println!("Use 'recall --help' for more information.");
        }
    }
}
