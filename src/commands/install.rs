use dirs::home_dir;
use std::env;
use std::io::Write;
use std::process::exit;

pub fn install_shell_integration(shell: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home = home_dir().ok_or("Could not find home directory")?;
    let recall_path = env::current_exe()?;
    
    match shell {
        "bash" => {
            let bashrc_path = home.join(".bashrc");
            let integration_code = format!(
                r#"
# recall command logger integration
export PROMPT_COMMAND="$PROMPT_COMMAND; history -a; {} log \"$(history 1 | sed 's/^[ ]*[0-9]*[ ]*//')\" 2>/dev/null"
"#,
                recall_path.display()
            );
            
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&bashrc_path)?
                .write_all(integration_code.as_bytes())?;
            
            println!("Bash integration installed. Please run 'source ~/.bashrc' or restart your terminal.");
        }
        "zsh" => {
            let zshrc_path = home.join(".zshrc");
            let integration_code = format!(
                r#"
# recall command logger integration
preexec() {{
    {} log "$1" 2>/dev/null
}}
"#,
                recall_path.display()
            );
            
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&zshrc_path)?
                .write_all(integration_code.as_bytes())?;
            
            println!("Zsh integration installed. Please run 'source ~/.zshrc' or restart your terminal.");
        }
        "fish" => {
            let fish_config_dir = home.join(".config/fish");
            std::fs::create_dir_all(&fish_config_dir)?;
            let fish_config_path = fish_config_dir.join("config.fish");
            let integration_code = format!(
                r#"
# recall command logger integration
function recall_log_command --on-event fish_preexec
    {} log "$argv" 2>/dev/null &
end
"#,
                recall_path.display()
            );
            
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&fish_config_path)?
                .write_all(integration_code.as_bytes())?;
            
            println!("Fish integration installed. Please restart your terminal.");
        }
        _ => {
            eprintln!("Unsupported shell: {}. Supported shells: bash, zsh, fish", shell);
            exit(1);
        }
    }
    
    Ok(())
}
