use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::action::Action;
use crate::app::Mode;
use crate::components::Component;

pub struct StatusBar {
    pub mode: Mode,
    pub account: String,
    pub mailbox: String,
    pub message_count: u32,
    pub status: String,
    pub error: String,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            account: String::new(),
            mailbox: String::new(),
            message_count: 0,
            status: String::new(),
            error: String::new(),
        }
    }
}

impl Component for StatusBar {
    fn update(&mut self, action: &Action) {
        match action {
            Action::SetStatus(s) => {
                self.status = s.clone();
                self.error.clear();
            }
            Action::SetError(e) => {
                self.error = e.clone();
                self.status.clear();
            }
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let mode_str = match self.mode {
            Mode::Normal => " NORMAL ",
            Mode::Reading => " READING ",
            Mode::Compose => " COMPOSE ",
            Mode::Search => " SEARCH ",
            Mode::Command => " COMMAND ",
            Mode::Setup => " SETUP ",
        };
        let mode_span = Span::styled(mode_str, Style::default().fg(Color::Black).bg(Color::Cyan));
        let acct = if self.account.is_empty() {
            "No account".to_string()
        } else {
            format!(" {} ", self.account)
        };
        let acct_span = Span::styled(acct, Style::default().fg(Color::White).bg(Color::DarkGray));
        let mb = if self.mailbox.is_empty() {
            String::new()
        } else {
            format!(" {} ({}) ", self.mailbox, self.message_count)
        };
        let mb_span = Span::styled(mb, Style::default().fg(Color::Gray));

        let right = if !self.error.is_empty() {
            Span::styled(
                format!(" {} ", self.error),
                Style::default().fg(Color::Red),
            )
        } else if !self.status.is_empty() {
            Span::styled(
                format!(" {} ", self.status),
                Style::default().fg(Color::Green),
            )
        } else {
            Span::raw("")
        };

        let line = Line::from(vec![mode_span, acct_span, mb_span, right]);
        let bar = Paragraph::new(line).style(Style::default().bg(Color::DarkGray));
        frame.render_widget(bar, area);
    }
}
