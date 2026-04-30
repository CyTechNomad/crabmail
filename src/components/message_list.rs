use chrono::{DateTime, Datelike, Local};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

use crate::action::Action;
use crate::components::Component;
use crate::imap_client::MessageSummary;

pub struct MessageList {
    pub messages: Vec<MessageSummary>,
    pub state: TableState,
    pub focused: bool,
}

impl MessageList {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            state: TableState::default(),
            focused: false,
        }
    }

    pub fn set_messages(&mut self, messages: Vec<MessageSummary>) {
        self.messages = messages;
        if !self.messages.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    pub fn selected_uid(&self) -> Option<u32> {
        self.state
            .selected()
            .and_then(|i| self.messages.get(i))
            .map(|m| m.uid)
    }
}

impl Component for MessageList {
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        if !self.focused {
            return Action::Noop;
        }
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let i = self.state.selected().unwrap_or(0);
                if i + 1 < self.messages.len() {
                    self.state.select(Some(i + 1));
                }
                Action::Noop
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = self.state.selected().unwrap_or(0);
                if i > 0 {
                    self.state.select(Some(i - 1));
                }
                Action::Noop
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(uid) = self.selected_uid() {
                    Action::OpenMessage(uid)
                } else {
                    Action::Noop
                }
            }
            KeyCode::Char('h') => Action::FocusMailboxes,
            KeyCode::Char('d') => {
                if let Some(uid) = self.selected_uid() {
                    Action::ConfirmDelete(uid)
                } else {
                    Action::Noop
                }
            }
            _ => Action::Noop,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let rows: Vec<Row> = self
            .messages
            .iter()
            .map(|m| {
                let flag = if m.flags.iter().any(|f| f.contains("Seen")) {
                    " "
                } else {
                    "●"
                };
                let (date, time) = format_date(&m.date);
                Row::new(vec![
                    Cell::new(flag),
                    Cell::new(truncate(&m.from, 25)),
                    Cell::new(m.subject.clone()),
                    Cell::new(Text::from(date).alignment(Alignment::Right)),
                    Cell::new(time),
                ])
            })
            .collect();

        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let widths = [
            Constraint::Length(2),
            Constraint::Length(25),
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Length(5),
        ];

        let table = Table::new(rows, widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(" Messages "),
            )
            .header(
                Row::new(vec!["", "From", "Subject", "Date", ""])
                    .style(Style::default().fg(Color::DarkGray)),
            )
            .row_highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(table, area, &mut self.state.clone());
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let end = s.char_indices().nth(max - 1).map(|(i, _)| i).unwrap_or(s.len());
        format!("{}…", &s[..end])
    } else {
        s.to_string()
    }
}

fn format_date(raw: &str) -> (String, String) {
    if let Ok(parsed) = DateTime::parse_from_rfc2822(raw.trim()) {
        let local = parsed.with_timezone(&Local);
        let today = Local::now().date_naive();
        let date = if local.date_naive() == today {
            "Today".into()
        } else if local.date_naive().year() == today.year() {
            local.format("%b %d").to_string()
        } else {
            local.format("%Y-%m-%d").to_string()
        };
        (date, local.format("%H:%M").to_string())
    } else {
        (raw.to_string(), String::new())
    }
}
