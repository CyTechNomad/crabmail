use anyhow::{Context, Result};
use async_imap::Session;
use futures::TryStreamExt;
use std::sync::{Arc, OnceLock};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

use crate::config::Account;

fn tls_config() -> Result<Arc<ClientConfig>> {
    static TLS_CONFIG: OnceLock<Arc<ClientConfig>> = OnceLock::new();
    if let Some(cfg) = TLS_CONFIG.get() {
        return Ok(cfg.clone());
    }
    let mut root_store = RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs()? {
        root_store.add(cert)?;
    }
    let config = Arc::new(
        ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth(),
    );
    Ok(TLS_CONFIG.get_or_init(|| config.clone()).clone())
}

#[derive(Debug, Clone)]
pub struct Mailbox {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct MailboxInfo {
    pub exists: u32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct MessageSummary {
    pub seq: u32,
    pub uid: u32,
    pub from: String,
    pub subject: String,
    pub date: String,
    pub flags: Vec<String>,
}

pub struct ImapClient {
    session: Session<Compat<TlsStream<TcpStream>>>,
    selected_exists: u32,
}

impl ImapClient {
    pub async fn connect(account: &Account, password: &str) -> Result<Self> {
        let connector = TlsConnector::from(tls_config()?);
        let tcp = TcpStream::connect((&*account.imap_host, account.imap_port)).await?;
        let server_name =
            tokio_rustls::rustls::pki_types::ServerName::try_from(account.imap_host.as_str())?
                .to_owned();
        let tls = connector.connect(server_name, tcp).await?;
        let client = async_imap::Client::new(tls.compat());
        let session = client
            .login(&account.email, password)
            .await
            .map_err(|e| anyhow::anyhow!("IMAP login failed: {}", e.0))?;
        Ok(Self {
            session,
            selected_exists: 0,
        })
    }

    pub async fn list_mailboxes(&mut self) -> Result<Vec<Mailbox>> {
        let names: Vec<_> = self
            .session
            .list(None, Some("*"))
            .await?
            .try_collect()
            .await?;
        Ok(names
            .into_iter()
            .filter(|n| {
                !n.attributes()
                    .iter()
                    .any(|a| matches!(a, async_imap::types::NameAttribute::NoSelect))
            })
            .map(|n| Mailbox {
                name: n.name().to_string(),
            })
            .collect())
    }

    pub async fn select_mailbox(&mut self, name: &str) -> Result<MailboxInfo> {
        let mb = self.session.select(name).await?;
        self.selected_exists = mb.exists;
        Ok(MailboxInfo {
            exists: mb.exists,
            name: name.to_string(),
        })
    }

    pub async fn fetch_headers(&mut self, count: u32) -> Result<Vec<MessageSummary>> {
        let exists = self.selected_exists;
        if exists == 0 {
            return Ok(vec![]);
        }
        let start = exists.saturating_sub(count - 1).max(1);
        let range = format!("{start}:{exists}");
        let fetches: Vec<_> = self
            .session
            .fetch(&range, "(UID FLAGS RFC822.HEADER)")
            .await?
            .try_collect()
            .await?;
        let mut summaries = Vec::new();
        for fetch in &fetches {
            let uid = fetch.uid.unwrap_or(fetch.message);
            let flags: Vec<String> = fetch.flags().map(|f| format!("{f:?}")).collect();
            let header = fetch.header().unwrap_or_default();
            let header_str = String::from_utf8_lossy(header);
            let from = extract_header(&header_str, "From");
            let subject = extract_header(&header_str, "Subject");
            let date = extract_header(&header_str, "Date");
            summaries.push(MessageSummary {
                seq: fetch.message,
                uid,
                from,
                subject,
                date,
                flags,
            });
        }
        summaries.reverse();
        Ok(summaries)
    }

    pub async fn fetch_raw_message(&mut self, uid: u32) -> Result<Vec<u8>> {
        let fetches: Vec<_> = self
            .session
            .uid_fetch(uid.to_string(), "RFC822")
            .await?
            .try_collect()
            .await?;
        let fetch = fetches.first().context("Message not found")?;
        Ok(fetch.body().unwrap_or_default().to_vec())
    }

    pub async fn search(&mut self, query: &str) -> Result<Vec<u32>> {
        let sanitized: String = query.chars().filter(|&c| c != '"' && c != '\\').collect();
        let uids: Vec<_> = self
            .session
            .uid_search(format!(
                "OR OR FROM \"{}\" SUBJECT \"{}\" BODY \"{}\"",
                sanitized, sanitized, sanitized
            ))
            .await?
            .into_iter()
            .collect();
        Ok(uids)
    }

    pub async fn fetch_headers_by_uids(&mut self, uids: &[u32]) -> Result<Vec<MessageSummary>> {
        if uids.is_empty() {
            return Ok(vec![]);
        }
        let uid_str: String = uids
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let fetches: Vec<_> = self
            .session
            .uid_fetch(&uid_str, "(UID FLAGS RFC822.HEADER)")
            .await?
            .try_collect()
            .await?;
        let mut summaries = Vec::new();
        for fetch in &fetches {
            let uid = fetch.uid.unwrap_or(fetch.message);
            let flags: Vec<String> = fetch.flags().map(|f| format!("{f:?}")).collect();
            let header = fetch.header().unwrap_or_default();
            let header_str = String::from_utf8_lossy(header);
            let from = extract_header(&header_str, "From");
            let subject = extract_header(&header_str, "Subject");
            let date = extract_header(&header_str, "Date");
            summaries.push(MessageSummary {
                seq: fetch.message,
                uid,
                from,
                subject,
                date,
                flags,
            });
        }
        summaries.reverse();
        Ok(summaries)
    }

    pub async fn delete_message(&mut self, uid: u32) -> Result<()> {
        self.session
            .uid_store(uid.to_string(), "+FLAGS (\\Deleted)")
            .await?
            .try_collect::<Vec<_>>()
            .await?;
        self.session
            .expunge()
            .await?
            .try_collect::<Vec<_>>()
            .await?;
        self.selected_exists = self.selected_exists.saturating_sub(1);
        Ok(())
    }

    pub async fn logout(&mut self) -> Result<()> {
        self.session.logout().await?;
        Ok(())
    }
}

fn extract_header(headers: &str, name: &str) -> String {
    let prefix = format!("{name}:");
    headers
        .lines()
        .find(|l| l.starts_with(&prefix))
        .map(|l| l[prefix.len()..].trim().to_string())
        .unwrap_or_default()
}
