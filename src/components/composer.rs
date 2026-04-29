use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::action::Action;
use crate::components::Component;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Field {
    To,
    Subject,
    Body,
}

pub struct Composer {
    pub to: String,
    pub subject: String,
    pub body: String,
    field: Field,
    cursor: usize,
}

impl Composer {
    pub fn new() -> Self {
        Self {
            to: String::new(),
            subject: String::new(),
            body: String::new(),
            field: Field::To,
            cursor: 0,
        }
    }

    pub fn prefill(&mut self, to: &str, subject: &str, body: &str) {
        self.to = to.to_string();
        self.subject = subject.to_string();
        self.body = body.to_string();
        self.field = Field::Body;
        self.cursor = self.body.len();
    }

    pub fn clear(&mut self) {
        self.to.clear();
        self.subject.clear();
        self.body.clear();
        self.field = Field::To;
        self.cursor = 0;
    }

    fn active_field_mut(&mut self) -> &mut String {
        match self.field {
            Field::To => &mut self.to,
            Field::Subject => &mut self.subject,
            Field::Body => &mut self.body,
        }
    }
}

impl Component for Composer {
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.clear();
                Action::CancelCompose
            }
            KeyCode::Tab | KeyCode::BackTab => {
                self.field = match (key.code, self.field) {
                    (KeyCode::Tab, Field::To) => Field::Subject,
                    (KeyCode::Tab, Field::Subject) => Field::Body,
                    (KeyCode::Tab, Field::Body) => Field::To,
                    (KeyCode::BackTab, Field::To) => Field::Body,
                    (KeyCode::BackTab, Field::Subject) => Field::To,
                    (KeyCode::BackTab, Field::Body) => Field::Subject,
                    _ => self.field,
                };
                self.cursor = self.active_field_mut().len();
                Action::Noop
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::SendMessage
            }
            KeyCode::Enter => {
                if self.field == Field::Body {
                    self.body.push('\n');
                    self.cursor = self.body.len();
                }
                Action::Noop
            }
            KeyCode::Backspace => {
                let field = self.active_field_mut();
                if !field.is_empty() {
                    field.pop();
                    self.cursor = field.len();
                }
                Action::Noop
            }
            KeyCode::Char(c) => {
                self.active_field_mut().push(c);
                self.cursor = self.active_field_mut().len();
                Action::Noop
            }
            _ => Action::Noop,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(area);

        let make_block = |title: &str, active: bool| {
            let style = if active {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Block::default()
                .borders(Borders::ALL)
                .border_style(style)
                .title(format!(" {title} "))
        };

        let to_p = Paragraph::new(self.to.as_str())
            .block(make_block("To", self.field == Field::To));
        let subj_p = Paragraph::new(self.subject.as_str())
            .block(make_block("Subject", self.field == Field::Subject));
        let body_p = Paragraph::new(self.body.as_str())
            .block(make_block("Body [C-w:send Esc:cancel]", self.field == Field::Body));

        frame.render_widget(to_p, chunks[0]);
        frame.render_widget(subj_p, chunks[1]);
        frame.render_widget(body_p, chunks[2]);

        let (cx, cy) = match self.field {
            Field::To => (chunks[0].x + 1 + self.to.len() as u16, chunks[0].y + 1),
            Field::Subject => (chunks[1].x + 1 + self.subject.len() as u16, chunks[1].y + 1),
            Field::Body => {
                let lines: Vec<&str> = self.body.lines().collect();
                let last_line = lines.last().unwrap_or(&"");
                let y = chunks[2].y + 1 + lines.len().saturating_sub(1) as u16;
                (chunks[2].x + 1 + last_line.len() as u16, y)
            }
        };
        frame.set_cursor_position((cx, cy));
    }
}
