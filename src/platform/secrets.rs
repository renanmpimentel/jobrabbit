//! Secure credential storage via keyring v3 (OS-native backend).
//!
//! Backend per OS: Secret Service (gnome-keyring / KWallet) on Linux, Keychain on
//! macOS, Credential Manager on Windows — selected by Cargo features per target.
//! Useful for storing tokens/secrets the agent needs, without plaintext on disk.

use anyhow::Result;

const SERVICE: &str = "jobrabbit";

/// Writes (or replaces) a secret.
pub fn set(key: &str, value: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, key)?;
    entry.set_password(value)?;
    Ok(())
}

/// Reads a secret. `Ok(None)` if it doesn't exist.
pub fn get(key: &str) -> Result<Option<String>> {
    let entry = keyring::Entry::new(SERVICE, key)?;
    match entry.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Removes a secret (idempotent).
pub fn delete(key: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, key)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
