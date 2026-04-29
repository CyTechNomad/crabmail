use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout};
use std::time::{Duration, Instant};

use crate::action::Action;
use crate::auth;
use crate::components::Component;
use crate::components::command_bar::{self, CommandBar};
use crate::components::composer::Composer;
use crate::components::mailbox_list::MailboxList;
use crate::components::message_list::MessageList;
use crate::components::reader::Reader;
use crate::components::search::Search;
use crate::components::setup_wizard::SetupWizard;
use crate::components::status_bar::StatusBar;
use crate::config::Config;
use crate::imap_client::ImapClient;
use crate::mail;
use crate::smtp_client;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Reading,
    Compose,
    Search,
    Command,
    Setup,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Focus {
    Mailboxes,
    Messages,
}

pub struct App {
    mode: Mode,
    focus: Focus,
    config: Config,
    active_account: Option<usize>,
    cached_password: Option<String>,
    imap: Option<ImapClient>,
    // Components
    mailbox_list: MailboxList,
    message_list: MessageList,
    reader: Reader,
    composer: Composer,
    search: Search,
    command_bar: CommandBar,
    status_bar: StatusBar,
    setup_wizard: SetupWizard,
    pending_confirm: Option<Action>,
    should_quit: bool,
    last_refresh: Instant,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            mode: Mode::Normal,
            focus: Focus::Mailboxes,
            config,
            active_account: None,
            cached_password: None,
            imap: None,
            mailbox_list: MailboxList::new(),
            message_list: MessageList::new(),
            reader: Reader::new(),
            composer: Composer::new(),
            search: Search::new(),
            command_bar: CommandBar::new(),
            status_bar: StatusBar::new(),
            setup_wizard: SetupWizard::new(),
            pending_confirm: None,
            should_quit: false,
            last_refresh: Instant::now(),
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        if self.config.accounts.is_empty() {
            self.mode = Mode::Setup;
            self.setup_wizard.activate();
        } else {
            self.connect_account(0).await;
        }

        loop {
            self.sync_component_state();
            terminal.draw(|frame| self.render(frame))?;

            if self.should_quit {
                if let Some(imap) = &mut self.imap {
                    let _ = imap.logout().await;
                }
                break;
            }

            if event::poll(Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
            {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                let action = self.handle_key(key);
                self.process_action(action).await;
            } else if let Some(secs) = self.config.auto_refresh_seconds {
                if secs > 0 && self.last_refresh.elapsed() >= Duration::from_secs(secs) {
                    self.last_refresh = Instant::now();
                    self.process_action(Action::RefreshMailbox).await;
                }
            }
        }
        Ok(())
    }

    fn sync_component_state(&mut self) {
        self.status_bar.mode = self.mode;
        if let Some(idx) = self.active_account
            && let Some(acct) = self.config.accounts.get(idx)
        {
            self.status_bar.account = acct.name.clone();
        }
        self.mailbox_list.focused = self.focus == Focus::Mailboxes && self.mode == Mode::Normal;
        self.message_list.focused = self.focus == Focus::Messages && self.mode == Mode::Normal;
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        // Confirm overlay — intercept next keypress without changing mode
        if self.pending_confirm.is_some() {
            return if matches!(key.code, KeyCode::Char('y')) {
                self.pending_confirm.take().unwrap()
            } else {
                self.pending_confirm = None;
                self.status_bar.status = "Cancelled".to_string();
                Action::Noop
            };
        }

        match self.mode {
            Mode::Setup => self.setup_wizard.handle_key_event(key),
            Mode::Command => self.command_bar.handle_key_event(key),
            Mode::Search => self.search.handle_key_event(key),
            Mode::Compose => self.composer.handle_key_event(key),
            Mode::Reading => self.reader.handle_key_event(key),
            Mode::Normal => self.handle_normal_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char(':') => Action::StartCommand,
            KeyCode::Char('/') => Action::StartSearch,
            KeyCode::Char('i') => Action::StartCompose,
            KeyCode::Char('r') => Action::RefreshMailbox,
            KeyCode::Char('h') => Action::FocusMailboxes,
            KeyCode::Char('l') => Action::FocusMessages,
            _ => match self.focus {
                Focus::Mailboxes => self.mailbox_list.handle_key_event(key),
                Focus::Messages => self.message_list.handle_key_event(key),
            },
        }
    }

    async fn process_action(&mut self, action: Action) {
        // Let status bar see all actions
        self.status_bar.update(&action);

        match action {
            Action::Quit => self.should_quit = true,
            Action::Noop | Action::Tick => {}
            Action::FocusMailboxes => self.focus = Focus::Mailboxes,
            Action::FocusMessages => {
                if !self.message_list.messages.is_empty() {
                    self.focus = Focus::Messages;
                }
            }
            Action::SelectMailbox(name) => {
                self.focus = Focus::Messages;
                self.select_mailbox(&name).await;
            }
            Action::OpenMessage(uid) => {
                self.open_message(uid).await;
            }
            Action::CloseReader => {
                self.mode = Mode::Normal;
                self.reader.close();
            }
            Action::StartCompose => {
                self.mode = Mode::Compose;
                self.composer.clear();
            }
            Action::StartReply => {
                if let Some(msg) = &self.reader.message {
                    let to = msg.from.clone();
                    let subject = if msg.subject.starts_with("Re:") {
                        msg.subject.clone()
                    } else {
                        format!("Re: {}", msg.subject)
                    };
                    let body = format!(
                        "\n\nOn {}, {} wrote:\n{}",
                        msg.date,
                        msg.from,
                        msg.display_body(80)
                            .lines()
                            .map(|l| format!("> {l}"))
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                    self.composer.prefill(&to, &subject, &body);
                    self.mode = Mode::Compose;
                }
            }
            Action::StartForward => {
                if let Some(msg) = &self.reader.message {
                    let subject = if msg.subject.starts_with("Fwd:") {
                        msg.subject.clone()
                    } else {
                        format!("Fwd: {}", msg.subject)
                    };
                    let body = format!(
                        "\n\n---------- Forwarded message ----------\nFrom: {}\nDate: {}\nSubject: {}\n\n{}",
                        msg.from,
                        msg.date,
                        msg.subject,
                        msg.display_body(80)
                    );
                    self.composer.prefill("", &subject, &body);
                    self.mode = Mode::Compose;
                }
            }
            Action::SendMessage => {
                self.send_email().await;
                self.mode = Mode::Normal;
            }
            Action::CancelCompose => {
                self.mode = Mode::Normal;
            }
            Action::StartSearch => {
                self.mode = Mode::Search;
                self.search.activate();
            }
            Action::ExecuteSearch(query) => {
                self.mode = Mode::Normal;
                self.execute_search(&query).await;
            }
            Action::ClearSearch => {
                self.mode = Mode::Normal;
                // Re-fetch current mailbox
                if let Some(name) = self.mailbox_list.selected_name() {
                    self.select_mailbox(&name).await;
                }
            }
            Action::StartCommand => {
                self.mode = Mode::Command;
                self.command_bar.activate();
            }
            Action::ExecuteCommand(cmd) => {
                self.mode = Mode::Normal;
                let action = command_bar::parse_command(&cmd);
                Box::pin(self.process_action(action)).await;
            }
            Action::CancelCommand => {
                self.mode = Mode::Normal;
            }
            Action::SwitchAccount(name) => {
                self.switch_account(&name).await;
            }
            Action::EditAccount => {
                if let Some(idx) = self.active_account {
                    let account = self.config.accounts[idx].clone();
                    self.setup_wizard.prefill(&account);
                    self.mode = Mode::Setup;
                } else {
                    self.status_bar.error = "No active account to edit".to_string();
                }
            }
            Action::AddAccount => {
                self.setup_wizard.activate();
                self.mode = Mode::Setup;
            }
            Action::DeleteMessage(uid) => {
                self.delete_message(uid).await;
            }
            Action::ConfirmDelete(uid) => {
                self.pending_confirm = Some(Action::DeleteMessage(uid));
                self.status_bar.status = "Delete message? (y/n)".to_string();
            }
            Action::EnterNormal => {
                self.mode = Mode::Normal;
            }
            Action::RefreshMailbox => {
                if let Some(name) = self.mailbox_list.selected_name() {
                    self.last_refresh = Instant::now();
                    self.status_bar.status = "Refreshing…".to_string();
                    self.select_mailbox(&name).await;
                }
            }
            Action::SetStatus(s) => {
                // Setup wizard completion
                if self.setup_wizard.active || self.mode == Mode::Setup {
                    self.finish_setup().await;
                }
                self.status_bar.status = s;
            }
            Action::SetError(_) | Action::Resize(_, _) | Action::Key(_) => {}
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        if self.mode == Mode::Setup {
            self.setup_wizard.render(frame, area);
            return;
        }

        if self.mode == Mode::Compose {
            let chunks = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);
            self.composer.render(frame, chunks[0]);
            self.status_bar.render(frame, chunks[1]);
            return;
        }

        // Main layout: status bar at bottom, optional search/command at bottom
        let has_overlay = self.search.active || self.command_bar.active;
        let main_chunks = if has_overlay {
            Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(area)
        } else {
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area)
        };

        // Main content area
        let content_area = main_chunks[0];

        if self.mode == Mode::Reading {
            self.reader.render(frame, content_area);
        } else {
            // Two-pane: mailboxes | messages
            let panes = Layout::horizontal([Constraint::Length(25), Constraint::Fill(1)])
                .split(content_area);
            self.mailbox_list.render(frame, panes[0]);
            self.message_list.render(frame, panes[1]);
        }

        // Overlay (search or command)
        if has_overlay {
            if self.search.active {
                self.search.render(frame, main_chunks[1]);
            } else {
                self.command_bar.render(frame, main_chunks[1]);
            }
            self.status_bar.render(frame, main_chunks[2]);
        } else {
            self.status_bar.render(frame, main_chunks[1]);
        }
    }

    // --- Async operations ---

    async fn connect_account(&mut self, idx: usize) {
        let Some(account) = self.config.accounts.get(idx) else {
            self.status_bar.error = "Account not found".to_string();
            return;
        };
        let password = match auth::get_password(&account.name) {
            Ok(p) => p,
            Err(e) => {
                self.status_bar.error = format!("Keychain error: {e}");
                return;
            }
        };
        self.cached_password = Some(password.clone());
        self.status_bar.status = format!("Connecting to {}...", account.imap_host);
        match ImapClient::connect(account, &password).await {
            Ok(mut client) => {
                match client.list_mailboxes().await {
                    Ok(mailboxes) => {
                        self.mailbox_list.set_mailboxes(mailboxes);
                        self.status_bar.status = "Connected".to_string();
                    }
                    Err(e) => {
                        self.status_bar.error = format!("Failed to list mailboxes: {e}");
                    }
                }
                self.imap = Some(client);
                self.active_account = Some(idx);
            }
            Err(e) => {
                self.status_bar.error = format!("Connection failed: {e}");
            }
        }
    }

    async fn select_mailbox(&mut self, name: &str) {
        let Some(imap) = &mut self.imap else { return };
        match imap.select_mailbox(name).await {
            Ok(info) => {
                self.status_bar.mailbox = info.name;
                self.status_bar.message_count = info.exists;
                match imap.fetch_headers(50).await {
                    Ok(msgs) => self.message_list.set_messages(msgs),
                    Err(e) => self.status_bar.error = format!("Fetch error: {e}"),
                }
            }
            Err(e) => self.status_bar.error = format!("Select error: {e}"),
        }
    }

    async fn open_message(&mut self, uid: u32) {
        let Some(imap) = &mut self.imap else { return };
        match imap.fetch_raw_message(uid).await {
            Ok(raw) => match mail::parse_message(&raw) {
                Ok(parsed) => {
                    self.reader.open(uid, parsed);
                    self.mode = Mode::Reading;
                }
                Err(e) => self.status_bar.error = format!("Parse error: {e}"),
            },
            Err(e) => self.status_bar.error = format!("Fetch error: {e}"),
        }
    }

    async fn send_email(&mut self) {
        let Some(idx) = self.active_account else {
            self.status_bar.error = "No active account".to_string();
            return;
        };
        let account = self.config.accounts[idx].clone();
        let Some(password) = &self.cached_password else {
            self.status_bar.error = "No cached password".to_string();
            return;
        };
        let password = password.clone();
        match smtp_client::send_email(
            &account,
            &password,
            &self.composer.to,
            &self.composer.subject,
            &self.composer.body,
        )
        .await
        {
            Ok(()) => {
                self.status_bar.status = "Message sent!".to_string();
                self.composer.clear();
            }
            Err(e) => self.status_bar.error = format!("Send failed: {e}"),
        }
    }

    async fn delete_message(&mut self, uid: u32) {
        let Some(imap) = &mut self.imap else { return };
        match imap.delete_message(uid).await {
            Ok(()) => {
                self.status_bar.status = "Message deleted".to_string();
                if self.mode == Mode::Reading {
                    self.mode = Mode::Normal;
                    self.reader.close();
                }
                // Refresh current mailbox
                if let Some(name) = self.mailbox_list.selected_name() {
                    self.select_mailbox(&name).await;
                }
            }
            Err(e) => self.status_bar.error = format!("Delete failed: {e}"),
        }
    }

    async fn execute_search(&mut self, query: &str) {
        let Some(imap) = &mut self.imap else { return };
        match imap.search(query).await {
            Ok(uids) => {
                if uids.is_empty() {
                    self.status_bar.status = "No results".to_string();
                    self.message_list.set_messages(vec![]);
                    return;
                }
                match imap.fetch_headers_by_uids(&uids).await {
                    Ok(msgs) => {
                        self.status_bar.status = format!("{} results", msgs.len());
                        self.message_list.set_messages(msgs);
                    }
                    Err(e) => self.status_bar.error = format!("Search fetch error: {e}"),
                }
            }
            Err(e) => self.status_bar.error = format!("Search error: {e}"),
        }
    }

    async fn switch_account(&mut self, name: &str) {
        if let Some(imap) = &mut self.imap {
            let _ = imap.logout().await;
        }
        self.imap = None;
        self.cached_password = None;
        self.mailbox_list.set_mailboxes(vec![]);
        self.message_list.set_messages(vec![]);

        if let Some(idx) = self.config.accounts.iter().position(|a| a.name == name) {
            self.connect_account(idx).await;
        } else {
            self.status_bar.error = format!("Account '{name}' not found");
        }
    }

    async fn finish_setup(&mut self) {
        let account = self.setup_wizard.build_account();
        let password = self.setup_wizard.password().to_string();

        if !password.is_empty() {
            if let Err(e) = auth::store_password(&account.name, &password) {
                self.status_bar.error = format!("Failed to store password: {e}");
                return;
            }
        }

        // Update existing or add new
        let idx = if let Some(pos) = self
            .config
            .accounts
            .iter()
            .position(|a| a.name == account.name)
        {
            self.config.accounts[pos] = account;
            pos
        } else {
            self.config.accounts.push(account);
            self.config.accounts.len() - 1
        };

        if let Err(e) = self.config.save() {
            self.status_bar.error = format!("Failed to save config: {e}");
            return;
        }

        self.mode = Mode::Normal;
        if let Some(imap) = &mut self.imap {
            let _ = imap.logout().await;
        }
        self.imap = None;
        self.connect_account(idx).await;
    }
}
