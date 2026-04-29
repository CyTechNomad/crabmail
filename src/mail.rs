use anyhow::{Context, Result};
use mail_parser::{MessageParser, MimeHeaders};

#[derive(Debug, Clone)]
pub struct Attachment {
    pub name: String,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub from: String,
    pub to: String,
    pub cc: String,
    pub subject: String,
    pub date: String,
    pub text_body: String,
    pub html_body: String,
    pub attachments: Vec<Attachment>,
}

impl ParsedMessage {
    pub fn display_body(&self, width: usize) -> String {
        if !self.text_body.is_empty() {
            return self.text_body.clone();
        }
        if !self.html_body.is_empty() {
            return html2text::from_read(self.html_body.as_bytes(), width);
        }
        "(no body)".to_string()
    }
}

pub fn parse_message(raw: &[u8]) -> Result<ParsedMessage> {
    let msg = MessageParser::default()
        .parse(raw)
        .context("Failed to parse email")?;

    let from = msg
        .from()
        .map(|a| {
            a.as_list()
                .map(|list| {
                    list.iter()
                        .map(|addr| {
                            match (addr.name(), addr.address()) {
                                (Some(n), Some(a)) => format!("{n} <{a}>"),
                                (None, Some(a)) => a.to_string(),
                                (Some(n), None) => n.to_string(),
                                _ => String::new(),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let to = msg
        .to()
        .map(|a| {
            a.as_list()
                .map(|list| {
                    list.iter()
                        .map(|addr| {
                            match (addr.name(), addr.address()) {
                                (Some(n), Some(a)) => format!("{n} <{a}>"),
                                (None, Some(a)) => a.to_string(),
                                _ => String::new(),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let cc = msg
        .cc()
        .map(|a| {
            a.as_list()
                .map(|list| {
                    list.iter()
                        .map(|addr| addr.address().unwrap_or_default().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let subject = msg.subject().unwrap_or_default().to_string();
    let date = msg
        .date()
        .map(|d| d.to_rfc3339())
        .unwrap_or_default();

    let text_body = msg
        .body_text(0)
        .map(|t| t.to_string())
        .unwrap_or_default();

    let html_body = msg
        .body_html(0)
        .map(|h| h.to_string())
        .unwrap_or_default();

    let attachments = msg
        .attachments()
        .map(|a| Attachment {
            name: a
                .attachment_name()
                .unwrap_or("unnamed")
                .to_string(),
            size: a.len(),
        })
        .collect();

    Ok(ParsedMessage {
        from,
        to,
        cc,
        subject,
        date,
        text_body,
        html_body,
        attachments,
    })
}
