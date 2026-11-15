# Semantic search

Built.

Today the companion finds a memory only when the words match. If the user once said "Biscuit gets anxious during thunderstorms" and later asks "how's my pup handling the storms?", keyword search finds nothing, so neither the companion's own search nor the automatic recall before a reply brings the note back. This feature adds search by meaning: a small model that runs on the user's own machine turns each piece of memory and each question into a representation of what it means, so related wording, synonyms and other languages find each other. Keyword search stays, and the two are blended.

## User stories

### US1. The companion finds what I meant, not just what I said (P1)

Three weeks ago the user told Steve that their dog Biscuit hides under the bed when it thunders. Tonight there is a storm, and the user writes "my pup is a wreck again". Before Steve replies, the note about Biscuit and thunder is recalled even though none of its words were used, and Steve asks whether Biscuit is under the bed. Later the user asks Steve directly what it knows about their dog's fears; Steve searches its memory and finds the same note. In the memory browser the user types "storm anxiety" and sees the note at the top.

**Why this priority**: This is the feature. Memory that only answers to exact words feels forgetful; memory that answers to meaning feels like being understood.

**How to check**: With the meaning model available and a seeded set of notes, ask questions phrased differently from the notes, in the conversation, through the companion's search and in the memory browser, and confirm the right note is among the top results each time, including for a question in another language than the note.

**Acceptance Scenarios**:

1. **Given** the meaning model is ready and a note exists whose words differ from the user's message, **When** the user sends a message about the same thing, **Then** automatic recall attaches that note within its usual budget.
2. **Given** the meaning model is ready, **When** the companion searches its memory with a question phrased unlike the note, **Then** the note is among the results it gets back, with its file, date and line as today.
3. **Given** the meaning model is ready, **When** the user searches in the memory browser, **Then** results that match by meaning appear alongside results that match by words, best first.
4. **Given** a note written in one language, **When** the user asks about it in another language, **Then** the note is found.
5. **Given** a question whose exact words appear in a note, **When** memory is searched, **Then** that note still ranks at least as high as it does with keyword search alone.
6. **Given** two near-identical notes, **When** both match, **Then** the results don't spend several places on the same thing.

### US2. I choose to turn it on, and know what it costs (P1)

After setup, the memory browser and settings mention, once and quietly, that Steve can also find memories by meaning if the user downloads a small model that runs on their computer. The offer says how large it is, where it comes from, and that their memories never leave the machine. The user agrees. A progress bar shows the download; the user closes the app halfway and it continues where it stopped next time. When it finishes, the app checks that the file is intact, prepares the existing memories in the background, and says that search by meaning is on. The user never has to do anything else.

**Why this priority**: The model is a download from a new place, and the principles keep every new network destination off until the user turns it on. Without a clear, honest way to turn it on, story 1 never happens.

**How to check**: On a fresh install confirm nothing is downloaded and search works by keyword; accept the offer and confirm the size and source were shown, progress is visible, an interrupted download resumes, a damaged file is rejected, and once ready the existing memories become searchable by meaning without blocking chat.

**Acceptance Scenarios**:

1. **Given** a fresh install, **When** the app runs, **Then** nothing is downloaded and every search works by keyword.
2. **Given** the user has not decided, **When** they open settings or the memory browser, **Then** they see the offer with the download size, its source and the promise that memory stays on the computer, and no other prompt, badge or reminder appears anywhere.
3. **Given** the user accepts, **When** the download runs, **Then** progress is shown, chat and keyword search work as before, and the user can cancel.
4. **Given** the download was interrupted (app closed, network lost), **When** the app runs again and the network is back, **Then** the download continues from where it stopped without asking again.
5. **Given** the download finished, **When** the file does not match what the app expects, **Then** it is discarded, the user is told plainly, and search stays keyword-only until the user tries again.
6. **Given** the model is in place, **When** existing memories are being prepared for search by meaning, **Then** this happens in the background, never delays a reply, and results by meaning cover more of the memory as it progresses.
7. **Given** the disk does not have room for the model, **When** the user accepts, **Then** the app says so before downloading and nothing is written.

### US3. Nothing breaks without it (P2)

The user declines the offer, or is offline, or the model file gets damaged, or the computer runs short of memory. In every case Steve keeps talking, recalls by keyword as before, and the memory browser's search still works. When something is wrong with the model, settings say so in plain words and offer to fix it; the conversation never shows an error about it.

**Why this priority**: Chat comes first (Principle V). Search by meaning is an improvement, never a dependency.

**How to check**: Run the same searches and conversations with the model absent, half-downloaded, damaged, switched off and failing to load, and confirm results by keyword, no conversation errors, and replies as fast as without this feature.

**Acceptance Scenarios**:

1. **Given** the model is absent, downloading, switched off, damaged or fails to load, **When** memory is searched in any of the four places, **Then** keyword results are returned as before.
2. **Given** the model fails while answering a search, **When** that happens during the preparation of a reply, **Then** the reply proceeds with keyword recall and no error appears in the conversation.
3. **Given** the model is damaged or keeps failing, **When** the user opens settings, **Then** they see what is wrong and can download it again or switch the feature off.
4. **Given** search by meaning is on, **When** a reply is prepared, **Then** the app's own work before sending stays within the existing budget (NFR-PERF-2).

### US4. Search stays true to my files (P2)

The user edits a daily note in a text editor, deletes a line, and forgets a memory in the memory browser. Search by meaning reflects each change: the edited line is found by its new meaning, the deleted line is gone, and the forgotten memory is found neither by words nor by meaning. When the user deletes the app's search data by hand, it is rebuilt from the files, by meaning too, without downloading anything again.

**Why this priority**: Files are the mind (Principle I) and forgetting must be complete (CTL-2). Search by meaning must not become a second, stale copy of memory.

**How to check**: Edit, delete and forget memory content, then search by meaning and confirm results match the files; delete the search data and confirm it is rebuilt with the same results.

**Acceptance Scenarios**:

1. **Given** a memory file was edited outside the app, **When** memory is next searched, **Then** results by meaning reflect the new text, as keyword results already do.
2. **Given** the user forgets a memory, **When** memory is searched by meaning, **Then** the forgotten text is not found and is not used for recall or dreaming.
3. **Given** the search data is deleted, **When** the app runs, **Then** it is rebuilt from the workspace files, by meaning too once the model is present, with no download.
4. **Given** a memory file is too long or unreadable, **When** it is prepared for search, **Then** the readable parts are searchable and the rest is reported as for keyword search today.

### US5. Switch it off, or take it back (P3)

The user wants the disk space back. In settings they switch search by meaning off and remove the model; search goes back to keywords and the space is freed. Months later they switch it on again, and it downloads again after showing the same offer. Another user only pauses it: switched off, the model stays on disk, and switching it back on works at once.

**Why this priority**: Control over what sits on one's machine is part of trust, but it is used rarely.

**How to check**: Switch off and confirm keyword-only results; switch on and confirm meaning results return without a download; remove the model and confirm the space is freed and turning it on again asks before downloading.

**Acceptance Scenarios**:

1. **Given** search by meaning is on, **When** the user switches it off, **Then** every search is keyword-only from then on and the model stays on disk.
2. **Given** it is off with the model on disk, **When** the user switches it on, **Then** results by meaning return without a download.
3. **Given** the user removes the model, **When** removal completes, **Then** the model's files and the meaning part of the search data are deleted and settings show it as not downloaded.
4. **Given** settings, **When** the user looks at search by meaning, **Then** they see whether it is on, its state (not downloaded, downloading with progress, preparing memories, ready, needs attention) and the space it uses.

## Edge cases

- **Offline at the moment of accepting.** The app says the download will start when the network is back, and starts then without asking again.
- **The user's network blocks the model's source.** The download fails with a plain message; the user can try again later. Nothing else changes.
- **Metered or slow connection.** The download can be cancelled at any time; a cancelled download keeps what it has so far until the user removes it, and resuming does not start from zero.
- **The app is updated with a different model.** Memories are prepared again for the new model in the background; until that finishes, results by meaning use what is ready and keyword search covers the rest. The new model is downloaded only after the user agrees again.
- **Very large memory** (ten years of daily notes, tens of thousands of pieces). Preparing it the first time takes a while and runs in the background in small steps; search stays within its time budget meanwhile (NFR-PERF-3).
- **Mixed and spaceless scripts** (Japanese, Thai, mixed English and Indonesian). Search by meaning handles them; keyword search keeps its existing handling.
- **A question that matches nothing by meaning or by words.** No results, as today; weak matches by meaning alone are not shown just to fill places.
- **The app sits in the tray for hours.** The model is not kept in memory while nothing searches, so the idle footprint stays within NFR-PERF-4.
- **A file changes while it is being prepared.** The newer version is prepared; the older one is never left behind in results.
- **The dream diary, the soul and tool results.** They stay out of search, by meaning as by keyword, as today.
- **Dreaming runs while memories are still being prepared.** Recall evidence counts what search actually returned, by words or by meaning; nothing waits.
- **The user forgets a memory while the model is downloading or preparing.** The forgotten text is never prepared afterwards.

## Requirements

### Functional requirements

**Finding by meaning**

- **FR-001**: When the meaning model is ready and switched on, memory search MUST find memory by meaning as well as by keyword and MUST blend both into one ranked list (MEM-6).
- **FR-002**: The blended ranking MUST keep a result that matches the query's words at least as high as keyword search alone would place it relative to results that match neither.
- **FR-003**: The blended ranking MUST keep the existing rules: daily-note results lose weight with age, near-duplicate results are thinned so the top results are diverse, and weak matches by meaning alone below a relevance floor are not returned.
- **FR-004**: Search by meaning MUST work across languages the model supports, so a question in one language finds a note in another (NFR-I18N-1).
- **FR-005**: The same search MUST serve all four places memory is searched today: the companion's memory search, automatic recall before a reply, the memory browser's search, and the recall evidence dreaming collects. Each keeps its own result count and budget.
- **FR-006**: Results MUST carry the same information as today: file, date, line and snippet. A result MAY say whether it matched by words, by meaning or both; the companion's result format MUST NOT otherwise change.
- **FR-007**: Recall evidence for dreaming MUST count hits found by meaning like hits found by words, with a relevance taken from the blended score on the same 0-to-1 scale (dreaming R1).

**Getting the model**

- **FR-008**: The model MUST NOT be downloaded until the user explicitly agrees. Before agreeing, the user MUST be told the download size, the source, and that their memories are processed only on their computer.
- **FR-009**: The offer MUST appear in settings and in the memory browser, until the user accepts or dismisses it. It MUST NOT appear as a notification, a badge, a chat message or a repeated reminder (NFR-SEC-6).
- **FR-010**: The download MUST run in the background, show progress, be cancellable, resume after an interruption or a restart without asking again, and never slow a reply.
- **FR-011**: The app MUST check free disk space before downloading and refuse with a plain message when there is not enough.
- **FR-012**: The downloaded model MUST be verified against a fingerprint shipped with the app before it is used. A model that fails the check MUST be deleted and MUST never be loaded.
- **FR-013**: The only new network destination MUST be the model's download source, named in the Assumptions. It MUST be contacted only while a download the user agreed to is running. Search itself MUST never use the network (Principle II, NFR-SEC-1).

**Preparing memory**

- **FR-014**: Once the model is ready, all memory that keyword search covers MUST be prepared for search by meaning in the background, in small steps, newest material first. Search MUST stay usable throughout, using meaning for what is ready and words for everything.
- **FR-015**: New and changed memory MUST be prepared for search by meaning as it is today for keyword search: picked up from the files before a search, including edits made outside the app (MEM-8).
- **FR-016**: Search by meaning MUST be derived only from the workspace files and MUST be rebuildable from them at any time without a new download (NFR-REL-2).
- **FR-017**: Forgetting MUST remove the content from search by meaning at the same moment as from the files and keyword search, and forgotten content MUST never be prepared again (CTL-2, Principle IV).
- **FR-018**: The soul, the dream diary, the conversation transcript and tool results MUST stay out of search by meaning, exactly as they are out of keyword search.
- **FR-019**: When the model changes with an app update, memory MUST be prepared again for the new model in the background; results from the old model MUST NOT be mixed with the new one.

**Reliability and control**

- **FR-020**: Whenever the model is absent, downloading, preparing, switched off, damaged or failing, every search MUST return keyword results, and no conversation MUST show an error because of it (Principle V).
- **FR-021**: A search by meaning that takes too long or fails MUST fall back to keyword results within the existing time budgets (NFR-PERF-2, NFR-PERF-3).
- **FR-022**: The model MUST be loaded only when a search needs it and released after a few idle minutes (NFR-PERF-4).
- **FR-023**: Settings MUST show search by meaning with its switch, its state (not downloaded, downloading with progress, preparing with progress, ready, needs attention with the reason), and the disk space it uses.
- **FR-024**: Users MUST be able to switch search by meaning off (keeping the model), switch it on again without a download, and remove the model, which deletes its files and the meaning part of the search data.
- **FR-025**: A model the user removed or never downloaded MUST be offered again only when the user turns the feature on, and MUST again show size and source before downloading.
- **FR-026**: Download, preparation, loading, searching and fallback MUST be decided in the core and testable with a stand-in model, with no download in tests (Principle VI).

### Key entities

- **Meaning model**: A small model, downloaded once, that runs on the user's computer and turns text into a representation of its meaning. It has a version, a size, a source and a fingerprint the app checks.
- **Piece of memory**: The part of a memory file that search returns (as keyword search already splits files). Each piece gains a representation of its meaning, tied to the model version that made it.
- **Blended result**: A search result with its keyword score, its meaning score where there is one, and the combined rank.
- **Download**: The one-time transfer of the model, with progress, a resumable partial file, and its verification.
- **Preparation**: The background work that gives every piece of memory its representation of meaning, with progress.
- **State of search by meaning**: Not downloaded, downloading, preparing, ready, switched off, or needs attention (and why).

## Success criteria

- **SC-001**: On a scripted set of 40 questions phrased differently from the notes that answer them, the right note is among the top 3 results for at least 85% with search by meaning on, against at most 30% with keyword search alone.
- **SC-002**: On a scripted set of 20 questions in a different language from their notes (English, Indonesian, Japanese, Spanish), the right note is among the top 5 results for at least 75%.
- **SC-003**: On a scripted set of 40 questions that share their key words with a note, the right note's rank is never worse than with keyword search alone.
- **SC-004**: With 10,000 pieces of memory prepared, 95% of searches return within 100 ms once the model is loaded (NFR-PERF-3), and the app's work before sending a message stays within 150 ms at the 95th percentile (NFR-PERF-2).
- **SC-005**: With the model absent, downloading, switched off, damaged or failing to load, 100% of searches in all four places return keyword results and 0 conversation errors appear.
- **SC-006**: After an hour in the tray with search by meaning on, the idle footprint meets NFR-PERF-4.
- **SC-007**: After a memory is forgotten, it is returned by 0 searches, by words or by meaning, and appears in 0 recalls or dream inputs.
- **SC-008**: Deleting the search data and running the app gives the same top 10 results for a scripted set of 20 questions as before, with no download.
- **SC-009**: 100% of damaged or substituted model files are rejected before use.
- **SC-010**: Preparing 10,000 pieces of memory the first time completes within 15 minutes on a mid-range laptop while the user chats, with replies no slower than without it.
- **SC-011**: During a network capture of a fresh install used for a day without accepting the offer, the app contacts only the provider; after accepting, it contacts only the provider and the model's source, and the source only while the download runs.
- **SC-012**: In the scripted week-long recall suite (SC-2 of the requirements), facts recalled correctly in a fresh context rise to at least 28 of 30.

## Assumptions

- **Depends on the memory page (memory core).** The memory files, the pieces keyword search splits them into, keyword search, automatic recall, the memory browser's search, the companion's memory search tool, forgetting and the index rebuild come from it. Recall evidence comes from the dreaming page.
- **The download is opt-in.** The the principles allow the one-time model download as an outbound connection, but keeps every new network destination off until the user turns it on. The architecture's wording, "downloaded in the background after first run", is read as "after the user agrees, in the background".
- **The source.** The model is downloaded from Hugging Face's public file servers (`huggingface.co` and the file delivery hosts it redirects to), from a fixed model and revision named in R1 and R7. The app ships the expected fingerprint, so a changed or substituted file is rejected.
- **The model.** A small multilingual text model, on the order of 100 to 250 MB on disk, that runs on an ordinary laptop's processor without a graphics card. The choice (R1) is made against SC-001, SC-002 and SC-004.
- **No account, no key.** The download needs no sign-in, and Claude's API is not used for this; it has no such service.
- **The offer.** Shown in settings and in the memory browser after setup until accepted or dismissed. Dismissing it hides it; the switch in settings stays available.
- **Partial results while preparing.** Until all memory is prepared, results by meaning cover what is ready, and keyword search covers everything, so nothing is lost in the meantime.
- **Relevance floor and blend weights** start from the architecture's hybrid search and are tuned in planning against SC-001 to SC-003.
- **What is searched stays the same**: long-term memory and daily notes. Search by meaning adds no new sources.
- **Out of scope**: the avatar (slice 6), release readiness and auto-update (slice 8), searching the conversation transcript by meaning, a choice of models in settings, a hosted embeddings service, and graphics-card acceleration.
- **Product documents to update with this feature's plan**: `docs/architecture.md` sections 2 and 6.2 (the opt-in download and its source, the fingerprint check, preparation and its states) and section 15 if new defaults are introduced. `docs/requirements.md` needs no change: MEM-6 and NFR-SEC-1 already describe this.
- **Platform.** Windows 11 first, as for the rest of the app.

## Decisions

Each section records a decision, why it was made, and what was rejected. The design follows `docs/architecture.md` sections 2 and 6.2 and builds on the memory page (the index in `steve.db`, chunks, FTS5 keyword search, ranking, auto-recall, forgetting) and the dreaming page (recall evidence, the webview tick that drives background work).

### R1. The model

**Decision**: `intfloat/multilingual-e5-small`, in the int8-quantized ONNX export published as `Xenova/multilingual-e5-small` (`onnx/model_quantized.onnx`, about 118 MB, and `tokenizer.json`, about 17 MB), at a pinned revision. It is 384-dimensional, covers about 100 languages, and is MIT-licensed. Text is embedded with the model's prefixes (`query: ` for a search, `passage: ` for a chunk), mean pooling and L2 normalization, so the dot product of two vectors is their cosine similarity. Chunks are at most 600 characters (memory R11), well within the model's 512 tokens.

**Rationale**: Small enough for a laptop processor (tens of milliseconds for a query, SC-004) and a download of about 135 MB (Assumptions: 100 to 250 MB), multilingual by training (FR-004, SC-002), with a permissive license, and among the strongest small multilingual retrieval models. The quantized export is what makes the size fit.

**Alternatives considered**: `paraphrase-multilingual-MiniLM-L12-v2` (similar size, weaker on retrieval). `bge-m3` (much stronger, but about 570 M parameters and over 500 MB even quantized). English-only models (fail SC-002).

### R2. Running it

**Decision**: The `ort` crate (ONNX Runtime) for inference and the `tokenizers` crate for the model's own `tokenizer.json`, behind a small `Embedder` trait in the core with two implementations: the ONNX one and a deterministic stand-in for tests. ONNX Runtime's library ships with the app (about 10 MB, from `ort`'s prebuilt binaries at build time); nothing about the runtime is downloaded at run time. Batches of passages are embedded on a blocking worker (`spawn_blocking`). Auto-recall embeds its single short query inline in the send path, before the reply starts streaming and within the 150 ms preparation budget (R5); no inference ever runs on the thread that streams a reply (Principle V).

**Rationale**: ONNX Runtime is the reference runtime for the export and runs the int8 model fast on any x64 processor. The trait is the lean-code exception the skill allows: a second implementation and a test boundary that cannot use the real one (the real one needs the download, FR-026).

**Alternatives considered**: `fastembed` (wraps `ort` and `tokenizers` but brings its own Hugging Face downloader and cache, which would bypass consent, the fingerprint check and our progress). `candle` or `tract` in pure Rust (no native library, but weaker support for the quantized XLM-RoBERTa graph and slower int8 inference).

**Measured during implementation**: ONNX Runtime is linked statically; its DirectML provider is loaded only on demand and never used, so nothing ships beside the app. Its telemetry is switched off when the environment is created (Principle II). The prebuilt runtime needs a processor with AVX2 (Intel since 2013, AMD since 2015); an older one is a known limit, recorded for release readiness. A query embeds in about 7 ms and three notes prepare in under 3 seconds including the model's load on the development laptop.

### R3. Storing vectors

**Decision**: A derived table `chunk_vectors (text_hash TEXT, model TEXT, vector BLOB, PRIMARY KEY (text_hash, model))` in `steve.db`, keyed by the fingerprint of the chunk's text, with `chunks` gaining a `text_hash` column. A search reads the vectors of the current model and scores them in Rust by dot product (brute force), keeping the vectors in memory while the model is loaded. Vectors are 384 little-endian `f32` values (1,536 bytes).

When a file is re-indexed, its chunks are replaced as today; vectors whose text no longer appears in any chunk are deleted in the same transaction. An edit to one line therefore re-embeds only the changed chunks, and forgotten text leaves the table at once (FR-017).

**Rationale**: At 10,000 chunks a brute-force scan is about 4 million multiply-adds, a few milliseconds, well under NFR-PERF-3. Ten years of daily notes (tens of thousands of chunks) is still tens of milliseconds. A plain table needs no SQLite extension and no new dependency, and it is rebuilt with the rest of the derived data (FR-016). Keying by text hash makes edits cheap and makes "no chunk holds this text any more" the deletion rule.

**Alternatives considered**: The `sqlite-vec` extension, as architecture section 2 names it (an extra native extension for a scan the core does faster in memory at this scale; revisit beyond 100,000 chunks). Keying by chunk id (ids change at every re-index, so any edit would re-embed the whole file).

### R4. Blending the two scores

**Decision**: A search collects up to 200 keyword candidates (as today) and the 50 nearest chunks by meaning, then scores each candidate:

```text
keyword  = bm25 score ÷ the best bm25 score of this query × the share of the query's terms the chunk holds
meaning  = (cosine − 0.78) ÷ (0.86 − 0.78), clamped to 0 to 1     (0 at the floor, 1 at the ceiling)
blended  = 0.35 × keyword + 0.65 × meaning
```

The existing `rank` then applies the daily-note age decay and the near-duplicate thinning (FR-003). A chunk whose meaning is under the floor and that matches no word is not a candidate. With the model unavailable, `meaning` is 0 everywhere and the order is exactly today's keyword order (FR-020). Each hit keeps `matched` (query terms it contains) and gains `meaning`; its **relevance** is `max(matched ÷ terms, meaning)`, on the 0-to-1 scale recall evidence already uses (FR-007).

**Rationale**: A weighted sum over per-query normalized scores is how the architecture describes hybrid search and how OpenClaw's memory search blends (vector 0.7, text 0.3). E5 similarities cluster high (unrelated text often scores 0.7 to 0.75), so a floor turns cosine into a usable relevance and keeps weak matches out (FR-003). Because results matching no word and with meaning under the floor score 0, a keyword match can never fall below such a result (FR-002); SC-003 checks that exact-word matches don't lose rank in practice. The floor and the ceiling were measured with the real model during implementation: unrelated notes score a cosine of 0.74 to 0.78 and related ones 0.80 to 0.85, so a 0-to-1 range would leave true matches near 0.2. The term share keeps one shared common word ("is") from outweighing a clear match by meaning; it applies only when meaning scores are present, so without the model the order is exactly the keyword order (FR-020). The weights, the floor and the ceiling are tuned further against SC-001 to SC-003 before release.

**Alternatives considered**: Reciprocal rank fusion (ignores how strong each match is, so a weak meaning match can outrank a strong keyword one). Meaning only (loses exact names and numbers).

### R5. Auto-recall with meaning

**Decision**: The recall gate becomes: a hit is worth attaching when it contains two of the query's terms (or its only term), as today, **or** its `meaning` is at least 0.4 (a cosine of about 0.81). The other rules stay (not already in the context, within 3 snippets and 2,000 characters). The query is embedded only when the model is already loaded and the vectors are in memory; otherwise this recall is keyword-only and the core starts loading the model in the background for the next one. Embedding a message takes tens of milliseconds, within NFR-PERF-2's 150 ms for the app's own work (SC-004).

**Rationale**: Loading the model takes longer than the whole recall budget, so the send path never waits for it (FR-021). People send messages in runs, so the second message of a conversation already has meaning recall. Recall evidence records the relevant hits as today, with the new relevance as their coverage (FR-007).

**Alternatives considered**: Loading the model on demand inside the send path (blows the budget once per idle period). Keeping the model loaded always (fails NFR-PERF-4 in the tray).

### R6. Loading and releasing

**Decision**: The model and the vectors are loaded on first need (a search, recall's background warm-up, or preparation) and released after 10 minutes without use. The companion's `memory_search` tool and the memory browser's search wait for the load (well under a second on a laptop, never on the reply's streaming thread); auto-recall never waits (R5). A load or inference failure is recorded as the state "needs attention" with its reason, and the search falls back to keyword results (FR-020). A failure is retried only after the user acts or the app restarts. Whether the idle time has passed is a pure check on the last use and the current time, so it is tested without waiting.

**Rationale**: Architecture 6.2's "loaded on demand and released after a few idle minutes" keeps the tray footprint low (SC-006). Ten minutes covers a normal pause in a conversation.

### R7. The download

**Decision**: `start_semantic_download` (async) fetches the two files with the existing `reqwest` client from `https://huggingface.co/Xenova/multilingual-e5-small/resolve/<revision>/<file>`, following redirects to Hugging Face's file-delivery hosts, into `state/models/multilingual-e5-small/<file>.part`. It resumes with an HTTP `Range` request from the partial file's length, writes in chunks with progress kept in memory, and is stopped by `cancel_semantic_download` (the stop-signal pattern of compaction and dreaming). Before it starts, it checks the free space on the state folder's drive against the files' total size plus 10% (the `fs4` crate). When both files are complete, each is checked against the SHA-256 shipped in the app (the `sha2` crate); a mismatch deletes the file and sets "needs attention" (FR-012). Matching files are renamed into place and a `model.json` records the model and revision. The download never runs unless the user accepted the offer (`settings.json search.semantic.accepted`, FR-008).

Free space, resume, cancellation and verification are decided in the core, behind a small `ModelSource` seam so tests serve the files from a local server.

**Rationale**: The client, TLS and redirects already exist for the Claude API. Resuming with `Range` is supported by the file hosts and keeps a slow connection from starting over (Edge cases). Shipping the fingerprint is what turns "downloaded from the internet" into "the exact file we tested" (FR-012, SC-009). Models live in `state/` because they are not the companion's data and can always be downloaded again.

**Alternatives considered**: Hugging Face's `hf-hub` crate (another dependency, its own cache layout and no progress hook). Shipping the model in the installer (adds 135 MB for everyone, including users who never want it).

### R8. Preparation

**Decision**: `semantic_tick` (async), called by the main view after the download completes and on its existing minute tick (dreaming R2), embeds the chunks that have no vector for the current model, in batches of 32, `MEMORY.md` first and then daily notes newest first, until none are left, until a message arrives (the chat runtime's notify, as dreaming yields), or for at most 20 seconds a call. Each batch embeds on a blocking worker and writes its vectors in one transaction, each row only while some chunk still holds its text (an insert guarded by `EXISTS`), so a note edited or forgotten while the batch was embedding never leaves a vector behind (FR-017). Searches use whatever vectors exist (FR-014). The search path itself also embeds up to 8 missing chunks when the model is already loaded, so a note just written or edited is found by meaning on the next search without waiting for the tick (FR-015).

**Rationale**: The webview already knocks every minute, so preparation needs no thread or timer of its own in the core (lean-code). At about 20 ms a chunk in batches, 10,000 chunks take a few minutes of ticks, within SC-010, and each call stops as soon as the user speaks (Principle V).

**Alternatives considered**: A background thread in the core (a `'static` core and a runtime handle, against the lean-code rule on concurrency; the dream tick already solved this). Embedding at index time inside `refresh` (would put model work on every search's critical path).

### R9. A new model with an update

**Decision**: Vectors are stored with their model name (R3). When an app version ships a different model, the old vectors are ignored, the old model is kept until the user accepts the new one's offer, and after the new download, preparation re-embeds everything for the new model while the old rows are deleted. Results never mix models (FR-019).

### R10. Settings, state and the offer

**Decision**: `settings.json` gains `search.semantic`:

- `accepted`: the user agreed to the download (default `false`, Principle II);
- `enabled`: search by meaning is used when ready (default `true`, so accepting is enough);
- `offerDismissed`: the user dismissed the offer (default `false`).

The state shown to the user is worked out, not stored: not downloaded (no verified files), downloading (with bytes and total), preparing (with chunks done and total), ready, off (`enabled` false), needs attention (the last failure, kept in memory). The offer shows in settings and in the memory browser while `accepted` and `offerDismissed` are both false (FR-009); it states the size (from the shipped manifest) and the source.

**Rationale**: Accepting is the consent the principles ask for; switching off without removing keeps the model for later (FR-024). Working the state out from the files means a deleted model folder is simply "not downloaded".

### R11. Forgetting and rebuilding

**Decision**: Forgetting re-indexes the file (the memory page), which replaces its chunks; the vectors of text no chunk holds any more are deleted in the same transaction (R3), so forgotten text is gone from search by meaning the moment it leaves the file (FR-017, SC-007). Deleting `steve.db` rebuilds keyword search at once and search by meaning through preparation, with no download, because the model lives in `state/models/` (FR-016, SC-008). The derived schema version is bumped, so existing databases rebuild once with the new `text_hash` column; `preimages`, `recall_events` and `dream_runs` are kept as before.

### R12. The webview

**Decision**: A "Search by meaning" section in settings shows the switch, the state with a progress bar while downloading or preparing, the space used, and the offer, a "Download" button, "Cancel", "Try again" and "Remove". The memory browser shows the same offer, once, above the search box, with "Download" and "Not now". The webview polls `semantic_status` every second while a download or preparation runs, and calls `semantic_tick` after a download completes and from the main view's minute tick. No notification, badge or chat message anywhere (FR-009).

### R13. Tests

**Decision**: The stand-in embedder maps each word to a fixed vector and a small synonym table to the same vectors ("dog", "pup", "Biscuit"; "storm", "thunder"; "anjing" and "dog"), sums and normalizes, so paraphrase and cross-language cases are deterministic. Core tests cover the blend, the floor, recall's gate, preparation order and yielding, forgetting, rebuild, model change, every failure state, and the download (resume, cancel, disk space, fingerprint mismatch) against a local file server. The real model is used only in the release checks (SC-001, SC-002, SC-004, SC-006, SC-010, SC-012), never in CI (the principles test gates).

### R14. Product documents

**Decision**: Updated with this plan: `docs/architecture.md` section 2 (the embeddings row: `ort` and `tokenizers`, the opt-in download, vectors in a plain table instead of `sqlite-vec`), section 4.2 (`chunk_vectors`, `chunks.text_hash`, `state/models/`), section 6.2 (the blend, the floor, recall's gate, preparation, loading and release, the offer) and section 15 (`search.semantic.*`, the weights and the floor). `docs/requirements.md` needs no change.

## Data

What this slice adds to `steve.db`, the state folder, the settings file, the bundled manifest, and the types that cross to the webview. The workspace files are unchanged: search by meaning is derived from them (FR-016).

### Data 1. `state/steve.db` (derived tables)

The derived schema version goes from `1` to `2`, so an existing database drops and rebuilds its derived tables once; `preimages`, `recall_events` and `dream_runs` are kept (R11).

#### `chunks` (changed)

| Column | Change |
|---|---|
| `text_hash` | New. `fingerprint` of the chunk's `text`. Indexed. |

#### `chunk_vectors` (new)

| Column | Type | Rules |
|---|---|---|
| `text_hash` | TEXT | A chunk text's fingerprint. |
| `model` | TEXT | The model id and revision that made the vector, e.g. `multilingual-e5-small@<revision>`. |
| `vector` | BLOB | 384 little-endian `f32`, L2-normalized (1,536 bytes). |

Primary key `(text_hash, model)`. Rules: a row exists only while some chunk has its `text_hash` (deleted in the re-index transaction that removes the last such chunk, R3); rows of another model are ignored by search and deleted after the current model's preparation completes (R9). Never holds the soul, the diary, the transcript or tool results, because `chunks` never does (FR-018).

### Data 2. `state/models/` (new)

```text
state/models/multilingual-e5-small/
  model_quantized.onnx        verified, about 118 MB
  tokenizer.json              verified, about 17 MB
  model.json                  {"model": "multilingual-e5-small", "revision": "<revision>"}
  *.part                      a download in progress; resumed from its length
```

A model counts as downloaded when `model.json` names the bundled manifest's model and revision and both files are present. Files are checked against their SHA-256 once, when the download completes (R7). Removing the model deletes the folder.

### Data 3. Bundled manifest (`crates/steve-core/src/memory/model.json`, new)

```json
{
  "model": "multilingual-e5-small",
  "revision": "<pinned commit>",
  "source": "https://huggingface.co/Xenova/multilingual-e5-small/resolve/<pinned commit>/",
  "files": [
    {"path": "onnx/model_quantized.onnx", "name": "model_quantized.onnx", "bytes": 118000000, "sha256": "…"},
    {"path": "tokenizer.json", "name": "tokenizer.json", "bytes": 17000000, "sha256": "…"}
  ],
  "dimensions": 384,
  "queryPrefix": "query: ",
  "passagePrefix": "passage: "
}
```

The revision, byte counts and SHA-256 values are pinned during implementation from the repository's file pointers and never change for a shipped app version.

### Data 4. `settings.json`

```json
{"search": {"semantic": {"accepted": false, "enabled": true, "offerDismissed": false}}}
```

| Field | Rules |
|---|---|
| `search.semantic.accepted` | Default `false`. Set by `start_semantic_download`; cleared by `remove_semantic_model`. No download runs while it is `false` (FR-008). |
| `search.semantic.enabled` | Default `true`. When `false`, every search is keyword-only and the model stays on disk (FR-024). |
| `search.semantic.offerDismissed` | Default `false`. Hides the offer; the settings section stays (FR-009). |

### Data 5. In-memory state (core)

| What | Rules |
|---|---|
| Download | Bytes done and total, while running; a stop signal for cancel. |
| Loaded model | The embedder and the current model's vectors, with the time of last use; released after 10 minutes unused (R6). |
| Last failure | The reason the model needs attention (verification, load, inference), until the user retries or the app restarts. |

### Data 6. Constants (core)

| Name | Value | Source |
|---|---|---|
| Keyword candidates | 200 | the memory page |
| Meaning candidates | 50 | R4 |
| Weights | keyword 0.35, meaning 0.65 | R4 |
| Meaning floor and ceiling | cosine 0.78 and 0.86 | R4 |
| Recall by meaning | meaning ≥ 0.4 | R5 |
| Idle release | 10 minutes | R6 |
| Preparation batch | 32 chunks; at most 20 seconds a tick | R8 |
| Embedded on search | at most 8 missing chunks | R8 |
| Free space margin | 10% over the files' size | R7 |

### Data 7. Types that cross to the webview (generated with `ts-rs`)

```rust
#[serde(tag = "kind")]
pub enum SemanticState {
    NotDownloaded,
    Downloading { done: u64, total: u64 },
    Preparing { done: u32, total: u32 },
    Ready,
    Off,
    NeedsAttention { reason: SemanticProblem },
}

pub enum SemanticProblem { Damaged, NoSpace, Unreachable, CouldNotLoad }

pub struct SemanticStatus {
    pub state: SemanticState,
    /// Shown while the user has neither accepted nor dismissed it (FR-009).
    pub offer: bool,
    pub download_bytes: u64,      // the manifest's total, for the offer
    pub source: String,           // "Hugging Face (huggingface.co)"
    pub disk_bytes: u64,          // what the model folder uses now
}
```

`SearchHit` (memory browser) is unchanged (FR-006).

## Interfaces

### Semantic Search Commands Between the Webview and the Core

Same rules as the pages onboarding, chat, memory, world, compaction, dreaming and journey: thin handlers, validation in the core, expected failures as values, each command in `build.rs` and allowed in `src-tauri/capabilities/main.json`, and `src/api/commands.ts` as the only caller. The only network destination this feature adds is the model's download source (see the model download below), contacted only by `start_semantic_download`.

#### New commands

| Command | Input | Output | Behavior | Requirements |
|---|---|---|---|---|
| `semantic_status` | — | `SemanticStatus` | The state, the offer flag, the download size and source, and the disk space used. No network, no model load. | FR-023 |
| `start_semantic_download` | — | `SemanticStatus` | Async. Records the user's consent (`accepted`), checks free space, then downloads (resuming a partial file) and verifies. Returns when the download is complete and verified, cancelled, or failed; the failure is in the state (`NeedsAttention`), not an error. Refused as `InvalidInput("busy")` while a download runs; returns at once with `Ready` or `Off` when the model is already in place. | FR-008, FR-010 to FR-013 |
| `cancel_semantic_download` | — | `SemanticStatus` | Stops a running download at its next write; the partial file is kept for resuming. No-op otherwise. | FR-010 |
| `set_semantic_search` | `enabled: boolean` | `SemanticStatus` | Switches search by meaning on or off. On with no model in place does not download: the state stays `NotDownloaded` and the offer shows. | FR-024 |
| `remove_semantic_model` | — | `SemanticStatus` | Refused as `InvalidInput("busy")` while a download runs. Deletes the model folder and every row of `chunk_vectors`, releases the loaded model, and clears `accepted`. | FR-024, FR-025 |
| `dismiss_semantic_offer` | — | `SemanticStatus` | Sets `offerDismissed`. | FR-009 |
| `semantic_tick` | — | `SemanticStatus` | Async. When the model is in place and enabled, prepares missing vectors in batches until none are left, a message arrives, or 20 seconds pass (R8). Never downloads. Returns at once when there is nothing to do or another tick runs. | FR-014, FR-015, FR-019 |

#### Changes to earlier commands and tools

| Command or tool | Change |
|---|---|
| `memory_search` (memory browser) | Blended results when the model is ready and enabled (waits for a load); keyword results otherwise. Same `SearchHit` shape. |
| Companion tool `memory_search` | The same blend; same result format (FR-006). Its recall evidence uses the new relevance (FR-007). |
| `send_message`, `retry_reply`, `regenerate_reply` | Auto-recall uses meaning when the model is already loaded; otherwise keyword-only for this message and a background load starts (R5). Recall evidence uses the new relevance. |
| `memory_forget`, `memory_save` | Unchanged behavior; vectors of text no chunk holds any more are deleted in the same re-index (FR-017). |

#### Webview duties

- Settings: a "Search by meaning" section with the switch, the state and its progress bar, the space used, the offer (size, source, "your memories stay on this computer") and the actions "Download", "Cancel", "Try again" and "Remove the model" (with confirmation).
- Memory browser: the same offer above the search box while `offer` is true, with "Download" and "Not now".
- While the state is `Downloading` or `Preparing`, poll `semantic_status` every second; after a download completes, call `semantic_tick`; the main view also calls `semantic_tick` from its minute tick.
- No notification, badge, reminder or chat message about search by meaning (FR-009).

#### Capability file additions

`allow-semantic-status`, `allow-start-semantic-download`, `allow-cancel-semantic-download`, `allow-set-semantic-search`, `allow-remove-semantic-model`, `allow-dismiss-semantic-offer`, `allow-semantic-tick`.

### The Model Download and Its Use

#### 1. Destination

The only new outbound destination, contacted only while `start_semantic_download` runs after the user accepted (FR-008, FR-013, the Principle II):

| Request | Value |
|---|---|
| Method | `GET`, following redirects |
| URL | `https://huggingface.co/Xenova/multilingual-e5-small/resolve/<pinned revision>/<file path>` for each manifest file |
| Redirect hosts | Hugging Face's file-delivery hosts (for example `cdn-lfs.huggingface.co`, `cas-bridge.xethub.hf.co`) |
| Headers | `User-Agent: mySteve/<version>`; `Range: bytes=<partial length>-` when resuming |
| Sent | Nothing else: no account, no key, no memory content, no identifier |

#### 2. Outcomes

| Server or local result | Behavior |
|---|---|
| `200` (or `206` to a range request) | Bytes appended to `<file>.part`; progress updated. |
| `200` to a range request | The server ignored the range: the partial file is truncated and written from the start. |
| `416` | The partial file is already complete: go to verification. |
| Network error, timeout, any other status | State `NeedsAttention { Unreachable }`; the partial file is kept; "Try again" resumes. |
| Free space below the files' total plus 10% | State `NeedsAttention { NoSpace }` before any request; nothing written. |
| SHA-256 differs from the manifest | The file is deleted; state `NeedsAttention { Damaged }`. |
| Both files verified | Renamed into place, `model.json` written, state `Preparing` (or `Off` when disabled). |
| Cancelled | Stops at the next write; the partial file is kept; state `NotDownloaded` with the offer accepted. |

#### 3. Use of the model

| Step | Rule |
|---|---|
| Passage | `passage: ` + the chunk's text as indexed (without comments), truncated to 512 tokens. |
| Query | `query: ` + the search text, truncated to 512 tokens. |
| Output | Mean of the last hidden state over the attention mask, L2-normalized; 384 `f32`. |
| Threading | Inference on a blocking worker, never on a reply's streaming thread. |
| Failure | Load or inference error: state `NeedsAttention { CouldNotLoad }`, keyword results for that search, no conversation error (FR-020). |

#### 4. Test stand-ins

- `Embedder`: the ONNX implementation and a deterministic stand-in (R13).
- `ModelSource`: the Hugging Face base URL in the app; a local file server in tests, serving the manifest's files, a truncated file, a range-ignoring server and a damaged file.

## Checks

### Prerequisites

- The pages onboarding, chat, memory, world, compaction, dreaming and journey set up, or a clean machine to run the wizard.
- Rust 1.98 or later, Node.js 22 or later, pnpm 10.
- For the manual scenarios: about 150 MB of free disk space and an internet connection for the one-time model download.

### Inspect the files

- `%APPDATA%\mySteve\state\models\multilingual-e5-small\`: the model files and `model.json` after a download.
- `%APPDATA%\mySteve\state\steve.db`: `chunk_vectors` (any SQLite browser).
- `%APPDATA%\mySteve\settings.json`: `search.semantic`.

### Manual validation scenarios

| # | Story | Do | Expect |
|---|---|---|---|
| 1 | US2 | A fresh install: use the app for a while without touching settings. | Nothing downloaded (no `state\models`); search works by keyword. |
| 2 | US2 | Open settings and the memory browser. | The offer, with the size, the source and "your memories stay on this computer", in both places; nothing else anywhere. |
| 3 | US2 | Choose "Download", then close the app halfway and reopen it. | Progress shown; after reopening, the download continues from where it stopped. |
| 4 | US2 | Let it finish. | "Preparing" with progress, then "Ready"; chat stays responsive throughout. |
| 5 | US1 | Tell Steve "Biscuit hides under the bed when it thunders". A few days later (or after a fresh context) write "my pup is a wreck again, there's a storm". | Steve brings up Biscuit and thunder. |
| 6 | US1 | Ask Steve what it knows about your dog's fears. | Its memory search finds the note. |
| 7 | US1 | In the memory browser search "storm anxiety", then "anjing takut petir". | The note is found both times. |
| 8 | US1 | Search for an exact name you noted. | It is still at the top. |
| 9 | US3 | Switch search by meaning off and repeat scenario 7. | Keyword results only; no error. |
| 10 | US3 | Close the app, replace `model_quantized.onnx` with any other file, reopen, search. | Settings show the model needs attention (damaged) with "Try again"; search works by keyword; chat shows nothing. |
| 11 | US3 | Turn the network off and choose "Download" (after removing the model). | A plain message that the source can't be reached; nothing else changes. |
| 12 | US4 | Edit a daily note in a text editor so a line means something new; search for the new meaning. | Found. |
| 13 | US4 | Forget the Biscuit note in the memory browser; repeat scenarios 5 and 7. | Not found and not recalled. |
| 14 | US4 | Close the app, delete `state\steve.db`, reopen, wait a minute, search by meaning. | Results return; nothing downloads. |
| 15 | US5 | Choose "Remove the model". | The folder is gone, the space freed, the state "not downloaded" and the offer back. |
| 16 | Edge | Leave the app in the tray for an hour with search by meaning ready. | Memory use within the idle budget (the model was released). |

### Release checks (real model, owner)

| Check | How | Pass |
|---|---|---|
| SC-001 | 40 paraphrased questions against seeded notes, with and without search by meaning. | Right note in the top 3 for at least 85% with meaning; at most 30% with keyword only. |
| SC-002 | 20 questions in another language than their notes (English, Indonesian, Japanese, Spanish). | Right note in the top 5 for at least 75%. |
| SC-003 | 40 questions sharing their key words with a note. | The right note's rank is never worse than with keyword search alone. |
| SC-004 | Time searches and message preparation with 10,000 chunks prepared. | 95% of searches within 100 ms after the model is loaded; preparation within 150 ms at the 95th percentile. |
| SC-006 | One hour idle in the tray with search by meaning ready. | NFR-PERF-4. |
| SC-010 | Prepare 10,000 chunks the first time while chatting. | Within 15 minutes; replies no slower. |
| SC-011 | Network capture of a day without accepting, then a download. | Only the provider before; only the provider and the model source during the download. |
| SC-012 | The week-long recall suite (requirements SC-2). | At least 28 of 30. |

The weights and the meaning floor (R4) are tuned during SC-001 to SC-003 and recorded in `docs/architecture.md` section 15.
