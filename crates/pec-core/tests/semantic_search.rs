//! Search by meaning on a temporary workspace with real SQLite and the stand-in embedder: the four search paths,
//! preparation, every state without a working model, forgetting and rebuilding, the switch and removal, and the
//! download against a local server (search R13).

mod common;

use common::{FakeProvider, Fixture, Meaning, Step, VecSink, model_dir, place_model, tool_use};
use pec_core::Core;
use pec_core::chat::LocalClock;
use pec_core::memory::{ForgetTarget, SemanticProblem, SemanticState};
use serde_json::json;
use std::sync::atomic::Ordering;
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate, matchers::method};

const BISCUIT: &str = "Biscuit hides under the bed when it thunders.";
const CURTAINS: &str = "Bought new curtains for the living room.";

fn on(day: &str) -> LocalClock {
    LocalClock {
        iso: format!("{day}T20:00:00+07:00"),
        zone: "Asia/Jakarta".into(),
    }
}

/// A set-up companion whose meaning model is the stand-in, downloading from `source` when asked.
fn companion(
    f: &Fixture,
    provider: &FakeProvider,
    meaning: &Meaning,
    source: &str,
    files: Vec<(String, u64, String)>,
) -> Core {
    let _ = f.ready(provider);
    f.core().with_semantic(meaning.loader(), source, files)
}

fn note(f: &Fixture, day: &str, lines: &[&str]) {
    let body: String = lines
        .iter()
        .enumerate()
        .map(|(i, l)| format!("- 21:{i:02} {l} <!-- from:companion src:1-0 -->\n"))
        .collect();
    f.write(&format!("memory/{day}.md"), &format!("# {day}\n\n{body}"));
}

fn texts(core: &Core, query: &str) -> Vec<String> {
    core.memory_search(query)
        .unwrap()
        .into_iter()
        .map(|h| h.text)
        .collect()
}

fn db(f: &Fixture) -> rusqlite::Connection {
    rusqlite::Connection::open(f.root().join("state/pec.db")).unwrap()
}

fn count(f: &Fixture, sql: &str) -> i64 {
    db(f).query_row(sql, [], |r| r.get(0)).unwrap()
}

/// A companion with two old notes and a prepared model.
async fn prepared(f: &Fixture, provider: &FakeProvider, meaning: &Meaning) -> Core {
    let core = companion(f, provider, meaning, "http://127.0.0.1:9/", Vec::new());
    note(f, "2026-11-01", &[BISCUIT]);
    note(f, "2026-11-02", &[CURTAINS]);
    place_model(f);
    let status = core.semantic_tick().await.unwrap();
    assert_eq!(status.state, SemanticState::Ready);
    core
}

#[tokio::test]
async fn meaning_finds_what_words_miss_everywhere_memory_is_searched() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let meaning = Meaning::new();
    let core = prepared(&f, &provider, &meaning).await;
    core.set_semantic_search(false).unwrap();
    assert!(
        texts(&core, "pup scared storms").is_empty(),
        "no shared words"
    );
    core.set_semantic_search(true).unwrap();

    // The memory browser, in two languages (search FR-001, FR-004).
    assert!(texts(&core, "pup scared storms")[0].contains(BISCUIT));
    assert!(texts(&core, "anjing takut petir")[0].contains(BISCUIT));
    // search FR-002: a note sharing the query's words still leads.
    assert!(texts(&core, "curtains living room")[0].contains(CURTAINS));

    // The companion's own search, with its usual result format (search FR-005, FR-006).
    provider.script(vec![
        vec![Step::Tools(vec![tool_use(
            "t1",
            "memory_search",
            json!({ "query": "how is my pup in storms" }),
        )])],
        vec![Step::Text("He hides."), Step::End("end_turn")],
    ]);
    core.send_message(
        &provider,
        "remember my dog?",
        &on("2026-11-20"),
        &VecSink::default(),
    )
    .await
    .unwrap();
    let result = f
        .transcript()
        .into_iter()
        .find(|l| l["provenance"] == "untrusted")
        .unwrap()
        .to_string();
    assert!(
        result.contains("Biscuit hides under the bed") && result.contains("memory/2026-11-01.md"),
        "{result}"
    );

    // Auto-recall before a reply, and its evidence for dreaming (search FR-007).
    core.send_message(
        &provider,
        "my pup is a wreck again, the storm is loud",
        &on("2026-11-20"),
        &VecSink::default(),
    )
    .await
    .unwrap();
    let request = provider.requests().last().unwrap().clone();
    let message = request.messages.last().unwrap().to_string();
    assert!(message.contains("Biscuit hides under the bed"), "{message}");
    let coverage: f64 = db(&f)
        .query_row(
            "SELECT max(coverage) FROM recall_events WHERE source = 'auto' AND path = 'memory/2026-11-01.md'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(coverage > 0.9, "{coverage}");
}

#[tokio::test]
async fn preparation_runs_in_ticks_yields_to_chat_and_fresh_notes_are_found_at_once() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let meaning = Meaning::new();
    let core = companion(&f, &provider, &meaning, "http://127.0.0.1:9/", Vec::new());
    note(&f, "2026-11-01", &[BISCUIT]);
    place_model(&f);
    core.sync_memory_index().unwrap();
    assert_eq!(
        core.semantic_status().unwrap().state,
        SemanticState::Preparing { done: 0, total: 1 }
    );

    // A reply running: the tick prepares nothing (Principle V).
    provider.script(vec![vec![Step::Hang]]);
    let sink = VecSink::default();
    let clock = on("2026-11-20");
    let reply = core.send_message(&provider, "hello", &clock, &sink);
    let tick = async {
        while !core.chat_state().unwrap().replying {
            tokio::task::yield_now().await;
        }
        core.semantic_tick().await.unwrap()
    };
    tokio::select! {
        _ = reply => panic!("the reply hangs"),
        status = tick => assert_eq!(status.state, SemanticState::Preparing { done: 0, total: 1 }),
    }
    let f2 = Fixture::new();
    let core = companion(&f2, &provider, &meaning, "http://127.0.0.1:9/", Vec::new());
    note(&f2, "2026-11-01", &[BISCUIT]);
    place_model(&f2);
    assert_eq!(
        core.semantic_tick().await.unwrap().state,
        SemanticState::Ready
    );
    let loads = meaning.loads.load(Ordering::SeqCst);
    // Nothing to do: no work, no new load.
    assert_eq!(
        core.semantic_tick().await.unwrap().state,
        SemanticState::Ready
    );
    assert_eq!(meaning.loads.load(Ordering::SeqCst), loads);
    // search FR-015: a note written now is found by meaning on the next search, before any tick.
    note(&f2, "2026-11-03", &["Ran a slow jog by the river."]);
    assert!(texts(&core, "marathon training")[0].contains("slow jog"));
}

#[tokio::test]
async fn without_a_working_model_every_search_is_keyword_search() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let meaning = Meaning::new();
    let core = companion(&f, &provider, &meaning, "http://127.0.0.1:9/", Vec::new());
    note(&f, "2026-11-01", &[BISCUIT]);
    note(&f, "2026-11-02", &[CURTAINS]);
    let keyword = |core: &Core| (texts(core, "curtains"), texts(core, "pup scared storms"));
    // Not downloaded.
    assert_eq!(
        core.semantic_status().unwrap().state,
        SemanticState::NotDownloaded
    );
    let baseline = keyword(&core);
    assert!(baseline.1.is_empty());
    // Off.
    place_model(&f);
    core.set_semantic_search(false).unwrap();
    assert_eq!(keyword(&core), baseline);
    // Fails to load: needs attention, keyword results, and no retry until the user acts (search FR-020, SC-005).
    core.set_semantic_search(true).unwrap();
    meaning.fail_load.store(true, Ordering::SeqCst);
    assert_eq!(keyword(&core), baseline);
    assert_eq!(
        core.semantic_status().unwrap().state,
        SemanticState::NeedsAttention {
            reason: SemanticProblem::CouldNotLoad
        }
    );
    let loads = meaning.loads.load(Ordering::SeqCst);
    assert_eq!(keyword(&core), baseline);
    assert_eq!(meaning.loads.load(Ordering::SeqCst), loads, "not retried");
    // A message is still answered, with no error.
    provider.script(vec![vec![Step::Text("Hi!"), Step::End("end_turn")]]);
    let sink = VecSink::default();
    assert_eq!(
        core.send_message(&provider, "pup scared storms", &on("2026-11-20"), &sink)
            .await,
        Ok(None)
    );
    assert_eq!(sink.deltas(), "Hi!");
    // Fails to embed after loading.
    meaning.fail_load.store(false, Ordering::SeqCst);
    meaning.fail_embed.store(true, Ordering::SeqCst);
    core.set_semantic_search(true).unwrap();
    assert_eq!(keyword(&core), baseline);
    assert!(matches!(
        core.semantic_status().unwrap().state,
        SemanticState::NeedsAttention { .. }
    ));
}

#[tokio::test]
async fn search_by_meaning_follows_edits_forgetting_rebuilds_and_model_changes() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let meaning = Meaning::new();
    let core = prepared(&f, &provider, &meaning).await;
    note(
        &f,
        "2026-11-04",
        &["Ran 5 km.", "Baked a cake for my sister."],
    );
    assert!(texts(&core, "jogging")[0].contains("Ran 5 km"));

    // An outside edit: the new meaning is found and the old one is not; only the changed line is embedded again.
    let calls = meaning.calls.load(Ordering::SeqCst);
    note(&f, "2026-11-04", &["Ran 5 km.", "Long day at the office."]);
    assert!(texts(&core, "deadline at work")[0].contains("office"));
    assert_eq!(
        meaning.calls.load(Ordering::SeqCst) - calls,
        2,
        "the new line and the query"
    );
    assert!(
        texts(&core, "kue sibling")
            .iter()
            .all(|t| !t.contains("Baked"))
    );
    // Vectors exist only for chunk texts (search FR-018).
    assert_eq!(
        count(&f, "SELECT count(*) FROM chunk_vectors"),
        count(&f, "SELECT count(DISTINCT text_hash) FROM chunks")
    );

    // Forgetting takes the vector with it (search FR-017, SC-007).
    let doc = core.memory_read("memory/2026-11-01.md").unwrap();
    core.memory_forget(
        "memory/2026-11-01.md",
        ForgetTarget::Line { line: 3 },
        &doc.hash,
    )
    .unwrap();
    assert!(
        texts(&core, "pup scared storms")
            .iter()
            .all(|t| !t.contains("Biscuit"))
    );
    assert_eq!(
        count(&f, "SELECT count(*) FROM chunk_vectors"),
        count(&f, "SELECT count(DISTINCT text_hash) FROM chunks")
    );

    // A deleted index is rebuilt from the files, by meaning too, with no download (search FR-016, SC-008).
    let before = texts(&core, "jogging office");
    drop(core);
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(f.root().join(format!("state/pec.db{suffix}")));
    }
    let core = f
        .core()
        .with_semantic(meaning.loader(), "http://127.0.0.1:9/", Vec::new());
    core.sync_memory_index().unwrap();
    assert_eq!(
        core.semantic_tick().await.unwrap().state,
        SemanticState::Ready
    );
    assert_eq!(texts(&core, "jogging office"), before);

    // Another model's vectors are never used and go once the current model is complete (search FR-019).
    db(&f)
        .execute(
            "INSERT INTO chunk_vectors (text_hash, model, vector) SELECT text_hash, 'old@1', x'0000803f' FROM chunks",
            [],
        )
        .unwrap();
    assert_eq!(texts(&core, "jogging office"), before);
    core.semantic_tick().await.unwrap();
    assert_eq!(
        count(
            &f,
            "SELECT count(*) FROM chunk_vectors WHERE model = 'old@1'"
        ),
        0
    );
}

#[tokio::test]
async fn the_switch_keeps_the_model_and_removal_takes_it_back() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let meaning = Meaning::new();
    let core = prepared(&f, &provider, &meaning).await;
    assert_eq!(
        core.set_semantic_search(false).unwrap().state,
        SemanticState::Off
    );
    assert!(texts(&core, "pup scared storms").is_empty());
    assert!(model_dir(&f).exists());
    assert_eq!(
        core.set_semantic_search(true).unwrap().state,
        SemanticState::Ready
    );
    assert!(texts(&core, "pup scared storms")[0].contains(BISCUIT));

    let status = core.remove_semantic_model().unwrap();
    assert_eq!(status.state, SemanticState::NotDownloaded);
    assert!(status.offer && status.disk_bytes == 0);
    assert!(!model_dir(&f).exists());
    assert_eq!(count(&f, "SELECT count(*) FROM chunk_vectors"), 0);
    assert!(texts(&core, "pup scared storms").is_empty());
    // Turning it on downloads nothing.
    assert_eq!(
        core.set_semantic_search(true).unwrap().state,
        SemanticState::NotDownloaded
    );
}

/// Serves a file, honouring `Range: bytes=N-` when `ranges` is set.
struct Served {
    body: Vec<u8>,
    ranges: bool,
}

impl Respond for Served {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let from = request
            .headers
            .get("range")
            .and_then(|r| r.to_str().ok())
            .and_then(|r| {
                r.strip_prefix("bytes=")?
                    .trim_end_matches('-')
                    .parse::<usize>()
                    .ok()
            });
        match from {
            Some(from) if self.ranges => {
                ResponseTemplate::new(206).set_body_bytes(self.body[from..].to_vec())
            }
            _ => ResponseTemplate::new(200).set_body_bytes(self.body.clone()),
        }
    }
}

fn sha(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

async fn server(files: &[(&str, &[u8])], ranges: bool) -> MockServer {
    let server = MockServer::start().await;
    for (name, body) in files {
        Mock::given(method("GET"))
            .and(wiremock::matchers::path(format!("/{name}")))
            .respond_with(Served {
                body: body.to_vec(),
                ranges,
            })
            .mount(&server)
            .await;
    }
    server
}

const MODEL: &[u8] = b"model bytes, a fair number of them for a resumable download";
const TOKENS: &[u8] = b"tokenizer bytes";

fn files(model: &[u8]) -> Vec<(String, u64, String)> {
    vec![
        (
            "model_quantized.onnx".into(),
            model.len() as u64,
            sha(model),
        ),
        ("tokenizer.json".into(), TOKENS.len() as u64, sha(TOKENS)),
    ]
}

#[tokio::test]
async fn the_download_needs_consent_resumes_and_is_verified() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let meaning = Meaning::new();
    let srv = server(
        &[("model_quantized.onnx", MODEL), ("tokenizer.json", TOKENS)],
        true,
    )
    .await;
    let core = companion(
        &f,
        &provider,
        &meaning,
        &format!("{}/", srv.uri()),
        files(MODEL),
    );

    // search FR-008, FR-009: an offer with the size and source, and no request until the user accepts.
    let status = core.semantic_status().unwrap();
    assert!(status.offer);
    assert_eq!(status.download_bytes, (MODEL.len() + TOKENS.len()) as u64);
    assert!(status.source.contains("huggingface.co"));
    assert!(srv.received_requests().await.unwrap().is_empty());
    assert!(!core.dismiss_semantic_offer().unwrap().offer);

    // An interrupted download resumes from its partial file with a range request (search FR-010).
    std::fs::create_dir_all(model_dir(&f)).unwrap();
    std::fs::write(
        model_dir(&f).join("model_quantized.onnx.part"),
        &MODEL[..20],
    )
    .unwrap();
    let status = core.start_semantic_download().await.unwrap();
    assert_eq!(status.state, SemanticState::Ready);
    assert_eq!(
        std::fs::read(model_dir(&f).join("model_quantized.onnx")).unwrap(),
        MODEL
    );
    let requests = srv.received_requests().await.unwrap();
    assert_eq!(requests[0].headers.get("range").unwrap(), "bytes=20-");
    assert!(
        f.read("../state/settings.json")
            .contains("\"accepted\": true")
    );
    // Already in place: nothing more is requested.
    core.start_semantic_download().await.unwrap();
    assert_eq!(srv.received_requests().await.unwrap().len(), requests.len());
}

#[tokio::test]
async fn a_damaged_unreachable_or_oversized_download_is_refused_safely() {
    let provider = FakeProvider::accepting();
    let meaning = Meaning::new();
    let attempt = |declared: Vec<(String, u64, String)>, served: &'static [u8], ranges: bool| {
        let provider = &provider;
        let meaning = &meaning;
        async move {
            let f = Fixture::new();
            let srv = server(
                &[("model_quantized.onnx", served), ("tokenizer.json", TOKENS)],
                ranges,
            )
            .await;
            let core = companion(&f, provider, meaning, &format!("{}/", srv.uri()), declared);
            std::fs::create_dir_all(model_dir(&f)).unwrap();
            std::fs::write(
                model_dir(&f).join("model_quantized.onnx.part"),
                &MODEL[..20],
            )
            .unwrap();
            let status = core.start_semantic_download().await.unwrap();
            (f, status.state)
        }
    };
    // A server that ignores the range sends the whole file, which starts over.
    let (f, state) = attempt(files(MODEL), MODEL, false).await;
    assert_eq!(state, SemanticState::Ready);
    assert_eq!(
        std::fs::read(model_dir(&f).join("model_quantized.onnx")).unwrap(),
        MODEL
    );
    // A file that doesn't match its fingerprint is deleted and never used (search FR-012, SC-009).
    let (f, state) = attempt(
        files(MODEL),
        b"model bytes, but not the ones that were tested",
        false,
    )
    .await;
    assert_eq!(
        state,
        SemanticState::NeedsAttention {
            reason: SemanticProblem::Damaged
        }
    );
    assert!(!model_dir(&f).join("model_quantized.onnx").exists());
    assert!(!model_dir(&f).join("model_quantized.onnx.part").exists());
    // Not enough room: refused before anything is requested or written (search FR-011).
    let huge = vec![("model_quantized.onnx".into(), u64::MAX / 4, sha(MODEL))];
    let (f, state) = attempt(huge, MODEL, true).await;
    assert_eq!(
        state,
        SemanticState::NeedsAttention {
            reason: SemanticProblem::NoSpace
        }
    );
    assert_eq!(
        std::fs::read(model_dir(&f).join("model_quantized.onnx.part")).unwrap(),
        &MODEL[..20]
    );
}

#[tokio::test]
async fn an_unreachable_source_or_a_cancel_keeps_what_was_downloaded() {
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let meaning = Meaning::new();
    let core = companion(&f, &provider, &meaning, "http://127.0.0.1:9/", files(MODEL));
    let status = core.start_semantic_download().await.unwrap();
    assert_eq!(
        status.state,
        SemanticState::NeedsAttention {
            reason: SemanticProblem::Unreachable
        }
    );

    // A slow server, and a cancel while it answers.
    let srv = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(MODEL)
                .set_delay(std::time::Duration::from_secs(30)),
        )
        .mount(&srv)
        .await;
    let f = Fixture::new();
    let core = companion(
        &f,
        &provider,
        &meaning,
        &format!("{}/", srv.uri()),
        files(MODEL),
    );
    let (status, ()) = tokio::join!(core.start_semantic_download(), async {
        while !matches!(
            core.semantic_status().unwrap().state,
            SemanticState::Downloading { .. }
        ) {
            tokio::task::yield_now().await;
        }
        core.cancel_semantic_download().unwrap();
    });
    let status = status.unwrap();
    assert_eq!(status.state, SemanticState::NotDownloaded);
    assert!(!status.offer, "accepted, so the offer stays away");
    assert!(core.remove_semantic_model().is_ok());
}

/// Runs the real model when `MYPEC_MODEL_DIR` points at its files: a manual smoke check, never in CI.
#[tokio::test]
#[ignore = "needs the real model files"]
async fn the_real_model_finds_meaning() {
    let Ok(source) = std::env::var("MYPEC_MODEL_DIR") else {
        return;
    };
    let f = Fixture::new();
    let provider = FakeProvider::accepting();
    let core = f.ready(&provider);
    note(
        &f,
        "2026-11-01",
        &[BISCUIT, CURTAINS, "The quarterly report is due on Friday."],
    );
    place_model(&f);
    for name in ["model_quantized.onnx", "tokenizer.json"] {
        std::fs::copy(
            std::path::Path::new(&source).join(name),
            model_dir(&f).join(name),
        )
        .unwrap();
    }
    let started = std::time::Instant::now();
    assert_eq!(
        core.semantic_tick().await.unwrap().state,
        SemanticState::Ready
    );
    println!("prepared in {:?}", started.elapsed());
    for (query, expected) in [
        ("my pup is scared of storms", "Biscuit"),
        ("anjing saya takut petir", "Biscuit"),
        ("new drapes for the lounge", "curtains"),
        ("work deadline this week", "quarterly"),
    ] {
        let started = std::time::Instant::now();
        let found = texts(&core, query);
        println!("{query}: {:?} in {:?}", found.first(), started.elapsed());
        assert!(found[0].contains(expected), "{query}: {found:?}");
    }
}
