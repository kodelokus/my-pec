//! The bundled extensions: sticky notes, alarms (extensions R1) and weather (weather). Each one's rules, files and
//! tools; the core decides when they run and the shell shows what they return.

pub mod alarms;
pub mod notes;
pub mod weather;

use pec_core::ext::Extension;

/// The bundled extensions, in the order settings lists them and contexts record their tools.
pub fn all() -> Vec<Box<dyn Extension>> {
    vec![
        Box::new(notes::Notes::default()),
        Box::new(alarms::Alarms::default()),
        // Last: an alarm due in the same tick is collected before a weather request (weather R4).
        Box::new(weather::Weather::default()),
    ]
}

#[cfg(test)]
pub(crate) mod testing {
    use pec_core::Core;
    use pec_core::credentials::{ApiKey, SecretStore, StoreUnavailable};

    struct NoKey;

    impl SecretStore for NoKey {
        fn get(&self) -> Option<ApiKey> {
            None
        }
        fn set(&self, _: &ApiKey) -> Result<(), StoreUnavailable> {
            Ok(())
        }
        fn delete(&self) -> Result<(), StoreUnavailable> {
            Ok(())
        }
    }

    /// A core on a temporary root with the bundled extensions.
    pub fn core() -> (tempfile::TempDir, Core) {
        let root = tempfile::tempdir().unwrap();
        let core = Core::new(root.path(), Box::new(NoKey)).with_extensions(super::all());
        (root, core)
    }

    /// The same, with extensions' requests answered by `fetch` instead of the network.
    pub fn core_fetching(fetch: Box<dyn pec_core::ext::Fetch>) -> (tempfile::TempDir, Core) {
        let (root, core) = core();
        (root, core.with_fetch(fetch))
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    /// Principle III: a running context rebuilds its extension tools from the app, so an update that changes a
    /// tool's name, description or schema reaches contexts already open. Change them only with a new harness
    /// version in pec-core, keep the old behaviour working for older contexts, then update this digest.
    #[test]
    fn the_extension_tools_change_only_with_a_new_harness_version() {
        let tools: Vec<serde_json::Value> = super::all()
            .iter()
            .flat_map(|e| e.tools())
            .map(|t| serde_json::json!([t.name, t.description, t.input_schema]))
            .collect();
        let digest = Sha256::digest(serde_json::to_vec(&tools).unwrap());
        let digest: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            digest,
            "890a7b06f5321ae454c1a76b8cf519ecd45d83dc2e0ffde0cfb46c73f17bf353"
        );
    }
}
