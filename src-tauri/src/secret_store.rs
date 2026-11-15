//! The key in Windows Credential Manager (onboarding data 3, FR-013, FR-015).

use keyring_core::api::CredentialStoreApi;
use keyring_core::{Entry, Error};
use pec_core::credentials::{ApiKey, SecretStore, StoreUnavailable};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

pub const SERVICE: &str = "My Pec";
const ACCOUNT: &str = "claude-api-key";

/// One entry behind one lock: the store does not order operations on an entry across threads (onboarding R6).
/// `None` when the credential store cannot be opened; every write then fails and nothing falls back.
pub struct CredentialManager(Option<Mutex<Entry>>);

impl CredentialManager {
    pub fn new(service: &str) -> Self {
        let entry = windows_native_keyring_store::Store::new().and_then(|store| {
            // `Local` keeps the key on this machine; the store's default would roam with a domain profile.
            store.build(
                service,
                ACCOUNT,
                Some(&HashMap::from([("persistence", "local")])),
            )
        });
        Self(entry.ok().map(Mutex::new))
    }

    fn entry(&self) -> Option<MutexGuard<'_, Entry>> {
        self.0
            .as_ref()
            .map(|e| e.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }
}

impl SecretStore for CredentialManager {
    fn get(&self) -> Option<ApiKey> {
        self.entry()?.get_password().ok().map(ApiKey::new)
    }

    fn set(&self, key: &ApiKey) -> Result<(), StoreUnavailable> {
        let entry = self.entry().ok_or(StoreUnavailable)?;
        entry
            .set_password(key.expose())
            .map_err(|_| StoreUnavailable)
    }

    fn delete(&self) -> Result<(), StoreUnavailable> {
        match self.entry().ok_or(StoreUnavailable)?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(_) => Err(StoreUnavailable),
        }
    }
}
