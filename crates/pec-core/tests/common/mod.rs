#![allow(dead_code)]

use pec_core::Core;
use pec_core::chat::{ChatEvent, ChatSink, LocalClock};
use pec_core::credentials::{ApiKey, SecretStore, StoreUnavailable};
use pec_core::llm::{LlmProvider, ModelInfo, ProviderError, Reply, StreamFailure, TurnRequest};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::TempDir;

pub const KEY: &str = "sk-ant-api03-TESTKEY-0123456789abcdefWXYZ";
pub const OTHER_KEY: &str = "sk-ant-api03-OTHERKEY-9876543210zyxwvuQRST";

/// An in-memory credential store. `refuse` makes every write fail, as a locked-down store would.
#[derive(Default, Clone)]
pub struct MemoryStore {
    pub value: Arc<Mutex<Option<String>>>,
    pub refuse: bool,
}

impl SecretStore for MemoryStore {
    fn get(&self) -> Option<ApiKey> {
        self.value.lock().unwrap().clone().map(ApiKey::new)
    }

    fn set(&self, key: &ApiKey) -> Result<(), StoreUnavailable> {
        if self.refuse {
            return Err(StoreUnavailable);
        }
        *self.value.lock().unwrap() = Some(key.expose().into());
        Ok(())
    }

    fn delete(&self) -> Result<(), StoreUnavailable> {
        *self.value.lock().unwrap() = None;
        Ok(())
    }
}

impl MemoryStore {
    pub fn stored(&self) -> Option<String> {
        self.value.lock().unwrap().clone()
    }

    pub fn delete_key(&self) {
        *self.value.lock().unwrap() = None;
    }
}

/// One step of a scripted reply.
#[derive(Clone)]
pub enum Step {
    /// A text delta.
    Text(&'static str),
    /// The reply ends with this stop reason; its content is the text so far.
    End(&'static str),
    /// The reply ends with these exact blocks.
    Blocks(Vec<Value>),
    /// The reply ends with these blocks and `stop_reason: "tool_use"`.
    Tools(Vec<Value>),
    /// The usage reported when the reply ends.
    Usage(Value),
    /// The stream fails, with the provider's requested wait.
    Fail(ProviderError, Option<Duration>),
    /// The stream never ends until it is cancelled.
    Hang,
    /// Runs while the reply streams, as if the user did something meanwhile.
    Do(Arc<dyn Fn() + Send + Sync>),
}

/// A provider that answers the model list with the scripted result, and each reply with the next script.
pub struct FakeProvider {
    pub result: Mutex<Result<Vec<ModelInfo>, ProviderError>>,
    pub calls: AtomicUsize,
    pub replies: Mutex<VecDeque<Vec<Step>>>,
    /// Every chat request, with the conversation file as it was when the request was made.
    pub requests: Mutex<Vec<(TurnRequest, String)>>,
    pub transcript: Mutex<Option<PathBuf>>,
}

impl FakeProvider {
    pub fn answering(result: Result<Vec<ModelInfo>, ProviderError>) -> Self {
        Self {
            result: Mutex::new(result),
            calls: AtomicUsize::new(0),
            replies: Mutex::new(VecDeque::new()),
            requests: Mutex::new(Vec::new()),
            transcript: Mutex::new(None),
        }
    }

    /// Queues the scripts for the next replies, in order. An empty queue answers "Hello!".
    pub fn script(&self, replies: Vec<Vec<Step>>) {
        self.replies.lock().unwrap().extend(replies);
    }

    pub fn requests(&self) -> Vec<TurnRequest> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .map(|(r, _)| r.clone())
            .collect()
    }

    pub fn accepting() -> Self {
        Self::answering(Ok(models(&["claude-opus-5-5", "claude-sonnet-5-5"])))
    }

    pub fn set(&self, result: Result<Vec<ModelInfo>, ProviderError>) {
        *self.result.lock().unwrap() = result;
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl LlmProvider for FakeProvider {
    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<ModelInfo>, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.result.lock().unwrap().clone()
    }

    async fn stream(
        &self,
        _key: &ApiKey,
        request: &TurnRequest,
        on_text: &mut (dyn FnMut(&str) + Send),
    ) -> Result<Reply, StreamFailure> {
        let file = self
            .transcript
            .lock()
            .unwrap()
            .as_ref()
            .map(|p| std::fs::read_to_string(p).unwrap_or_default());
        self.requests
            .lock()
            .unwrap()
            .push((request.clone(), file.unwrap_or_default()));
        let steps = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| vec![Step::Text("Hello!"), Step::End("end_turn")]);
        let (mut text, mut usage) = (String::new(), None);
        for step in steps {
            match step {
                Step::Text(delta) => {
                    text.push_str(delta);
                    on_text(delta);
                    tokio::task::yield_now().await;
                }
                Step::Usage(value) => usage = Some(value),
                Step::Do(action) => action(),
                Step::End(stop) => {
                    return Ok(Reply {
                        content: vec![json!({ "type": "text", "text": text })],
                        stop_reason: Some(stop.into()),
                        usage,
                        model: request.model.clone(),
                    });
                }
                Step::Blocks(content) => {
                    return Ok(Reply {
                        content,
                        stop_reason: Some("end_turn".into()),
                        usage,
                        model: request.model.clone(),
                    });
                }
                Step::Tools(content) => {
                    return Ok(Reply {
                        content,
                        stop_reason: Some("tool_use".into()),
                        usage,
                        model: request.model.clone(),
                    });
                }
                Step::Fail(error, wait) => return Err(StreamFailure { error, text, wait }),
                Step::Hang => std::future::pending::<()>().await,
            }
        }
        panic!("a script must end with End, Blocks, Tools, Fail or Hang")
    }
}

/// A `tool_use` block as the provider returns it.
pub fn tool_use(id: &str, name: &str, input: Value) -> Value {
    json!({ "type": "tool_use", "id": id, "name": name, "input": input })
}

/// Collects every event a reply sends.
#[derive(Default)]
pub struct VecSink(pub Mutex<Vec<ChatEvent>>);

impl ChatSink for VecSink {
    fn send(&self, event: ChatEvent) {
        self.0.lock().unwrap().push(event);
    }
}

impl VecSink {
    pub fn events(&self) -> Vec<ChatEvent> {
        self.0.lock().unwrap().clone()
    }

    pub fn deltas(&self) -> String {
        self.events()
            .into_iter()
            .filter_map(|e| match e {
                ChatEvent::Delta { text } => Some(text),
                _ => None,
            })
            .collect()
    }

    pub fn last(&self) -> ChatEvent {
        self.events().pop().expect("an event")
    }
}

pub fn clock() -> LocalClock {
    LocalClock {
        iso: "2026-11-13T21:40:05+07:00".into(),
        zone: "Asia/Jakarta".into(),
    }
}

pub fn models(ids: &[&str]) -> Vec<ModelInfo> {
    ids.iter()
        .map(|id| ModelInfo {
            id: (*id).into(),
            display_name: id.replace('-', " "),
            supports_effort: true,
        })
        .collect()
}

/// A fresh data root and credential store per test.
pub struct Fixture {
    pub dir: TempDir,
    pub store: MemoryStore,
}

impl Fixture {
    pub fn new() -> Self {
        Self::with_store(MemoryStore::default())
    }

    pub fn with_store(store: MemoryStore) -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
            store,
        }
    }

    pub fn core(&self) -> Core {
        Core::new(self.dir.path(), Box::new(self.store.clone()))
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn workspace(&self, file: &str) -> PathBuf {
        self.root().join("workspace").join(file)
    }

    pub fn read(&self, file: &str) -> String {
        std::fs::read_to_string(self.workspace(file)).unwrap()
    }

    /// `state.json` as written on disk.
    pub fn state(&self) -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(self.root().join("state/state.json")).unwrap())
            .unwrap()
    }

    pub fn store_key(&self, key: &str) {
        self.store.set(&ApiKey::new(key)).unwrap();
    }
}

impl Fixture {
    /// A companion as setup leaves it, with a stored key, and a provider that watches the conversation file.
    pub fn ready(&self, provider: &FakeProvider) -> Core {
        for (file, text) in [
            ("SOUL.md", "# Soul\n\nBe warm."),
            (
                "IDENTITY.md",
                "# Identity\n\n- Name: Steve\n- Avatar: pip\n",
            ),
            (
                "USER.md",
                "# About the user\n\n- Call them: Sam\n- Preferred language: English\n",
            ),
            ("MEMORY.md", "# Memory\n"),
        ] {
            self.write(file, text);
        }
        self.store_key(KEY);
        *provider.transcript.lock().unwrap() = Some(self.workspace("sessions/main.jsonl"));
        self.core()
    }

    pub fn write(&self, file: &str, text: &str) {
        let path = self.workspace(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    /// Every entry of the conversation file, parsed.
    pub fn transcript(&self) -> Vec<Value> {
        self.read("sessions/main.jsonl")
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect()
    }
}

/// search R13: a deterministic stand-in for the meaning model. Each word in a small table of concepts
/// (with synonyms and translations) adds that concept's vector; other words add nothing. Texts that share concepts
/// are close in meaning, whatever their words or language.
pub struct StandIn {
    pub fail: Arc<std::sync::atomic::AtomicBool>,
    pub calls: Arc<AtomicUsize>,
}

const CONCEPTS: [&[&str]; 6] = [
    &["dog", "dogs", "pup", "puppy", "biscuit", "anjing"],
    &[
        "storm",
        "storms",
        "thunder",
        "thunders",
        "thunderstorm",
        "petir",
        "hujan",
    ],
    &[
        "run", "running", "ran", "jog", "jogging", "lari", "marathon",
    ],
    &["cake", "bake", "baked", "baking", "kue"],
    &["sister", "sibling", "kakak", "adik"],
    &["work", "office", "job", "kantor", "deadline"],
];

impl pec_core::memory::Embedder for StandIn {
    fn embed(&mut self, texts: &[String], _query: bool) -> Result<Vec<Vec<f32>>, String> {
        self.calls.fetch_add(texts.len(), Ordering::SeqCst);
        if self.fail.load(Ordering::SeqCst) {
            return Err("stand-in failure".into());
        }
        Ok(texts
            .iter()
            .map(|text| {
                let mut v = vec![0f32; CONCEPTS.len()];
                for word in text.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
                    if let Some(i) = CONCEPTS.iter().position(|c| c.contains(&word)) {
                        v[i] += 1.0;
                    }
                }
                let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm > 0.0 {
                    v.iter_mut().for_each(|x| *x /= norm);
                }
                v
            })
            .collect())
    }
}

/// A loader of the stand-in, failing to load when `fail_load` is set.
pub struct Meaning {
    pub fail_load: Arc<std::sync::atomic::AtomicBool>,
    pub fail_embed: Arc<std::sync::atomic::AtomicBool>,
    pub calls: Arc<AtomicUsize>,
    pub loads: Arc<AtomicUsize>,
}

impl Meaning {
    pub fn new() -> Self {
        Self {
            fail_load: Arc::default(),
            fail_embed: Arc::default(),
            calls: Arc::default(),
            loads: Arc::default(),
        }
    }

    pub fn loader(&self) -> pec_core::memory::EmbedderLoader {
        let (fail_load, fail, calls, loads) = (
            self.fail_load.clone(),
            self.fail_embed.clone(),
            self.calls.clone(),
            self.loads.clone(),
        );
        Arc::new(move |_dir: &Path| {
            loads.fetch_add(1, Ordering::SeqCst);
            if fail_load.load(Ordering::SeqCst) {
                return Err("could not load".to_owned());
            }
            Ok(Box::new(StandIn {
                fail: fail.clone(),
                calls: calls.clone(),
            }) as Box<dyn pec_core::memory::Embedder>)
        })
    }
}

/// The folder the meaning model lives in.
pub fn model_dir(f: &Fixture) -> PathBuf {
    f.root().join("state/models/multilingual-e5-small")
}

/// Puts a model in place as a completed download leaves it (the stand-in ignores the files' contents).
pub fn place_model(f: &Fixture) {
    let dir = model_dir(f);
    std::fs::create_dir_all(&dir).unwrap();
    for name in ["model_quantized.onnx", "tokenizer.json"] {
        std::fs::write(dir.join(name), b"stand-in").unwrap();
    }
    let manifest: Value =
        serde_json::from_str(include_str!("../../src/memory/model.json")).unwrap();
    std::fs::write(
        dir.join("model.json"),
        json!({ "model": manifest["model"], "revision": manifest["revision"] }).to_string(),
    )
    .unwrap();
}
