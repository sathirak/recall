use chrono::{DateTime, Utc};
use dirs::home_dir;
use libsql::{Builder, Database};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct CommandEntry {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub user: String,
    pub pwd: String,
    pub session_id: String,
}

pub struct DatabaseManager {
    db: Arc<Database>,
}

impl DatabaseManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let db_path = get_db_file_path();
        
        // Create directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let db = Builder::new_local(db_path)
            .build()
            .await?;
        
        let manager = DatabaseManager { db: Arc::new(db) };
        manager.init_schema().await?;
        
        Ok(manager)
    }
    
    async fn init_schema(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.db.connect()?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS command_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                command TEXT NOT NULL,
                user TEXT NOT NULL,
                pwd TEXT NOT NULL,
                session_id TEXT NOT NULL
            )",
            (),
        )
        .await?;
        
        // Create an index on timestamp for faster queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON command_history(timestamp DESC)",
            (),
        )
        .await?;
        
        Ok(())
    }
    
    pub async fn log_command(&self, entry: &CommandEntry) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.db.connect()?;
        
        conn.execute(
            "INSERT INTO command_history (timestamp, command, user, pwd, session_id) 
             VALUES (?, ?, ?, ?, ?)",
            (
                entry.timestamp.to_rfc3339().as_str(),
                entry.command.as_str(),
                entry.user.as_str(),
                entry.pwd.as_str(),
                entry.session_id.as_str(),
            ),
        )
        .await?;
        
        // Get the last inserted row ID
        let mut rows = conn.query("SELECT last_insert_rowid()", ()).await?;
        if let Some(row) = rows.next().await? {
            Ok(row.get::<i64>(0)?)
        } else {
            Ok(0)
        }
    }
    
    pub async fn get_history(&self, limit: usize) -> Result<Vec<CommandEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.db.connect()?;
        
        let mut rows = conn.query(
            "SELECT id, timestamp, command, user, pwd, session_id 
             FROM command_history 
             ORDER BY timestamp DESC 
             LIMIT ?",
            [limit as i64],
        )
        .await?;
        
        let mut entries = Vec::new();
        
        while let Some(row) = rows.next().await? {
            let timestamp_str: String = row.get(1)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)?
                .with_timezone(&Utc);
            
            entries.push(CommandEntry {
                id: Some(row.get(0)?),
                timestamp,
                command: row.get(2)?,
                user: row.get(3)?,
                pwd: row.get(4)?,
                session_id: row.get(5)?,
            });
        }
        
        Ok(entries)
    }
}

pub fn get_db_file_path() -> PathBuf {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(".agito");
    path.push("history.db");
    path
}
