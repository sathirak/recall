use crate::db::{CommandEntry, DatabaseManager};
use chrono::Utc;
use std::env;
use std::os::unix::fs::MetadataExt;

fn get_session_id() -> String {
    // Method 1: Use XDG_SESSION_ID if available (systemd sessions)
    if let Ok(session_id) = env::var("XDG_SESSION_ID") {
        return format!("xdg:{}", session_id);
    }

    // Method 2: Check for terminal-specific session info
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

    // Method 3: Use process session ID from /proc/self/stat
    if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
        let fields: Vec<&str> = stat.split_whitespace().collect();
        if fields.len() > 5 {
            return format!("process_sid_{}", fields[5]);
        }
    }

    // Method 4: Use fish's process ID (if in fish shell)
    if let Ok(fish_pid) = env::var("fish_pid") {
        return format!("fish_{}", fish_pid);
    }

    // Method 5: Use shell-specific variables
    if let Ok(bash_pid) = env::var("BASHPID") {
        return format!("bash_{}", bash_pid);
    }

    // Fallback: Create a session ID based on parent shell process
    let ppid = std::fs::read_to_string("/proc/self/stat")
        .ok()
        .and_then(|s| s.split_whitespace().nth(3).map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    format!("shell_{}", ppid)
}

fn resolve_binary_name(binary: &str) -> String {
    // First check if it's an alias
    if let Ok(output) = std::process::Command::new("bash")
        .args(&["-c", &format!("type -t '{}'", binary)])
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let output_trimmed = output_str.trim();
            if output_trimmed == "alias" {
                // Get the actual command the alias points to
                if let Ok(alias_output) = std::process::Command::new("bash")
                    .args(&["-c", &format!("alias '{}'", binary)])
                    .output()
                {
                    if alias_output.status.success() {
                        let alias_str = String::from_utf8_lossy(&alias_output.stdout);
                        // Parse alias output like: alias ls='ls --color=auto'
                        if let Some(start) = alias_str.find("='") {
                            if let Some(end) = alias_str[start + 2..].find('\'') {
                                let full_command = &alias_str[start + 2..start + 2 + end];
                                // Extract just the binary name
                                let first_word = full_command.split_whitespace().next().unwrap_or(binary);
                                return first_word.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    
    // If not an alias, return the binary as is
    binary.to_string()
}

fn parse_shell_command(command: &str) -> Vec<(String, String)> {
    let mut commands = Vec::new();
    
    // Split on command separators: ; && ||
    // This is a simplified approach - a full parser would be more complex
    let parts: Vec<&str> = command
        .split(&[';', '\n'])
        .flat_map(|part| {
            // Handle && and || operators
            let mut subparts = Vec::new();
            let mut current = part;
            
            while let Some(pos) = current.find("&&").or_else(|| current.find("||")) {
                let (before, after_op) = current.split_at(pos);
                subparts.push(before.trim());
                current = &after_op[2..].trim(); // Skip the operator
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
        
        // Handle pipes
        let pipeline_parts: Vec<&str> = part.split('|').collect();
        
        for pipe_part in pipeline_parts {
            let pipe_part = pipe_part.trim();
            if pipe_part.is_empty() {
                continue;
            }
            
            // Extract the first word as the binary
            let words: Vec<&str> = pipe_part.split_whitespace().collect();
            if let Some(first_word) = words.first() {
                let binary = resolve_binary_name(first_word);
                commands.push((pipe_part.to_string(), binary));
            }
        }
    }
    
    commands
}

pub async fn log_command(command: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_manager = DatabaseManager::new().await?;
    
    // Parse the command to extract individual commands and their binaries
    let commands = parse_shell_command(command);
    
    if commands.is_empty() {
        // Fallback: if parsing yields no results, log the original command
        let entry = CommandEntry {
            id: None,
            timestamp: Utc::now(),
            command: command.to_string(),
            binary: "unknown".to_string(),
            user: env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            pwd: env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            session_id: get_session_id(),
        };

        let row_id = db_manager.log_command(&entry).await?;
        
        println!(
            "Command logged: {} (Binary: unknown, Session: {}, ID: {})",
            command, entry.session_id, row_id
        );
    } else {
        // Log each command separately
        for (cmd, binary) in commands {
            let entry = CommandEntry {
                id: None,
                timestamp: Utc::now(),
                command: cmd.clone(),
                binary: binary.clone(),
                user: env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
                pwd: env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "unknown".to_string()),
                session_id: get_session_id(),
            };

            let row_id = db_manager.log_command(&entry).await?;

            println!(
                "Command logged: {} (Binary: {}, Session: {}, ID: {})",
                cmd, binary, entry.session_id, row_id
            );
        }
    }

    Ok(())
}
