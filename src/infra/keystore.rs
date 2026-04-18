//! Thin wrapper around the OS credential store.
//!
//! Secrets for configurable data-source providers live **only** here — there
//! is no plaintext file fallback. On platforms where no backend is available
//! (for example a headless Linux box without `secret-service`), callers
//! surface `KeystoreError::Unavailable` to the UI and the Data Sources
//! settings section is rendered read-only.

use thiserror::Error;

const SERVICE: &str = "com.bullpen.app";

#[derive(Debug, Error)]
pub enum KeystoreError {
    #[error("keychain backend unavailable on this platform")]
    Unavailable,
    #[error("keychain error: {0}")]
    Other(String),
}

impl From<keyring::Error> for KeystoreError {
    fn from(err: keyring::Error) -> Self {
        match err {
            keyring::Error::PlatformFailure(_) | keyring::Error::NoStorageAccess(_) => {
                Self::Unavailable
            }
            other => Self::Other(other.to_string()),
        }
    }
}

fn entry(account: &str) -> Result<keyring::Entry, KeystoreError> {
    keyring::Entry::new(SERVICE, account).map_err(Into::into)
}

pub fn get_key(account: &str) -> Result<Option<String>, KeystoreError> {
    match entry(account)?.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub fn set_key(account: &str, value: &str) -> Result<(), KeystoreError> {
    entry(account)?.set_password(value)?;
    Ok(())
}

pub fn delete_key(account: &str) -> Result<(), KeystoreError> {
    match entry(account)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub fn has_key(account: &str) -> Result<bool, KeystoreError> {
    Ok(get_key(account)?.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // keyring's default credential builder is a process-global singleton.
    // Serialize tests that swap it in.
    static KEYRING_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn mock_backend_rejects_unknown_account_cleanly() {
        let _guard = KEYRING_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());

        // The mock builder creates a fresh, password-less credential on every
        // `Entry::new`, so `get_key` returns `None` (NoEntry is mapped). This
        // verifies the error-to-Option conversion path.
        let account = "source.test_provider.api_key";
        assert_eq!(get_key(account).unwrap(), None);
        // Deleting a non-existent entry is a no-op, not an error.
        delete_key(account).unwrap();
        assert!(!has_key(account).unwrap());
    }

    #[test]
    fn set_key_returns_ok_on_mock_backend() {
        let _guard = KEYRING_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());

        // Mock credentials don't persist between Entry instances (documented
        // behavior), so we can only assert that `set_key` succeeds — the
        // real keychain is the only place round-trips are meaningful.
        let account = "source.another_provider.api_key";
        assert!(set_key(account, "secret-xyz").is_ok());
    }
}
