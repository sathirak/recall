use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use chrono::{DateTime, Local};
use crate::db::CommandHistoryEntry;

pub struct App {
    pub commands: Vec<CommandHistoryEntry>,
    pub list_state: ListState,
    pub should_quit: bool,
    pub show_help: bool,
}

impl App {
    pub fn new(commands: Vec<CommandHistoryEntry>) -> App {
        let mut app = App {
            commands,
            list_state: ListState::default(),
            should_quit: false,
            show_help: false,
        };
        
        if !app.commands.is_empty() {
            app.list_state.select(Some(0));
        }
        
        app
    }

    pub fn next(&mut self) {
        if self.commands.is_empty() {
            return;
        }
        
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.commands.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.commands.is_empty() {
            return;
        }
        
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.commands.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

pub fn run_tui(commands: Vec<CommandHistoryEntry>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new(commands);
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    app.should_quit = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.next();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.previous();
                }
                KeyCode::Char('h') | KeyCode::Char('?') => {
                    app.toggle_help();
                }
                KeyCode::Esc => {
                    if app.show_help {
                        app.show_help = false;
                    } else {
                        app.should_quit = true;
                    }
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ].as_ref())
        .split(f.area());

    let title = Paragraph::new("History")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = app
        .commands
        .iter()
        .map(|cmd| {
            let local_time: DateTime<Local> = cmd.timestamp.into();
            let time_str = local_time.format("%Y-%m-%d %H:%M:%S").to_string();
            
            let content = vec![
                Line::from(vec![
                    Span::styled(&cmd.command, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("Time: ", Style::default().fg(Color::Gray)),
                    Span::styled(time_str, Style::default().fg(Color::Green)),
                    Span::styled(" | PWD: ", Style::default().fg(Color::Gray)),
                    Span::styled(&cmd.pwd, Style::default().fg(Color::Blue)),
                    Span::styled(" | User: ", Style::default().fg(Color::Gray)),
                    Span::styled(&cmd.user, Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
            ];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Commands"))
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("→ ");
    
    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // Instructions
    let instructions = if app.commands.is_empty() {
        Paragraph::new("No commands found in history")
    } else {
        Paragraph::new("Use ↑/↓ or j/k to navigate • h/? for help • q/Esc to quit")
    };
    
    let instructions = instructions
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);

    // Help popup
    if app.show_help {
        let help_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  ↑/k        Move up"),
            Line::from("  ↓/j        Move down"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Actions:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  h/?        Show/hide this help"),
            Line::from("  q/Esc      Quit application"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Info:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  Commands are sorted by recency (newest first)"),
            Line::from("  Session ID shows first 8 characters"),
            Line::from(""),
        ];

        let help_paragraph = Paragraph::new(help_text)
            .block(Block::default().title("Help").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black));

        let area = centered_rect(60, 70, f.area());
        f.render_widget(Clear, area);
        f.render_widget(help_paragraph, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
