use crossterm::event::KeyEvent;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Noop,
    Tick,
    Resize(u16, u16),
    Key(KeyEvent),
    // Navigation
    FocusMailboxes,
    FocusMessages,
    SelectMailbox(String),
    OpenMessage(u32),
    CloseReader,
    // Compose
    StartCompose,
    StartReply,
    StartForward,
    SendMessage,
    CancelCompose,
    // Search
    StartSearch,
    ExecuteSearch(String),
    ClearSearch,
    // Command
    StartCommand,
    ExecuteCommand(String),
    CancelCommand,
    // Account
    SwitchAccount(String),
    EditAccount,
    AddAccount,
    // Status
    SetStatus(String),
    SetError(String),
    // Mode
    EnterNormal,
}
