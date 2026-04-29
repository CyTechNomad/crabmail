# 🦀 crabmail

A terminal-based email client written in Rust. Read, compose, search, and manage
email from your terminal with a TUI powered by
[ratatui](https://github.com/ratatui/ratatui).

## Features

- **IMAP support** — connect to any IMAP server (Gmail, Outlook, Fastmail,
  self-hosted, etc.)
- **SMTP sending** — compose and send emails with STARTTLS and implicit TLS
  support
- **Multiple accounts** — configure and switch between accounts
- **Mailbox navigation** — browse folders/labels with `\Noselect` filtering
- **Message reading** — renders plain text and converts HTML emails to readable
  text
- **Compose, reply, and forward**
- **Search** — search messages within the current mailbox
- **Delete messages** — with confirmation prompt
- **Secure credential storage** — passwords stored in the OS keychain via
  [keyring](https://crates.io/crates/keyring) (macOS Keychain, Windows
  Credential Manager, Linux Secret Service)
- **Setup wizard** — guided first-run account configuration
- **File logging** — logs to `~/.config/crabmail/crabmail.log`, keeps your TUI
  clean

## Requirements

- Rust 2024 edition (1.85+)
- A working IMAP/SMTP email account

## Installation

```sh
git clone git@github.com:CyTechNomad/crabmail.git
cd crabmail
cargo build --release
```

The binary will be at `target/release/crabmail`.

## Usage

```sh
cargo run
# or after building:
./target/release/crabmail
```

On first launch, the setup wizard will walk you through adding an account.
Configuration is stored at `~/.config/crabmail/config.toml`.

### Keybindings

#### Normal Mode

| Key         | Action                                 |
| ----------- | -------------------------------------- |
| `j` / `k`   | Navigate down / up                     |
| `h` / `l`   | Focus mailboxes / messages             |
| `Enter`     | Open selected message / select mailbox |
| `i`         | Compose new email                      |
| `/`         | Search                                 |
| `:`         | Command mode                           |
| `r`         | Refresh mailbox                        |
| `d`         | Delete message (press `y` to confirm)  |
| `Esc`       | Back / cancel                          |
| `q`         | Quit                                   |

#### Reading a Message

| Key         | Action              |
| ----------- | ------------------- |
| `j` / `k`   | Scroll down / up    |
| `r`         | Reply to message    |
| `f`         | Forward message     |
| `d`         | Delete message      |
| `q` / `Esc` | Close reader        |

#### Composing

| Key           | Action                              |
| ------------- | ----------------------------------- |
| `Tab`         | Cycle fields (To → Subject → Body) |
| `Shift+Tab`   | Cycle fields backward               |
| `Ctrl+w`      | Send message                        |
| `Esc`         | Cancel compose                      |

#### Commands

| Command              | Action              |
| -------------------- | ------------------- |
| `:q` / `:quit`       | Quit                |
| `:account <name>`    | Switch account      |
| `:add-account`       | Add a new account   |
| `:edit-account`      | Edit active account |
| `:help`              | Show keybind hints  |

## Configuration

Config lives at `~/.config/crabmail/config.toml`:

```toml
# Auto-refresh mailbox every N seconds (optional, omit to disable)
auto_refresh_seconds = 120

[[accounts]]
name = "Personal"
email = "you@example.com"
imap_host = "imap.example.com"
imap_port = 993
smtp_host = "smtp.example.com"
smtp_port = 587
use_tls = true
```

Passwords are stored in your OS keychain, not in the config file.

## Project Structure

```
src/
├── main.rs          # Entry point, terminal setup, logging
├── app.rs           # Core application state and event loop
├── action.rs        # Action enum (all user/system events)
├── config.rs        # TOML config loading/saving
├── auth.rs          # Keychain credential storage
├── imap_client.rs   # IMAP connection and mailbox operations
├── smtp_client.rs   # SMTP email sending
├── mail.rs          # Email parsing (MIME, HTML→text, attachments)
└── components/
    ├── mod.rs           # Component trait
    ├── mailbox_list.rs  # Mailbox/folder sidebar
    ├── message_list.rs  # Message list panel
    ├── reader.rs        # Message reader view
    ├── composer.rs       # Email compose view
    ├── search.rs        # Search input
    ├── command_bar.rs   # Command mode input
    ├── status_bar.rs    # Status/error bar
    └── setup_wizard.rs  # First-run account setup
```

## License

MIT
