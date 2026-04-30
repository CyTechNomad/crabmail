use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::action::Action;
use crate::components::Component;
use crate::theme::Theme;

pub struct Search {
    pub input: String,
    pub active: bool,
}

impl Search {
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

impl Component for Search {
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        if !self.active {
            return Action::Noop;
        }
        match key.code {
            KeyCode::Esc => {
                self.deactivate();
                Action::ClearSearch
            }
            KeyCode::Enter => {
                let query = self.input.clone();
                self.active = false;
                if query.is_empty() {
                    Action::ClearSearch
                } else {
                    Action::ExecuteSearch(query)
                }
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
        let p = Paragraph::new(format!("/{}", self.input))
            .style(Style::default().fg(theme.search))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.search))
                    .title(" Search "),
            );
        frame.render_widget(p, area);
        frame.set_cursor_position((area.x + 2 + self.input.len() as u16, area.y + 1));
    }
}
