use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE: &str = "crabmail";

pub fn store_password(account_name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE, account_name)
        .with_context(|| format!("Failed to create keyring entry for {account_name}"))?;
    entry
        .set_password(password)
        .with_context(|| format!("Failed to store password for {account_name}"))?;
    Ok(())
}

pub fn get_password(account_name: &str) -> Result<String> {
    let entry = Entry::new(SERVICE, account_name)
        .with_context(|| format!("Failed to create keyring entry for {account_name}"))?;
    entry
        .get_password()
        .with_context(|| format!("No password found for {account_name}"))
}

pub fn delete_password(account_name: &str) -> Result<()> {
    let entry = Entry::new(SERVICE, account_name)
        .with_context(|| format!("Failed to create keyring entry for {account_name}"))?;
    entry
        .delete_credential()
        .with_context(|| format!("Failed to delete password for {account_name}"))?;
    Ok(())
}
