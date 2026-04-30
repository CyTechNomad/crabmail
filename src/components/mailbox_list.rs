use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use crate::action::Action;
use crate::components::Component;
use crate::imap_client::Mailbox;
use crate::theme::Theme;

pub struct MailboxList {
    pub mailboxes: Vec<Mailbox>,
    pub state: ListState,
    pub focused: bool,
}

impl MailboxList {
    pub fn new() -> Self {
        Self {
            mailboxes: vec![],
            state: ListState::default(),
            focused: true,
        }
    }

    pub fn set_mailboxes(&mut self, mailboxes: Vec<Mailbox>) {
        self.mailboxes = mailboxes;
        if !self.mailboxes.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn selected_name(&self) -> Option<String> {
        self.state
            .selected()
            .and_then(|i| self.mailboxes.get(i))
            .map(|m| m.name.clone())
    }
}

impl Component for MailboxList {
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        if !self.focused {
            return Action::Noop;
        }
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let i = self.state.selected().unwrap_or(0);
                if i + 1 < self.mailboxes.len() {
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
                if let Some(name) = self.selected_name() {
                    Action::SelectMailbox(name)
                } else {
                    Action::Noop
                }
            }
            _ => Action::Noop,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let items: Vec<ListItem> = self
            .mailboxes
            .iter()
            .map(|m| ListItem::new(format!(" 📂 {}", m.name)))
            .collect();

        let border_style = if self.focused {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(theme.dimmed)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(" Mailboxes "),
            )
            .highlight_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ");

        frame.render_stateful_widget(list, area, &mut self.state.clone());
    }
}
