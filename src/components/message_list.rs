use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table, TableState};

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
                Row::new(vec![
                    flag.to_string(),
                    truncate(&m.from, 25),
                    truncate(&m.subject, 40),
                    truncate(&m.date, 20),
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
            Constraint::Length(20),
        ];

        let table = Table::new(rows, widths)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(" Messages "),
            )
            .header(
                Row::new(vec!["", "From", "Subject", "Date"])
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
    if s.len() > max {
        format!("{}…", &s[..max - 1])
    } else {
        s.to_string()
    }
}
