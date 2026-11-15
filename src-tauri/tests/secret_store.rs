//! The real Credential Manager round trip, under a test service name (FR-013, FR-015).

use my_pec_lib::secret_store::CredentialManager;
use pec_core::credentials::{ApiKey, SecretStore};

#[test]
fn a_key_round_trips_through_credential_manager() {
    let store = CredentialManager::new(&format!("MyPec-test-{}", std::process::id()));
    let key = "sk-ant-api03-ROUNDTRIP-0123456789abcdef";

    store
        .set(&ApiKey::new(key))
        .expect("credential manager accepts the key");
    assert_eq!(
        store.get().map(|k| k.expose().to_owned()).as_deref(),
        Some(key)
    );

    store.delete().unwrap();
    assert!(store.get().is_none());
    store
        .delete()
        .expect("deleting an absent key is not an error");
}
