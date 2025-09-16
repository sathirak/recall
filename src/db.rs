use chrono::{DateTime, Utc};
use dirs::home_dir;
use libsql::{Builder, Database};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandHistoryEntry {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub binary: String,
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

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = Builder::new_local(db_path).build().await?;

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
                binary TEXT NOT NULL,
                user TEXT NOT NULL,
                pwd TEXT NOT NULL,
                session_id TEXT NOT NULL
            )",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON command_history(timestamp DESC)",
            (),
        )
        .await?;

        Ok(())
    }

    pub async fn log_command(
        &self,
        entry: &CommandHistoryEntry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.db.connect()?;

        conn.execute(
            "INSERT INTO command_history (timestamp, command, binary, user, pwd, session_id) 
             VALUES (?, ?, ?, ?, ?, ?)",
            (
                entry.timestamp.to_rfc3339().as_str(),
                entry.command.as_str(),
                entry.binary.as_str(),
                entry.user.as_str(),
                entry.pwd.as_str(),
                entry.session_id.as_str(),
            ),
        )
        .await?;

        Ok(())
    }

    pub async fn fetch_recent_commands(
        &self,
        limit: i64,
    ) -> Result<Vec<CommandHistoryEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.db.connect()?;

        let mut rows = conn
            .query(
                "SELECT id, timestamp, command, binary, user, pwd, session_id 
             FROM command_history 
             ORDER BY timestamp DESC 
             LIMIT ?",
                &[limit],
            )
            .await?;

        let mut commands = Vec::new();

        while let Some(row) = rows.next().await? {
            let entry = CommandHistoryEntry {
                id: Some(row.get::<i64>(0)?),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<String>(1)?)?
                    .with_timezone(&Utc),
                command: row.get::<String>(2)?,
                binary: row.get::<String>(3)?,
                user: row.get::<String>(4)?,
                pwd: row.get::<String>(5)?,
                session_id: row.get::<String>(6)?,
            };
            commands.push(entry);
        }

        Ok(commands)
    }
}

pub fn get_db_file_path() -> PathBuf {
    if let Ok(test_path) = std::env::var("RECALL_DB_PATH") {
        return PathBuf::from(test_path);
    }
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(".recall");
    path.push("recall.db");
    path
}
