//! Semantic search: search by meaning. The embedder seam and the ONNX model behind it, loading and releasing it, the
//! current model's vectors, preparation, and the status, download, switch, removal and offer (search R2, R5 to
//! search R10).

use super::download::{self, Ended};
use super::index::{Index, REFRESH_BYTES};
use crate::chat::{lock, now_ms};
use crate::{AppError, Core, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::Notify;
use ts_rs::TS;

/// search R6: the model is released after this long unused.
const IDLE_MS: u64 = 10 * 60 * 1000;
/// search R8: chunks embedded at once by preparation, and how long one tick may prepare.
const BATCH: usize = 32;
const TICK_MS: u64 = 20_000;
/// search R8: chunks a search embeds itself when the model is loaded, so fresh notes are found at once.
const ON_SEARCH: usize = 8;

/// Turns text into L2-normalized vectors: the ONNX model in the app, a stand-in in tests (search R2).
pub trait Embedder: Send {
    /// One vector per text, embedded as a search query or as a passage of memory.
    fn embed(
        &mut self,
        texts: &[String],
        query: bool,
    ) -> std::result::Result<Vec<Vec<f32>>, String>;
}

/// Builds the embedder from the model folder.
pub type EmbedderLoader =
    Arc<dyn Fn(&Path) -> std::result::Result<Box<dyn Embedder>, String> + Send + Sync>;

/// One file of the model, as the bundled manifest pins it (search data 3).
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ModelFile {
    pub path: String,
    pub name: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manifest {
    pub model: String,
    pub revision: String,
    pub source: String,
    pub files: Vec<ModelFile>,
    pub query_prefix: String,
    pub passage_prefix: String,
}

pub(crate) fn manifest() -> &'static Manifest {
    static MANIFEST: OnceLock<Manifest> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        serde_json::from_str(include_str!("model.json")).expect("model.json is valid")
    })
}

/// The model and revision vectors are stored under.
fn model_id() -> String {
    let m = manifest();
    format!("{}@{}", m.model, m.revision)
}

/// Why search by meaning needs the user's attention.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SemanticProblem {
    Damaged,
    NoSpace,
    Unreachable,
    CouldNotLoad,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum SemanticState {
    NotDownloaded,
    Downloading {
        #[ts(type = "number")]
        done: u64,
        #[ts(type = "number")]
        total: u64,
    },
    Preparing {
        done: u32,
        total: u32,
    },
    Ready,
    Off,
    NeedsAttention {
        reason: SemanticProblem,
    },
}

/// search FR-023: what settings and the memory browser show.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SemanticStatus {
    pub state: SemanticState,
    /// search FR-009: shown while the user has neither accepted nor dismissed it.
    pub offer: bool,
    #[ts(type = "number")]
    pub download_bytes: u64,
    pub source: String,
    #[ts(type = "number")]
    pub disk_bytes: u64,
}

/// The loaded model, the current model's vectors by text hash, and the download in progress.
#[derive(Default)]
struct Inner {
    embedder: Option<Arc<Mutex<Box<dyn Embedder>>>>,
    vectors: HashMap<String, Vec<f32>>,
    last_use: u64,
    loading: bool,
    failure: Option<SemanticProblem>,
    download: Option<(u64, u64, Arc<Notify>)>,
    ticking: bool,
}

pub(crate) struct SemanticRuntime {
    loader: EmbedderLoader,
    source: String,
    files: Vec<ModelFile>,
    inner: Arc<Mutex<Inner>>,
}

impl Default for SemanticRuntime {
    fn default() -> Self {
        Self {
            loader: Arc::new(onnx::load),
            source: manifest().source.clone(),
            files: manifest().files.clone(),
            inner: Arc::default(),
        }
    }
}

impl SemanticRuntime {
    /// Tests: a stand-in embedder, a local download source and its files as (name, bytes, SHA-256); no files means
    /// the manifest's (search FR-026).
    pub(crate) fn new(
        loader: EmbedderLoader,
        source: String,
        files: Vec<(String, u64, String)>,
    ) -> Self {
        let files = if files.is_empty() {
            manifest().files.clone()
        } else {
            files
                .into_iter()
                .map(|(name, bytes, sha256)| ModelFile {
                    path: name.clone(),
                    name,
                    bytes,
                    sha256,
                })
                .collect()
        };
        Self {
            loader,
            source,
            files,
            inner: Arc::default(),
        }
    }
}

/// search R6: whether a model last used at `last` should be released at `now`.
pub(crate) fn idle(last: u64, now: u64) -> bool {
    now.saturating_sub(last) >= IDLE_MS
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Clears the ticking flag however a tick ends.
struct Ticking<'a>(&'a Mutex<Inner>);

impl Drop for Ticking<'_> {
    fn drop(&mut self) {
        lock(self.0).ticking = false;
    }
}

impl Core {
    fn semantic(&self) -> &SemanticRuntime {
        &self.memory.semantic
    }

    fn model_dir(&self) -> PathBuf {
        self.state_dir().join("models").join(&manifest().model)
    }

    /// The pinned model is in its folder: `model.json` names its revision and every file is there. The files were
    /// verified when the download completed; a damaged one shows when it fails to load (search data 2).
    fn model_in_place(&self) -> bool {
        let dir = self.model_dir();
        let named = std::fs::read_to_string(dir.join("model.json"))
            .ok()
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
            .is_some_and(|v| v["revision"] == manifest().revision.as_str());
        named
            && self
                .semantic()
                .files
                .iter()
                .all(|f| dir.join(&f.name).is_file())
    }

    fn usable(&self) -> bool {
        lock(&self.semantic().inner).failure.is_none()
            && self.settings().is_ok_and(|s| s.search.semantic.enabled)
            && self.model_in_place()
    }

    /// Drops the model and its vectors from memory.
    fn release(&self) {
        let mut inner = lock(&self.semantic().inner);
        inner.embedder = None;
        inner.vectors.clear();
    }

    /// search FR-020: a failing model needs attention and leaves search to keywords.
    fn fail(&self, problem: SemanticProblem) {
        self.release();
        lock(&self.semantic().inner).failure = Some(problem);
    }

    /// Loads the model and the current vectors now, unless they are loaded or unusable.
    fn load_model(&self) -> bool {
        if lock(&self.semantic().inner).embedder.is_some() {
            return true;
        }
        if !self.usable() {
            return false;
        }
        match (self.semantic().loader)(&self.model_dir()) {
            Ok(embedder) => {
                let Ok(vectors) = self.with_index(|i| i.vectors(&model_id())) else {
                    return false;
                };
                let mut inner = lock(&self.semantic().inner);
                inner.embedder = Some(Arc::new(Mutex::new(embedder)));
                inner.vectors = vectors;
                inner.last_use = now_ms();
                true
            }
            Err(_) => {
                self.fail(SemanticProblem::CouldNotLoad);
                false
            }
        }
    }

    /// search R5: recall never waits for the model; it starts loading on a thread of its own for the next one.
    fn start_loading(&self) {
        if !self.usable() {
            return;
        }
        let inner = Arc::clone(&self.semantic().inner);
        {
            let mut held = lock(&inner);
            if held.loading || held.embedder.is_some() {
                return;
            }
            held.loading = true;
        }
        let loader = Arc::clone(&self.semantic().loader);
        let (dir, db) = (self.model_dir(), self.index_path());
        std::thread::spawn(move || {
            let loaded = loader(&dir).map(|e| {
                let vectors = Index::open(&db)
                    .and_then(|i| i.vectors(&model_id()))
                    .unwrap_or_default();
                (e, vectors)
            });
            let mut held = lock(&inner);
            held.loading = false;
            match loaded {
                Ok((embedder, vectors)) => {
                    held.embedder = Some(Arc::new(Mutex::new(embedder)));
                    held.vectors = vectors;
                    held.last_use = now_ms();
                }
                Err(_) => held.failure = Some(SemanticProblem::CouldNotLoad),
            }
        });
    }

    /// Embeds chunk texts and stores the vectors that still have a chunk (search R8).
    fn embed_and_store(
        &self,
        embedder: &Mutex<Box<dyn Embedder>>,
        batch: Vec<(String, String)>,
    ) -> bool {
        let texts: Vec<String> = batch.iter().map(|(_, t)| t.clone()).collect();
        let Ok(vectors) = lock(embedder).embed(&texts, false) else {
            self.fail(SemanticProblem::CouldNotLoad);
            return false;
        };
        self.store(batch.into_iter().map(|(h, _)| h).zip(vectors).collect())
    }

    fn store(&self, vectors: Vec<(String, Vec<f32>)>) -> bool {
        let Ok(kept) = self.with_index(|i| i.store_vectors(&model_id(), vectors)) else {
            return false;
        };
        let mut inner = lock(&self.semantic().inner);
        inner.last_use = now_ms();
        inner.vectors.extend(kept);
        true
    }

    /// Each current vector's cosine to `query`, by text hash; none when the model can't answer now. `wait` lets the
    /// model load and embeds a few missing chunks first (search R5, R6, R8).
    pub(crate) fn query_meaning(&self, query: &str, wait: bool) -> Option<HashMap<String, f32>> {
        if wait && self.load_model() {
            let embedder = lock(&self.semantic().inner).embedder.clone()?;
            if let Ok(batch) = self.with_index(|i| i.missing(&model_id(), ON_SEARCH))
                && !batch.is_empty()
            {
                self.embed_and_store(&embedder, batch);
            }
        }
        let embedder = lock(&self.semantic().inner).embedder.clone();
        let Some(embedder) = embedder else {
            if !wait {
                self.start_loading();
            }
            return None;
        };
        let embedded = if wait {
            lock(&embedder).embed(&[query.to_owned()], true)
        } else {
            // A preparation batch holds the model: this message is recalled by keyword (search R5).
            embedder.try_lock().ok()?.embed(&[query.to_owned()], true)
        };
        let Ok(vector) = embedded else {
            self.fail(SemanticProblem::CouldNotLoad);
            return None;
        };
        let query = vector.first()?;
        let mut inner = lock(&self.semantic().inner);
        inner.last_use = now_ms();
        Some(
            inner
                .vectors
                .iter()
                .map(|(h, v)| (h.clone(), dot(query, v)))
                .collect(),
        )
    }

    /// search FR-023: the state, the offer, the size, the source and the space used.
    pub fn semantic_status(&self) -> Result<SemanticStatus> {
        let settings = self.settings()?.search.semantic;
        let in_place = self.model_in_place();
        let (download, failure) = {
            let inner = lock(&self.semantic().inner);
            (
                inner.download.as_ref().map(|(d, t, _)| (*d, *t)),
                inner.failure,
            )
        };
        let state = if let Some((done, total)) = download {
            SemanticState::Downloading { done, total }
        } else if let Some(reason) = failure {
            SemanticState::NeedsAttention { reason }
        } else if !in_place {
            SemanticState::NotDownloaded
        } else if !settings.enabled {
            SemanticState::Off
        } else {
            match self.with_index(|i| i.prepared(&model_id()))? {
                (done, total) if done < total => SemanticState::Preparing { done, total },
                _ => SemanticState::Ready,
            }
        };
        let disk_bytes = std::fs::read_dir(self.model_dir())
            .map(|d| {
                d.flatten()
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| m.len())
                    .sum()
            })
            .unwrap_or(0);
        Ok(SemanticStatus {
            state,
            offer: !settings.accepted && !settings.offer_dismissed && !in_place,
            download_bytes: self.semantic().files.iter().map(|f| f.bytes).sum(),
            source: "Hugging Face (huggingface.co)".into(),
            disk_bytes,
        })
    }

    /// search FR-008, FR-010 to FR-013: the user's consent, then the download and its verification.
    pub async fn start_semantic_download(&self) -> Result<SemanticStatus> {
        let stop = Arc::new(Notify::new());
        {
            let mut inner = lock(&self.semantic().inner);
            if inner.download.is_some() {
                return Err(AppError::invalid("busy"));
            }
            inner.failure = None;
            if !self.model_in_place() {
                let total = self.semantic().files.iter().map(|f| f.bytes).sum();
                inner.download = Some((0, total, Arc::clone(&stop)));
            }
        }
        let mut settings = self.settings()?;
        settings.search.semantic.accepted = true;
        self.save_settings(&settings)?;
        if self.model_in_place() {
            return self.semantic_status();
        }
        let dir = self.model_dir();
        let progress = |done: u64| {
            if let Some(d) = lock(&self.semantic().inner).download.as_mut() {
                d.0 = done;
            }
        };
        let fetched = download::fetch(
            &self.semantic().source,
            &self.semantic().files,
            &dir,
            &progress,
            &stop,
        )
        .await;
        let named = fetched.is_ok()
            && std::fs::write(
                dir.join("model.json"),
                serde_json::json!({ "model": manifest().model, "revision": manifest().revision })
                    .to_string(),
            )
            .is_ok();
        {
            let mut inner = lock(&self.semantic().inner);
            inner.download = None;
            match fetched {
                Err(Ended::Failed(problem)) => inner.failure = Some(problem),
                Ok(()) if !named => inner.failure = Some(SemanticProblem::NoSpace),
                _ => {}
            }
        }
        self.semantic_status()
    }

    /// search FR-010: stops a running download at its next write; the partial file stays for resuming.
    pub fn cancel_semantic_download(&self) -> Result<SemanticStatus> {
        if let Some((_, _, stop)) = lock(&self.semantic().inner).download.as_ref() {
            stop.notify_one();
        }
        self.semantic_status()
    }

    /// search FR-024: off keeps the model; on uses it again without a download.
    pub fn set_semantic_search(&self, enabled: bool) -> Result<SemanticStatus> {
        let mut settings = self.settings()?;
        settings.search.semantic.enabled = enabled;
        self.save_settings(&settings)?;
        if !enabled {
            self.release();
        }
        lock(&self.semantic().inner).failure = None;
        self.semantic_status()
    }

    /// search FR-024, FR-025: the model and every vector go; the offer comes back.
    pub fn remove_semantic_model(&self) -> Result<SemanticStatus> {
        if lock(&self.semantic().inner).download.is_some() {
            return Err(AppError::invalid("busy"));
        }
        self.release();
        lock(&self.semantic().inner).failure = None;
        let dir = self.model_dir();
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|_| AppError::not_writable(&dir))?;
        }
        self.with_index(|i| i.drop_vectors(None))?;
        let mut settings = self.settings()?;
        settings.search.semantic.accepted = false;
        settings.search.semantic.offer_dismissed = false;
        self.save_settings(&settings)?;
        self.semantic_status()
    }

    /// search FR-009: hides the offer; the settings section stays.
    pub fn dismiss_semantic_offer(&self) -> Result<SemanticStatus> {
        let mut settings = self.settings()?;
        settings.search.semantic.offer_dismissed = true;
        self.save_settings(&settings)?;
        self.semantic_status()
    }

    /// search R6, R8: releases an idle model, then prepares missing vectors in batches until none are left, a
    /// reply runs, or the tick's time is up. Never downloads.
    pub async fn semantic_tick(&self) -> Result<SemanticStatus> {
        {
            let mut inner = lock(&self.semantic().inner);
            if inner.embedder.is_some() && !inner.ticking && idle(inner.last_use, now_ms()) {
                inner.embedder = None;
                inner.vectors.clear();
            }
            if inner.ticking {
                drop(inner);
                return self.semantic_status();
            }
            inner.ticking = true;
        }
        let _ticking = Ticking(&self.semantic().inner);
        // What preparation covers is what keyword search covers: bring the index up to date first.
        let workspace = self.workspace_dir();
        self.with_index(|i| i.refresh(&workspace, REFRESH_BYTES))?;
        let deadline = now_ms() + TICK_MS;
        while self.usable() && lock(&self.chat.active).is_none() && now_ms() < deadline {
            let batch = self.with_index(|i| i.missing(&model_id(), BATCH))?;
            if batch.is_empty() {
                // search FR-019: once everything has the current model's vectors, the old model's go.
                self.with_index(|i| i.drop_vectors(Some(&model_id())))?;
                break;
            }
            let embedder = lock(&self.semantic().inner).embedder.clone();
            let embedder = match embedder {
                Some(e) => e,
                None => {
                    let (loader, dir) = (Arc::clone(&self.semantic().loader), self.model_dir());
                    match tokio::task::spawn_blocking(move || loader(&dir)).await {
                        Ok(Ok(e)) => {
                            let vectors = self.with_index(|i| i.vectors(&model_id()))?;
                            let e = Arc::new(Mutex::new(e));
                            let mut inner = lock(&self.semantic().inner);
                            inner.embedder = Some(Arc::clone(&e));
                            inner.vectors = vectors;
                            inner.last_use = now_ms();
                            e
                        }
                        _ => {
                            self.fail(SemanticProblem::CouldNotLoad);
                            break;
                        }
                    }
                }
            };
            let texts: Vec<String> = batch.iter().map(|(_, t)| t.clone()).collect();
            let worker = Arc::clone(&embedder);
            let embedded =
                tokio::task::spawn_blocking(move || lock(&worker).embed(&texts, false)).await;
            let Ok(Ok(vectors)) = embedded else {
                self.fail(SemanticProblem::CouldNotLoad);
                break;
            };
            self.store(batch.into_iter().map(|(h, _)| h).zip(vectors).collect());
        }
        self.semantic_status()
    }
}

/// The ONNX model: the tokenizer, the session and mean pooling (search interfaces).
mod onnx {
    use super::{Embedder, manifest};
    use ort::environment::Environment;
    use ort::session::Session;
    use ort::value::Tensor;
    use std::path::Path;
    use std::sync::OnceLock;
    use tokenizers::{PaddingParams, Tokenizer, TruncationParams};

    pub(super) struct Onnx {
        session: Session,
        tokenizer: Tokenizer,
        inputs: Vec<String>,
    }

    fn text(e: impl std::fmt::Display) -> String {
        e.to_string()
    }

    /// One ONNX Runtime environment per process, with its telemetry off (Principle II). `ort` panics on a library it
    /// cannot use; that is a model that could not load, not a reason to stop (Principle V).
    fn environment() -> Result<&'static Environment, String> {
        static ENV: OnceLock<Result<Environment, String>> = OnceLock::new();
        ENV.get_or_init(|| {
            std::panic::catch_unwind(|| {
                builder()?
                    .with_name("my-pec")
                    .with_telemetry(false)
                    .build()
                    .map_err(text)
            })
            .unwrap_or_else(|_| Err("ONNX Runtime could not be loaded".into()))
        })
        .as_ref()
        .map_err(Clone::clone)
    }

    /// Windows loads Microsoft's `onnxruntime.dll` from beside the app, or from `ORT_DYLIB_PATH`, never the older
    /// Windows ML copy in System32.
    #[cfg(windows)]
    fn builder() -> Result<ort::environment::EnvironmentBuilder, String> {
        let path = match std::env::var_os("ORT_DYLIB_PATH") {
            Some(path) if !path.is_empty() => path.into(),
            _ => std::env::current_exe()
                .map_err(text)?
                .with_file_name("onnxruntime.dll"),
        };
        ort::init_from(path).map_err(text)
    }

    #[cfg(not(windows))]
    fn builder() -> Result<ort::environment::EnvironmentBuilder, String> {
        Ok(ort::init())
    }

    pub(super) fn load(dir: &Path) -> Result<Box<dyn Embedder>, String> {
        let mut tokenizer = Tokenizer::from_file(dir.join("tokenizer.json")).map_err(text)?;
        tokenizer.with_padding(Some(PaddingParams::default()));
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length: 512,
                ..Default::default()
            }))
            .map_err(text)?;
        let session = Session::builder(environment()?)
            .map_err(text)?
            .with_intra_threads(2)
            .map_err(text)?
            .commit_from_file(dir.join("model_quantized.onnx"))
            .map_err(text)?;
        let inputs = session
            .inputs()
            .iter()
            .map(|i| i.name().to_owned())
            .collect();
        Ok(Box::new(Onnx {
            session,
            tokenizer,
            inputs,
        }))
    }

    impl Embedder for Onnx {
        fn embed(&mut self, texts: &[String], query: bool) -> Result<Vec<Vec<f32>>, String> {
            let m = manifest();
            let prefix = if query {
                &m.query_prefix
            } else {
                &m.passage_prefix
            };
            let prefixed: Vec<String> = texts.iter().map(|t| format!("{prefix}{t}")).collect();
            let encodings = self.tokenizer.encode_batch(prefixed, true).map_err(text)?;
            let (batch, seq) = (
                encodings.len(),
                encodings.first().map_or(0, |e| e.get_ids().len()),
            );
            let mask: Vec<i64> = encodings
                .iter()
                .flat_map(|e| e.get_attention_mask().iter().map(|&x| i64::from(x)))
                .collect();
            let mut inputs = Vec::new();
            for name in &self.inputs {
                let data: Vec<i64> = match name.as_str() {
                    "input_ids" => encodings
                        .iter()
                        .flat_map(|e| e.get_ids().iter().map(|&x| i64::from(x)))
                        .collect(),
                    "attention_mask" => mask.clone(),
                    "token_type_ids" => vec![0; batch * seq],
                    other => return Err(format!("unexpected input {other}")),
                };
                let tensor = Tensor::from_array(([batch, seq], data)).map_err(text)?;
                inputs.push((name.clone(), tensor));
            }
            let outputs = self.session.run(inputs).map_err(text)?;
            let (shape, hidden) = outputs[0].try_extract_tensor::<f32>().map_err(text)?;
            let dim = *shape.last().ok_or("no output")? as usize;
            let mut out = Vec::with_capacity(batch);
            for b in 0..batch {
                let mut sum = vec![0f32; dim];
                let mut count = 0f32;
                for t in 0..seq {
                    if mask[b * seq + t] == 0 {
                        continue;
                    }
                    count += 1.0;
                    let row = &hidden[(b * seq + t) * dim..(b * seq + t + 1) * dim];
                    for (s, v) in sum.iter_mut().zip(row) {
                        *s += v;
                    }
                }
                let norm = sum
                    .iter()
                    .map(|v| (v / count.max(1.0)).powi(2))
                    .sum::<f32>()
                    .sqrt()
                    .max(1e-12);
                out.push(sum.iter().map(|v| v / count.max(1.0) / norm).collect());
            }
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_model_is_released_after_ten_idle_minutes() {
        // search FR-022, search R6.
        let minute = 60_000;
        assert!(!idle(1_000, 1_000 + 9 * minute));
        assert!(idle(1_000, 1_000 + 10 * minute));
        assert!(!idle(5_000, 1_000), "a clock going back is not idleness");
    }

    #[test]
    fn the_manifest_pins_the_model() {
        let m = manifest();
        assert_eq!(m.revision.len(), 40);
        assert_eq!(m.files.len(), 2);
        assert!(m.files.iter().all(|f| f.sha256.len() == 64 && f.bytes > 0));
        assert!(m.source.ends_with(&format!("{}/", m.revision)));
    }
}
