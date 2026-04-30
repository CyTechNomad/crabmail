use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

use crate::action::Action;
use crate::components::Component;
use crate::theme::Theme;

pub struct CommandBar {
    pub input: String,
    pub active: bool,
}

impl CommandBar {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            active: false,
        }
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.input.clear();
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.input.clear();
    }
}

impl Component for CommandBar {
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        if !self.active {
            return Action::Noop;
        }
        match key.code {
            KeyCode::Esc => {
                self.deactivate();
                Action::CancelCommand
            }
            KeyCode::Enter => {
                let cmd = self.input.clone();
                self.deactivate();
                Action::ExecuteCommand(cmd)
            }
            KeyCode::Backspace => {
                self.input.pop();
                Action::Noop
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                Action::Noop
            }
            _ => Action::Noop,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if !self.active {
            return;
        }
        let p = Paragraph::new(format!(":{}", self.input))
            .style(Style::default().fg(theme.text).bg(theme.bar_bg));
        frame.render_widget(p, area);
        frame.set_cursor_position((area.x + 1 + self.input.len() as u16, area.y));
    }
}

pub fn parse_command(input: &str) -> Action {
    let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
    match parts.first().copied() {
        Some("q" | "quit") => Action::Quit,
        Some("account") => {
            if let Some(name) = parts.get(1) {
                Action::SwitchAccount(name.to_string())
            } else {
                Action::SetError("Usage: :account <name>".to_string())
            }
        }
        Some("help") => Action::SetStatus(
            "j/k:nav l/h:focus Enter:open i:compose /:search ::cmd q:back".to_string(),
        ),
        Some("edit-account") => Action::EditAccount,
        Some("add-account") => Action::AddAccount,
        Some("theme") => {
            if let Some(name) = parts.get(1) {
                Action::SetTheme(name.to_string())
            } else {
                Action::SetStatus(format!(
                    "Themes: {}",
                    crate::theme::Theme::available().join(", ")
                ))
            }
        }
        Some(other) => Action::SetError(format!("Unknown command: {other}")),
        None => Action::Noop,
    }
}
