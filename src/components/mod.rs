pub mod command_bar;
pub mod composer;
pub mod mailbox_list;
pub mod message_list;
pub mod reader;
pub mod search;
pub mod setup_wizard;
pub mod status_bar;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::action::Action;

pub trait Component {
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        let _ = key;
        Action::Noop
    }
    fn update(&mut self, action: &Action) {
        let _ = action;
    }
    fn render(&self, frame: &mut Frame, area: Rect);
}
