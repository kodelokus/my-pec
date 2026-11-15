# Development

How we work on My Pec. The [principles](principles.md) are the rules; this page is how we keep to them day to day. It needs no particular editor or assistant.

## Where things are written down

| Document | What it holds |
|---|---|
| [principles.md](principles.md) | The seven principles, the technology constraints and the quality gates. They win over everything else. |
| [requirements.md](requirements.md) | What the product does, with stable ids (CHAT-5, MEM-9, EXT-2, NFR-PERF-4, decisions D1 to D19) and the delivery slices. |
| [architecture.md](architecture.md) | How it is built: the crates, files, data, request layout, jobs and windows. |
| [features/](features/) | One page per feature: user stories, requirements (FR-), success criteria (SC-), decisions (R), data formats and interfaces, and the manual checks. |

Code comments cite these documents. A citation is the feature's page name and an id, for example `extensions R4`, `weather FR-015`, `compaction SC-003`, `memory data 2` or `chat interfaces`; product-level ids such as `CHAT-5` and the principles (`Principle III`) are cited as they are.

## Working on a feature

1. **Describe it first.** Write or extend its page in `docs/features/`: user stories with acceptance scenarios, testable requirements, measurable success criteria, the decisions and the alternatives rejected, data formats and interfaces, and how the feature satisfies each principle. A network destination, a new dependency or a deviation from a principle is named and justified there.
2. **Keep the product documents true.** If the feature changes what the product does or how it is built, update `requirements.md` and `architecture.md` in the same change. Changing a principle is an amendment to `principles.md`, approved by the project owner.
3. **Build it in small, tested steps**, following the rules below.
4. **Run every check** (`pwsh scripts/verify.ps1 -E2E`) before asking for a review, and record anything checked by hand in the page's Checks section.

## Lean code

Write the least code that satisfies the feature page, the architecture and the principles. The principles are not negotiable; ceremony around them is.

**Before writing**

- Find what exists first: the owning module in `pec-core` (each store is the only writer of its data) and the shared pieces (error types, the `LlmProvider` trait, the memory writer and atomic write, provenance tags, preimages, the usage ledger, the context assembler, the scheduler's locks and checkpoints, the generated TypeScript bindings). Extend them; don't build a parallel helper or let one module write another's files.
- Build only what the feature needs. No options, traits, generics or settings no current caller uses. A trait or interface needs a second implementation or a test boundary that can't use the real one: `LlmProvider`, `AvatarRenderer`, `Extension` and `Fetch` qualify; most things don't.
- One file per concept, not per tiny type. A file over about 300 lines needs a reason.
- A new dependency needs a need stated in the feature page.

**While writing**

- Prefer data over code: personality templates, avatar manifests, defaults, tool definitions and user-facing messages each live in one place as data.
- Tauri command handlers and scheduler steps stay thin: deserialize, call the core, map the result. Rules live in `pec-core` or `pec-ext` (Principle VI).
- Every Claude API detail stays in the core's `llm` module.
- Types that cross to the webview are defined once in Rust; the TypeScript side is generated, never written by hand.
- No concurrency primitive the call path doesn't need.
- Comments explain a rule that isn't obvious, or cite a document as above, in one line. They never restate the code or narrate history.
- React: one component per screen concept; shared modules for command calls and events; no wrappers, providers or stores a screen doesn't need.
- Every user-facing string is in `src/strings.en.ts` and `src/strings.id.ts`, in both languages.

**Before committing**

- Delete dead code, unused parameters, needless `pub` and abstractions the work ended up not needing.
- Check the tests against the rules below.

## Tests

Tests prove behaviour the principles, the requirements and the feature pages care about, once, at the cheapest boundary that can prove it. Coverage counts are not a goal. Every principle and every quality gate has an automated test; every rule has a test written with its code. Integration tests use the real dependencies (real files in a temporary workspace, real SQLite with its extensions); only the model is faked, and no test reaches the real API, GitHub or any other service.

**Before adding or changing a test, answer:**

1. What does it protect? A requirement, a principle, a contract or a bug. If none, don't write it.
2. What change to the production code would make it fail? If none is credible, it's noise.
3. Why doesn't an existing test already catch it? One owner test per contract, at the strongest cheap boundary; a second layer only for a risk that layer adds (an atomic write, a SQL constraint, a transaction, command wiring and capabilities, serialization to the webview, resuming an interrupted job).
4. Does it need a test-only hook? Prefer the real boundary. The fake `LlmProvider`, the fake `Fetch` and the injected clock are the standing ones.

**Where to prove what**

| Prove with | For |
|---|---|
| Unit tests in `pec-core` or `pec-ext` | rules, state machines, parsing, budgets, scoring; table-driven cases |
| A request snapshot | what the model receives: the prefix unchanged across turns, the layout after compaction |
| Core integration (temporary workspace, real SQLite, fake model, injected clock) | atomic writes, single-writer ownership, transcript appends, index rebuilds, undo, job checkpoints and resume |
| Contract tests on recorded responses | the Claude client: stream parsing, tool calls, refusals, errors and retries |
| Vitest | the webview's behaviour through rendered output and user events, never re-proving core rules |
| End-to-end (WebdriverIO through `tauri-driver`, against the local replay server) | one spec per user-visible journey, keyboard and screen-reader use |

**Avoid**: asserting a value the code under test produced, asserting mock calls instead of results, mocks that supply the behaviour being asserted, the same contract tested at every layer, one copy-pasted test per command instead of a table, fixtures the app never writes, test names that don't match what they test, snapshots of component markup.

**Keep fast**: a fresh temporary workspace per test from the shared fixture; no shared global state, so `cargo test` keeps its parallelism; no sleeping and no real timers; a test slower than 5 seconds needs a comment saying why.

## Checks

`pwsh scripts/verify.ps1` runs every gate: formatting, Clippy, the Principle VI dependency check, `cargo test`, the generated bindings, Biome, TypeScript and Vitest. `-E2E` adds the end-to-end specs against a debug build. While working, run only what you touched (`cargo test -p pec-core <name>`, `npx vitest run <file>`); run the whole script before a review.

## Live2D

Cubism Core and the Cubism Web Framework are licensed by Live2D, never committed, and fetched into `vendor/cubism/` for builds that include Live2D characters. Without them the app builds and runs with the drawn and still characters.
