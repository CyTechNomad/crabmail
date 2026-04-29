use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::action::Action;
use crate::components::Component;
use crate::config::Account;
use zeroize::Zeroize;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    Name,
    Email,
    ImapHost,
    ImapPort,
    SmtpHost,
    SmtpPort,
    Password,
    Confirm,
}

const STEPS: [Step; 8] = [
    Step::Name,
    Step::Email,
    Step::ImapHost,
    Step::ImapPort,
    Step::SmtpHost,
    Step::SmtpPort,
    Step::Password,
    Step::Confirm,
];

pub struct SetupWizard {
    pub active: bool,
    step: usize,
    name: String,
    email: String,
    imap_host: String,
    imap_port: String,
    smtp_host: String,
    smtp_port: String,
    password: String,
}

impl SetupWizard {
    pub fn new() -> Self {
        Self {
            active: false,
            step: 0,
            name: String::new(),
            email: String::new(),
            imap_host: String::new(),
            imap_port: "993".to_string(),
            smtp_host: String::new(),
            smtp_port: "587".to_string(),
            password: String::new(),
        }
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.step = 0;
    }

    pub fn prefill(&mut self, account: &Account) {
        self.active = true;
        self.step = 0;
        self.name = account.name.clone();
        self.email = account.email.clone();
        self.imap_host = account.imap_host.clone();
        self.imap_port = account.imap_port.to_string();
        self.smtp_host = account.smtp_host.clone();
        self.smtp_port = account.smtp_port.to_string();
        self.password.clear();
    }

    fn current_field_mut(&mut self) -> &mut String {
        match STEPS[self.step] {
            Step::Name => &mut self.name,
            Step::Email => &mut self.email,
            Step::ImapHost => &mut self.imap_host,
            Step::ImapPort => &mut self.imap_port,
            Step::SmtpHost => &mut self.smtp_host,
            Step::SmtpPort => &mut self.smtp_port,
            Step::Password => &mut self.password,
            Step::Confirm => &mut self.name, // unused
        }
    }

    pub fn build_account(&self) -> Account {
        Account {
            name: self.name.clone(),
            email: self.email.clone(),
            imap_host: self.imap_host.clone(),
            imap_port: self.imap_port.parse().unwrap_or(993),
            smtp_host: self.smtp_host.clone(),
            smtp_port: self.smtp_port.parse().unwrap_or(465),
            use_tls: true,
        }
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn clear_password(&mut self) {
        self.password.zeroize();
    }
}

impl Component for SetupWizard {
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        if !self.active {
            return Action::Noop;
        }
        match key.code {
            KeyCode::Esc => {
                self.active = false;
                Action::EnterNormal
            }
            KeyCode::Enter => {
                if STEPS[self.step] == Step::Confirm {
                    self.active = false;
                    return Action::SetStatus("Account configured!".to_string());
                }
                if self.step + 1 < STEPS.len() {
                    self.step += 1;
                }
                Action::Noop
            }
            KeyCode::BackTab => {
                if self.step > 0 {
                    self.step -= 1;
                }
                Action::Noop
            }
            KeyCode::Backspace => {
                if STEPS[self.step] != Step::Confirm {
                    self.current_field_mut().pop();
                }
                Action::Noop
            }
            KeyCode::Char(c) => {
                if STEPS[self.step] != Step::Confirm {
                    self.current_field_mut().push(c);
                }
                Action::Noop
            }
            _ => Action::Noop,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.active {
            return;
        }
        let outer = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" 🦀 Crabmail Setup ");

        let inner = outer.inner(area);
        frame.render_widget(outer, area);

        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(inner);

        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                "Welcome to Crabmail! ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Let's set up your first account."),
        ]));
        frame.render_widget(title, chunks[0]);

        let fields = [
            ("Account Name", &self.name, false),
            ("Email", &self.email, false),
            ("IMAP Host", &self.imap_host, false),
            ("IMAP Port", &self.imap_port, false),
            ("SMTP Host", &self.smtp_host, false),
            ("SMTP Port", &self.smtp_port, false),
            ("Password", &self.password, true),
        ];

        let mut lines = vec![];
        for (i, (label, value, is_secret)) in fields.iter().enumerate() {
            let marker = if i == self.step { "▸ " } else { "  " };
            let display = if *is_secret {
                "*".repeat(value.len())
            } else {
                value.to_string()
            };
            let style = if i == self.step {
                Style::default().fg(Color::Cyan)
            } else if i < self.step {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(Span::styled(
                format!("{marker}{label}: {display}"),
                style,
            )));
        }

        if STEPS[self.step] == Step::Confirm {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  Press Enter to save and connect, Esc to cancel.",
                Style::default().fg(Color::Yellow),
            )));
        }

        let body = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(body, chunks[1]);

        let hint = Paragraph::new("Enter: next  Shift+Tab: back  Esc: quit")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hint, chunks[2]);
    }
}
