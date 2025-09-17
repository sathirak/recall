use crate::db::{CommandHistoryEntry, DatabaseManager};
use crate::ui;

pub async fn get_command_history() -> Result<Vec<CommandHistoryEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let db_manager = DatabaseManager::new().await?;

    let entries = db_manager.fetch_recent_commands(100).await?;
    
    if entries.is_empty() {
        println!("No commands found in history.");
        return Ok(entries);
    }

    ui::run_tui(entries.clone())?;
    
    Ok(entries)
}