use crate::db::{CommandHistoryEntry, DatabaseManager};
use chrono::Utc;
use std::env;
use std::os::unix::fs::MetadataExt;

fn get_session_id() -> String {
    if let Ok(session_id) = env::var("XDG_SESSION_ID") {
        return format!("xdg:{}", session_id);
    }

    if let Ok(terminal) = std::fs::read_link("/proc/self/fd/0") {
        if let Some(term_name) = terminal.to_str() {
            if term_name.starts_with("/dev/") {
                let clean_name = term_name.replace("/dev/", "");
                if let Ok(metadata) = std::fs::metadata(term_name) {
                    return format!("term_{}_{}", clean_name, metadata.ino());
                }
                return format!("term_{}", clean_name);
            }
        }
    }

    if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
        let fields: Vec<&str> = stat.split_whitespace().collect();
        if fields.len() > 5 {
            return format!("process_sid_{}", fields[5]);
        }
    }

    if let Ok(fish_pid) = env::var("fish_pid") {
        return format!("fish_{}", fish_pid);
    }

    if let Ok(bash_pid) = env::var("BASHPID") {
        return format!("bash_{}", bash_pid);
    }

    let ppid = std::fs::read_to_string("/proc/self/stat")
        .ok()
        .and_then(|s| s.split_whitespace().nth(3).map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    format!("shell_{}", ppid)
}

fn parse_shell_command(command: &str) -> Vec<(String, String)> {
    let mut commands = Vec::new();
    
    let parts: Vec<&str> = command
        .split(&[';', '\n'])
        .flat_map(|part| {
            let mut subparts = Vec::new();
            let mut current = part;
            
            while let Some(pos) = current.find("&&").or_else(|| current.find("||")) {
                let (before, after_op) = current.split_at(pos);
                subparts.push(before.trim());
                current = &after_op[2..].trim();
            }
            subparts.push(current.trim());
            subparts
        })
        .filter(|s| !s.is_empty())
        .collect();
    
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        
        let pipeline_parts: Vec<&str> = part.split('|').collect();
        
        for pipe_part in pipeline_parts {
            let pipe_part = pipe_part.trim();
            if pipe_part.is_empty() {
                continue;
            }
            
            let words: Vec<&str> = pipe_part.split_whitespace().collect();
            if let Some(first_word) = words.first() {
                commands.push((pipe_part.to_string(), first_word.to_string()));
            }
        }
    }
    
    commands
}

pub async fn log_command(command: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_manager = DatabaseManager::new().await?;
    let session_key = get_session_id();
    let session_id = db_manager.get_or_create_session(&session_key).await?;
    
    let commands = parse_shell_command(command);
    
    if commands.is_empty() {
        let entry = CommandHistoryEntry {
            id: None,
            timestamp: Utc::now(),
            command: command.to_string(),
            binary: "unknown".to_string(),
            user: env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            pwd: env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            session_id,
        };

        db_manager.log_command(&entry).await?;
    } else {
        for (cmd, binary) in commands {
            let entry = CommandHistoryEntry {
                id: None,
                timestamp: Utc::now(),
                command: cmd,
                binary,
                user: env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
                pwd: env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "unknown".to_string()),
                session_id,
            };

            db_manager.log_command(&entry).await?;
        }
    }

    Ok(())
}
