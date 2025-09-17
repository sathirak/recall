use crate::db::CommandHistoryEntry;
use chrono::{DateTime, Local};
use chrono_humanize::HumanTime;
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
use std::io;

fn get_session_color(session_id: i64) -> Color {
    let colors = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::DarkGray,
        Color::Gray,
    ];
    
    let index = (session_id as usize) % colors.len();
    colors[index]
}

pub struct App {
    pub commands: Vec<CommandHistoryEntry>,
    pub list_state: ListState,
    pub should_quit: bool,
    pub show_help: bool,
    pub session_view: Option<i64>,
    pub all_commands: Vec<CommandHistoryEntry>,
    pub search_mode: bool,
    pub search_query: String,
    pub filtered_commands: Vec<CommandHistoryEntry>,
}

impl App {
    pub fn new(commands: Vec<CommandHistoryEntry>) -> App {
        let mut reversed_commands = commands.clone();
        reversed_commands.reverse();
        
        let mut app = App {
            all_commands: commands.clone(),
            commands: reversed_commands.clone(),
            list_state: ListState::default(),
            should_quit: false,
            show_help: false,
            session_view: None,
            search_mode: false,
            search_query: String::new(),
            filtered_commands: reversed_commands,
        };
        if !app.commands.is_empty() {
            app.list_state.select(Some(app.commands.len() - 1));
        }
        app
    }

    pub fn enter_session_view(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if let Some(cmd) = self.commands.get(selected) {
                let session_id = cmd.session_id;
                let mut session_commands: Vec<CommandHistoryEntry> = self.all_commands
                    .iter()
                    .cloned()
                    .filter(|c| c.session_id == session_id)
                    .collect();
                session_commands.reverse();
                self.commands = session_commands;
                self.session_view = Some(session_id);
                self.list_state = ListState::default();
                if !self.commands.is_empty() {
                    self.list_state.select(Some(self.commands.len() - 1));
                }
            }
        }
    }

    pub fn exit_session_view(&mut self) {
        let mut reversed_commands = self.all_commands.clone();
        reversed_commands.reverse();
        self.commands = reversed_commands.clone();
        self.filtered_commands = reversed_commands;
        self.session_view = None;
        self.search_mode = false;
        self.search_query.clear();
        self.list_state = ListState::default();
        if !self.commands.is_empty() {
            self.list_state.select(Some(self.commands.len() - 1));
        }
    }

    pub fn toggle_search(&mut self) {
        self.search_mode = !self.search_mode;
        if !self.search_mode {
            self.search_query.clear();
            self.apply_search_filter();
        }
    }

    pub fn add_search_char(&mut self, c: char) {
        if self.search_mode {
            self.search_query.push(c);
            self.apply_search_filter();
        }
    }

    pub fn remove_search_char(&mut self) {
        if self.search_mode && !self.search_query.is_empty() {
            self.search_query.pop();
            self.apply_search_filter();
        }
    }

    pub fn apply_search_filter(&mut self) {
        if self.search_query.is_empty() {
            let mut reversed_commands = self.all_commands.clone();
            reversed_commands.reverse();
            self.commands = reversed_commands.clone();
            self.filtered_commands = reversed_commands;
        } else {
            let query = self.search_query.to_lowercase();
            let filtered: Vec<CommandHistoryEntry> = self.all_commands
                .iter()
                .filter(|cmd| cmd.command.to_lowercase().contains(&query))
                .cloned()
                .collect();
            let mut reversed_filtered = filtered.clone();
            reversed_filtered.reverse();
            self.commands = reversed_filtered.clone();
            self.filtered_commands = reversed_filtered;
        }
        self.list_state = ListState::default();
        if !self.commands.is_empty() {
            self.list_state.select(Some(self.commands.len() - 1));
        }
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
                    0
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

pub fn run_tui(
    commands: Vec<CommandHistoryEntry>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                    if !app.search_mode {
                        app.should_quit = true;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !app.search_mode {
                        app.next();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !app.search_mode {
                        app.previous();
                    }
                }
                KeyCode::Enter => {
                    if app.session_view.is_none() && !app.search_mode {
                        app.enter_session_view();
                    }
                }
                KeyCode::Backspace => {
                    if app.search_mode {
                        app.remove_search_char();
                    }
                }
                KeyCode::Char(c) => {
                    if app.search_mode {
                        app.add_search_char(c);
                    } else if c == 'q' && !app.search_mode {
                        app.should_quit = true;
                    } else if (c == 'h' || c == '?') && !app.search_mode {
                        app.toggle_help();
                    } else if c == '/' && app.session_view.is_none() && !app.search_mode {
                        app.toggle_search();
                    } else if c == 'j' && !app.search_mode {
                        app.next();
                    } else if c == 'k' && !app.search_mode {
                        app.previous();
                    } else if c == 'b' {
                        if app.show_help {
                            app.show_help = false;
                        } else if app.search_mode {
                            app.toggle_search();
                        } else if app.session_view.is_some() {
                            app.exit_session_view();
                        } else {
                            app.should_quit = true;
                        }
                    }
                }
                KeyCode::Esc => {
                    if app.show_help {
                        app.show_help = false;
                    } else if app.search_mode {
                        app.toggle_search();
                    } else if app.session_view.is_some() {
                        app.exit_session_view();
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
    let chunks = if app.session_view.is_none() {
        // All commands page with search bar
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.area())
    };

    let title_text = if let Some(_session_id) = app.session_view {
        "Session".to_string()
    } else {
        "All Commands".to_string()
    };
    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = if app.session_view.is_some() {
        // Session view: show pwd and binary
        app.commands
            .iter()
            .map(|cmd| {
                let local_time: DateTime<Local> = cmd.timestamp.into();
                let human_time = HumanTime::from(local_time);
                let time_str = human_time.to_string();
                
                let display_time = if time_str.contains("seconds ago") || time_str == "now" {
                    "now".to_string()
                } else {
                    time_str
                };

                let session_color = get_session_color(cmd.session_id);
                
                let content = vec![
                    Line::from(vec![
                        Span::styled(
                            "● ",
                            Style::default()
                                .fg(session_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            &cmd.command,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Binary: ", Style::default().fg(Color::Gray)),
                        Span::styled(&cmd.binary, Style::default().fg(Color::Yellow)),
                        Span::styled(" • PWD: ", Style::default().fg(Color::Gray)),
                        Span::styled(&cmd.pwd, Style::default().fg(Color::Blue)),
                    ]),
                    Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(display_time, Style::default().fg(Color::Green)),
                    ]),
                    Line::from(""),
                ];
                ListItem::new(content)
            })
            .collect()
    } else {
        // All commands view: compact single-line format
        app.commands
            .iter()
            .map(|cmd| {
                let local_time: DateTime<Local> = cmd.timestamp.into();
                let human_time = HumanTime::from(local_time);
                let time_str = human_time.to_string();
                
                let display_time = if time_str.contains("seconds ago") || time_str == "now" {
                    "now".to_string()
                } else {
                    time_str
                };

                let session_color = get_session_color(cmd.session_id);
                
                let content = vec![
                    Line::from(vec![
                        Span::styled(
                            "● ",
                            Style::default()
                                .fg(session_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(display_time, Style::default().fg(session_color)),
                        Span::styled(" → ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            &cmd.command,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                ];
                ListItem::new(content)
            })
            .collect()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("→ ");

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // Instructions
    let instructions = if app.commands.is_empty() {
        Paragraph::new("No commands found")
    } else if app.session_view.is_some() {
        Paragraph::new("Viewing session • ↑/↓ or j/k to navigate • b/Esc to go back • q to quit")
    } else if app.search_mode {
        Paragraph::new("Search mode • Type to search • Esc to exit search • q to quit")
    } else {
        Paragraph::new("Use ↑/↓ or j/k to navigate • Enter to view session • / to search • h/? for help • q/Esc to quit")
    };

    let instructions = instructions
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);

    // Search bar (only on all commands page)
    if app.session_view.is_none() {
        let search_text = if app.search_mode {
            format!("Search: {}_", app.search_query)
        } else if app.search_query.is_empty() {
            "Press / to search...".to_string()
        } else {
            format!("Search: {}", app.search_query)
        };
        
        let search_title = if app.search_mode { "Search (active)" } else { "Search" };
        let search_style = if app.search_mode { 
            Style::default().fg(Color::White) 
        } else { 
            Style::default().fg(Color::DarkGray) 
        };
        
        let search_bar = Paragraph::new(search_text)
            .style(search_style)
            .block(Block::default().borders(Borders::ALL).title(search_title));
        f.render_widget(search_bar, chunks[3]);
    }

    // Help popup (unchanged)
    if app.show_help {
        let help_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Navigation:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  ↑/k        Move up"),
            Line::from("  ↓/j        Move down"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Actions:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Enter      View session details"),
            Line::from("  /          Search commands"),
            Line::from("  h/?        Show/hide this help"),
            Line::from("  b/Esc      Go back/quit"),
            Line::from("  q          Quit application"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Info:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Commands are sorted by recency (newest at bottom)"),
            Line::from("  Colored circles (●) represent different sessions"),
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

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
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
