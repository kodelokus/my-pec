# My Pec Principles

The rules every change to My Pec follows. They take precedence over every other document in this repository. [requirements.md](requirements.md) says what the product does, [architecture.md](architecture.md) how it is built, the pages in [features/](features/) what each feature does in detail, and [development.md](development.md) how we work.

## Core Principles

### I. Files Are the Mind

- The workspace files are the single source of truth for everything the companion is, knows and
  has made: `SOUL.md`, `IDENTITY.md`, `USER.md`, `MEMORY.md`, `DREAMS.md`, `memory/`, `sessions/`,
  `world/`, and what the bundled extensions keep (`notes/`, `alarms.md`, `weather.md`).
- The database MUST hold only data that can be rebuilt from the workspace, plus bookkeeping
  (recall signals, preimages, the usage ledger, run history).
- Deleting the search index MUST NOT lose any memory. The index MUST be rebuildable from the files.
- Edits the user makes to workspace files outside the app MUST be picked up and MUST NOT be
  silently overwritten.
- `world/` holds what the companion has drawn. The app MUST only append to it. The only exception
  is removal the user explicitly requests. What was drawn MUST be stored there and MUST NOT be
  worked out again from the companion's id, so an update to the app never changes it.
- Workspace files MUST stay human-readable. The names and roles of the files OpenClaw defines
  MUST stay compatible with OpenClaw's. `world/` is My Pec's own and has no OpenClaw counterpart.

**Rationale**: The user owns the companion's mind and can read, correct and delete it with any
text editor. Plain files also survive app bugs and migrations. The world the companion draws is
part of what makes it that companion, so it follows the same rule.

### II. Local-First, and the Key Stays in the Core

- All user data MUST stay on the user's disk. Outbound connections are limited to the configured
  LLM API, the update server, the one-time embedding model download, and the destinations that a
  feature's page in `docs/features/` names, only while the user has that feature on.
- A feature that adds a network destination MUST name it in its page in `docs/features/` and MUST
  be off until the user turns it on.
- The app MUST NOT send telemetry or analytics unless the user opts in.
- The user enters the API key in the webview. The webview MUST pass it to the core once and
  MUST NOT keep it afterwards. From then on the key MUST exist in memory only in the core
  process and at rest only in the operating system's credential store.
- The core MUST NOT return the key to the webview, apart from its last four characters. The
  key MUST NOT appear in logs, crash reports, exports or any app file.
- The webview MUST reach the core only through commands allow-listed in its window's capability
  file. It MUST NOT have file system, shell or network access of its own.

**Rationale**: A companion hears private things. The only party that should see them is the LLM
provider the user chose.

### III. Append-Only Conversation Epochs

- Between two compactions the system prompt and tool list MUST NOT change, and each request MUST
  equal the previous request plus appended messages.
- History MUST NOT be edited or trimmed client-side. Compaction is the only operation that
  shortens what the model sees, and it MUST run synchronously at a turn boundary.
- Assistant content MUST be stored and replayed exactly as received.
- Changes that happen mid-epoch, such as a soul edit, MUST be appended as new messages.
- The transcript on disk MUST be append-only and MUST keep the complete history whatever
  compaction does. The only exception is redaction the user explicitly requests.

**Rationale**: Prompt caching matches on the request prefix, and current Claude models reject
replayed thinking blocks when earlier history has changed. One rule keeps turns cheap and valid.

### IV. Provenance and Undo for Durable Memory

- Every long-term memory entry MUST carry a reference to its source.
- Every memory candidate MUST carry a provenance tag. Content tagged `untrusted` or `system` MUST
  NOT be promoted to long-term memory automatically.
- Every automatic rewrite of `SOUL.md`, `USER.md` or `MEMORY.md` MUST store the previous content
  first and MUST be reversible from the user interface.
- An automatic rewrite MUST pass validation (entry-loss limit, source references kept, size
  budget, expected structure) or fall back to appending.
- The companion MUST NOT change `SOUL.md` without the user's explicit approval.
- A request to forget MUST remove the content from the files, the index and future dream input.

**Rationale**: Memory that rewrites itself is only trustworthy if every entry can be traced and
every change can be taken back.

### V. Chat Comes First

- Background work (indexing, compaction, dreaming) MUST yield to an incoming user message at its
  next step boundary.
- Background jobs MUST be checkpointed and safe to rerun. A job interrupted before it commits
  MUST leave memory exactly as it was.
- A failure in a background job, the avatar, the index or the embedding model MUST NOT block or
  break chat.
- The user's message MUST be persisted before the request is sent and MUST NOT be lost.
- Heavy work MUST NOT run on the thread that serves a streaming reply.

**Rationale**: The conversation is the product. Everything else exists to support it.

### VI. Shell-Independent Core

- All agent, context, memory, compaction, dreaming, scheduling and LLM logic MUST live in the
  `pec-core` crate. The bundled extensions' rules MAY live in their own crate, which MUST depend
  on `pec-core` alone. Neither crate may depend on Tauri or on any user interface.
- The Tauri shell MUST be limited to windows, the tray, notifications, launch at login, command
  handlers, capability configuration, secret storage, updates, and the clock and timer that drive
  the core's extension tick.
- The core MUST run and be tested headless against a fake LLM provider.
- The core MUST depend on the `LlmProvider` trait. Provider-specific code, including every Claude
  API detail and beta feature, MUST be confined to one module.
- The user interface MUST drive the avatar only through the `AvatarRenderer` interface.

**Rationale**: The memory loop is the hard part and must be testable without a window. Keeping
the provider and the renderer behind interfaces means either can change without touching the rest.

### VII. Working Defaults, Optional Settings

- A fresh install MUST produce a working companion with the API key as the only required input.
- Every setting MUST have a default. No feature may require configuration before first use.
- The app MUST NOT require an account, a sign-in or a server of the project's own.
- Onboarding MUST be resumable and MUST NOT leave the app half-configured.
- A feature that needs something not yet available MUST degrade to a working fallback, such as
  keyword search before the embedding model has downloaded.

**Rationale**: The promise is install, paste a key, start talking.

## Technology Constraints

- **Shell and languages**: Tauri 2. The core is Rust. The user interface is React and TypeScript,
  built with Vite, in the system webview.
- **LLM provider**: Claude only in v1, through a project-owned HTTPS client in the core's `llm`
  module. The API version MUST be pinned, and reply content blocks MUST be kept as the raw JSON
  that arrived.
- **Storage**: Markdown and JSONL in the workspace. SQLite, with FTS5 and a vector extension, for
  derived data only.
- **Embeddings**: a model run locally. No feature may require a second API key.
- **Avatar**: Live2D through the official Cubism SDK for Web. A static-image renderer MUST always
  be available as the fallback. Any feature that depends on Live2D's licensing review, such as
  user-imported models, MUST stay behind a build flag until it is cleared.
- **Shared types**: Rust definitions are the single source. TypeScript types MUST be generated
  from them, not written by hand. The calls into the core MAY be wrapped by hand, in one module,
  using those types.
- **Platforms**: Windows 11 is the first supported platform. Platform-specific code MUST be
  isolated so that other platforms can follow.

Changing any item in this section requires an amendment to these principles.

## Development Workflow and Quality Gates

**Workflow**

- Every feature is described in its page in `docs/features/` (user stories, requirements, success
  criteria, decisions, data and interfaces) before it is built, and the page is kept true to what
  was built.
- `docs/requirements.md` and `docs/architecture.md` are the product-level references. A feature
  that contradicts either MUST update it in the same change.
- Each feature page MUST state how the feature satisfies each principle, or record a justified
  exception.

**Test gates**

- Core logic MUST be covered by tests that run headless against a fake LLM. No test in CI may
  call the real API.
- A snapshot test MUST prove that the request prefix is unchanged across turns within an epoch.
- A change to compaction MUST include tests for the cut point, the kept turns and summary
  validation.
- A change to dreaming MUST include tests that validation rejects a lossy rewrite and that undo
  restores the previous file exactly.
- A change to the Claude client MUST pass contract tests on recorded responses.

**Security and data gates**

- No secret may appear in logs, error messages or test fixtures.
- A new webview command MUST be added to a capability file explicitly.
- Command payloads and tool inputs MUST be validated. Model output MUST be rendered as sanitized
  Markdown.
- A change to a workspace file format MUST include a migration and MUST keep older files readable.

**Simplicity**

- A new dependency or abstraction MUST be justified by a need stated in the feature's page.

## Governance

- These principles take precedence over other project documents and practices where they
  conflict.
- **Amendments**: an amendment is an edit to this file with its rationale, approved by the
  project owner. Documents that depend on the changed rule, including `docs/requirements.md` and
  `docs/architecture.md`, MUST be updated in the same change.
- **Versioning**: MAJOR for removing or redefining a principle, MINOR for adding a principle or
  section or materially expanding one, PATCH for clarifications and wording.
- **Compliance**: reviews of feature pages and code MUST check compliance with these principles.
  A deviation MUST be recorded in the feature's page with its reason and the simpler alternative that was
  rejected.
- **Runtime guidance**: `docs/architecture.md` describes how the principles are implemented.

**Version**: 1.3.0 | **Ratified**: 2026-11-13 | **Last Amended**: 2026-11-15
