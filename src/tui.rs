use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::io;

use crate::{location::LocationLookup, lookup::Lookup, parser::P2000Message};

pub struct AppState {
    pub messages: Vec<P2000Message>,
    pub selected_index: usize,
    pub search_query: String,
    pub search_mode: bool,
    pub filtered_indices: Vec<usize>,
    pub scroll_offset: usize,
    pub list_height: u16,
}

impl AppState {
    pub fn new(messages: Vec<P2000Message>) -> Self {
        let filtered_indices: Vec<usize> = (0..messages.len()).collect();
        AppState {
            messages,
            selected_index: 0,
            search_query: String::new(),
            search_mode: false,
            filtered_indices,
            scroll_offset: 0,
            list_height: 10,
        }
    }

    pub fn set_list_height(&mut self, height: u16) {
        self.list_height = height.saturating_sub(2) as u16;
    }

    fn ensure_selected_visible(&mut self) {
        let visible_end = self.scroll_offset + self.list_height as usize;
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= visible_end {
            self.scroll_offset = self.selected_index.saturating_sub(self.list_height as usize - 1);
        }
    }

    pub fn selected_message(&self) -> Option<&P2000Message> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|idx| self.messages.get(*idx))
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.filtered_indices.len().saturating_sub(1) {
            self.selected_index += 1;
            self.ensure_selected_visible();
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.ensure_selected_visible();
        }
    }

    pub fn filter_messages(&mut self) {
        let query = self.search_query.to_lowercase();
        self.filtered_indices = (0..self.messages.len())
            .filter(|&i| {
                let msg = &self.messages[i];
                msg.content.to_lowercase().contains(&query)
                    || msg.priority.as_ref().map_or(false, |p| p.to_lowercase().contains(&query))
                    || msg.location.to_lowercase().contains(&query)
            })
            .collect();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn add_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.filter_messages();
    }

    pub fn remove_search_char(&mut self) {
        self.search_query.pop();
        self.filter_messages();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.filter_messages();
    }
}

pub struct App {
    pub state: AppState,
    lookup: Lookup,
    location_lookup: LocationLookup,
}

impl App {
    pub fn new(messages: Vec<P2000Message>, lookup: Lookup, location_lookup: LocationLookup) -> Self {
        App {
            state: AppState::new(messages),
            lookup,
            location_lookup,
        }
    }

    pub fn handle_input(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('s') => {
                self.state.search_mode = !self.state.search_mode;
                if !self.state.search_mode {
                    self.state.search_query.clear();
                    self.state.filter_messages();
                }
            }
            KeyCode::Char(c) if self.state.search_mode => {
                self.state.add_search_char(c);
            }
            KeyCode::Backspace if self.state.search_mode => {
                self.state.remove_search_char();
            }
            KeyCode::Enter if self.state.search_mode => {
                self.state.search_mode = false;
            }
            KeyCode::Up => self.state.move_up(),
            KeyCode::Down => self.state.move_down(),
            KeyCode::PageUp => {
                for _ in 0..10 {
                    self.state.move_up();
                }
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
                    self.state.move_down();
                }
            }
            _ => {}
        }
        false
    }

    pub fn draw(&mut self, f: &mut ratatui::Frame) {
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(1)
            .constraints([
                ratatui::layout::Constraint::Min(1),
                ratatui::layout::Constraint::Length(8),
                ratatui::layout::Constraint::Length(3),
            ])
            .split(f.area());

        // Capture the list area height
        self.state.set_list_height(chunks[0].height);

        // Message list - only render visible items based on scroll offset
        let items: Vec<ListItem> = self
            .state
            .filtered_indices
            .iter()
            .skip(self.state.scroll_offset)
            .take(self.state.list_height as usize)
            .enumerate()
            .map(|(display_i, &msg_idx)| {
                let actual_i = display_i + self.state.scroll_offset;
                let msg = &self.state.messages[msg_idx];
                let is_selected = actual_i == self.state.selected_index;
                let style = if is_selected {
                    ratatui::style::Style::default()
                        .bg(ratatui::style::Color::DarkGray)
                        .fg(ratatui::style::Color::White)
                } else {
                    ratatui::style::Style::default()
                };

                let priority_color = match msg.priority.as_deref() {
                    Some(p) if p.starts_with('A') => ratatui::style::Color::Red,
                    Some(p) if p.starts_with('P') => ratatui::style::Color::Yellow,
                    _ => ratatui::style::Color::White,
                };

                let line = if let Some(priority) = &msg.priority {
                    Line::from(vec![
                        Span::styled(
                            format!("{:>3}", priority),
                            ratatui::style::Style::default().fg(priority_color),
                        ),
                        Span::raw(" | "),
                        Span::raw(msg.timestamp.format("%H:%M:%S").to_string()),
                        Span::raw(" | "),
                        Span::raw(msg.content.clone()),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw(msg.timestamp.format("%H:%M:%S").to_string()),
                        Span::raw(" | "),
                        Span::raw(msg.content.clone()),
                    ])
                };

                ListItem::new(line).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("P2000 Messages"));
        f.render_widget(list, chunks[0]);

        // Detail view
        if let Some(msg) = self.state.selected_message() {
            let capcodes_display = self
                .format_capcodes(msg)
                .unwrap_or_else(|| msg.capcodes.join(", "));

            let abbrev_display = self.format_abbreviations(msg);

            // Search for place names in the full message (content + location)
            let full_text = format!("{} {}", msg.location, msg.content);
            let location_display = self
                .location_lookup
                .find_location_by_text(&full_text)
                .map(|found| self.location_lookup.format_found_location(&found))
                .unwrap_or_else(|| msg.location.clone());

            let detail_text = format!(
                "Priority: {:?} | Code: {:?} | Location: {}\n\
                Timestamp: {} | Type: {} | Freq: {}\n\
                Radio Addr: {} | Capcodes: {}\n\
                Abbreviations: {}\n\
                Content: {}",
                msg.priority,
                msg.incident_code,
                location_display,
                msg.timestamp.format("%Y-%m-%d %H:%M:%S"),
                msg.message_type,
                msg.frequency,
                msg.radio_address,
                capcodes_display,
                abbrev_display,
                msg.content
            );

            let detail = Paragraph::new(detail_text)
                .block(Block::default().borders(Borders::ALL).title("Details"))
                .wrap(Wrap { trim: true });
            f.render_widget(detail, chunks[1]);
        }

        // Help/search bar
        let help_text = if self.state.search_mode {
            format!(
                "SEARCH: {} (Enter to exit, Backspace to delete)",
                self.state.search_query
            )
        } else {
            "↑/↓: Navigate | PageUp/Down: Jump | s: Search | q: Quit".to_string()
        };

        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));
        f.render_widget(help, chunks[2]);
    }

    fn format_capcodes(&self, msg: &P2000Message) -> Option<String> {
        let mut parts = Vec::new();
        for code in &msg.capcodes {
            if let Some(info) = self.lookup.resolve_capcode(code) {
                let mut segment = String::new();
                if !info.description.is_empty() {
                    segment.push_str(&info.description);
                } else if !info.short.is_empty() {
                    segment.push_str(&info.short);
                }
                if !info.place.is_empty() {
                    if !segment.is_empty() {
                        segment.push_str(" - ");
                    }
                    segment.push_str(&info.place);
                }
                if segment.is_empty() {
                    segment.push_str(&info.code);
                }
                parts.push(segment);
            }
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(", "))
        }
    }

    fn format_abbreviations(&self, msg: &P2000Message) -> String {
        let mut seen = std::collections::HashSet::new();
        let mut expansions = Vec::new();

        // Pre-tokenize once and keep cleaned tokens
        let raw_tokens: Vec<String> = msg
            .content
            .split_whitespace()
            .map(|raw| raw.trim_matches(|c: char| !c.is_alphanumeric() && c != '&').to_string())
            .filter(|t| !t.is_empty())
            .collect();

        let mut i = 0;
        while i < raw_tokens.len() {
            let token = &raw_tokens[i];

            // Direct match
            if let Some(exp) = self.lookup.expand_abbreviation(token) {
                if seen.insert(token.clone()) {
                    expansions.push(format!("{}: {}", token, exp));
                }
            }

            // Combined match with next token (e.g., "P" + "1" => "P1")
            if i + 1 < raw_tokens.len() {
                let combined = format!("{}{}", token, raw_tokens[i + 1]);
                if let Some(exp) = self.lookup.expand_abbreviation(&combined) {
                    if seen.insert(combined.clone()) {
                        expansions.push(format!("{}: {}", combined, exp));
                    }
                }
            }

            i += 1;
        }

        if expansions.is_empty() {
            "-".to_string()
        } else {
            expansions.join("; ")
        }
    }
}

pub async fn run_tui(
    messages: Vec<P2000Message>,
    lookup: Lookup,
    location_lookup: LocationLookup,
) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(messages, lookup, location_lookup);
    let result = event_loop(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn event_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| app.draw(f))?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.handle_input(key.code) {
                    return Ok(());
                }
            }
        }
    }
}
