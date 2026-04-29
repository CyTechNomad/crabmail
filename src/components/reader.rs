use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::action::Action;
use crate::components::Component;
use crate::mail::ParsedMessage;

pub struct Reader {
    pub message: Option<ParsedMessage>,
    pub uid: Option<u32>,
    pub scroll: u16,
}

impl Reader {
    pub fn new() -> Self {
        Self {
            message: None,
            uid: None,
            scroll: 0,
        }
    }

    pub fn open(&mut self, uid: u32, msg: ParsedMessage) {
        self.message = Some(msg);
        self.uid = Some(uid);
        self.scroll = 0;
    }

    pub fn close(&mut self) {
        self.message = None;
        self.uid = None;
        self.scroll = 0;
    }
}

impl Component for Reader {
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll = self.scroll.saturating_add(1);
                Action::Noop
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
                Action::Noop
            }
            KeyCode::Char('q') | KeyCode::Esc => Action::CloseReader,
            KeyCode::Char('r') => Action::StartReply,
            KeyCode::Char('f') => Action::StartForward,
            KeyCode::Char('d') => {
                if let Some(uid) = self.uid {
                    Action::ConfirmDelete(uid)
                } else {
                    Action::Noop
                }
            }
            _ => Action::Noop,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let Some(msg) = &self.message else {
            let p = Paragraph::new("No message selected")
                .block(Block::default().borders(Borders::ALL).title(" Reader "));
            frame.render_widget(p, area);
            return;
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("From: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&msg.from),
            ]),
            Line::from(vec![
                Span::styled("To: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&msg.to),
            ]),
        ];
        if !msg.cc.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Cc: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&msg.cc),
            ]));
        }
        lines.push(Line::from(vec![
            Span::styled("Subject: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&msg.subject),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Date: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&msg.date),
        ]));
        lines.push(Line::from(
            "─".repeat(area.width.saturating_sub(2) as usize),
        ));

        let body = msg.display_body(area.width.saturating_sub(2) as usize);
        for line in body.lines() {
            lines.push(Line::from(line.to_string()));
        }

        if !msg.attachments.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Attachments:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            for a in &msg.attachments {
                lines.push(Line::from(format!("  📎 {} ({}B)", a.name, a.size)));
            }
        }

        let p = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Reader [q:back r:reply f:forward d:delete] "),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        frame.render_widget(p, area);
    }
}
