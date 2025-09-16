use crate::db::{CommandHistoryEntry, DatabaseManager};
use crate::ui;

pub async fn fetch_command_history() -> Result<Vec<CommandHistoryEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let db_manager = DatabaseManager::new().await?;

    let entries = db_manager.fetch_recent_commands(100).await?;
    
    if entries.is_empty() {
        println!("No commands found in history.");
        return Ok(entries);
    }

    println!("Found {} commands in history:", entries.len());
    for (i, entry) in entries.iter().take(5).enumerate() {
        println!("{}. {} - {} ({})", i + 1, entry.timestamp.format("%Y-%m-%d %H:%M:%S"), entry.command, entry.binary);
    }

    // Launch the TUI only if we have valid data
    ui::run_tui(entries.clone())?;
    
    Ok(entries)
}