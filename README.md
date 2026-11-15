# My Pec

My Pec is a Windows desktop app that gives you one AI companion of your own, named Pec by default. You install it, paste your Claude API key, give the companion a personality and a look, and start talking. Everything it knows about you lives on your computer as plain Markdown files that you can read, edit and delete.

My Pec has no account, no server and no telemetry. Your conversation goes only to the Claude API, with your own key. Optional features that go online, such as Weather, are off until you turn them on, and [PRIVACY.md](PRIVACY.md) lists each one.

## Status

In testing, not released yet. Setup, chat with streaming replies, memory with daily notes and recall, compaction of long conversations, nightly dreaming with a dream diary, search by meaning, the companion's world, an animated character (Pip, or a Live2D model), sticky notes, alarms and weather all work today, with the interface in English or Bahasa Indonesia. Release readiness (the Microsoft Store package, the usage meter, export and erase) comes next.

## What you need

- Windows 11.
- A Claude API key from <https://platform.claude.com/settings/keys>, with credit on the account. You pay Anthropic directly for what you use. My Pec takes no cut.

## Install

Download the latest `My Pec_x.y.z_x64-setup.exe` from the Releases page and run it. It installs for your user only and needs no administrator rights.

The installer is not code-signed yet, so Windows SmartScreen shows "Windows protected your PC". Choose **More info**, then **Run anyway**. If Smart App Control is on, Windows may block the installer entirely; building from source avoids that.

## Build from source

You need:

- The [Tauri prerequisites for Windows](https://v2.tauri.app/start/prerequisites/): Microsoft C++ Build Tools and WebView2.
- Rust (stable), with edition 2024 support.
- Node.js (current LTS) and pnpm 10.

```bash
pnpm install
pnpm tauri dev
```

To build an installer:

```bash
pnpm tauri build
```

The app also runs on macOS for development, without alarm notifications. To build a signed and notarized `.dmg` there, put your Developer ID signing identity and notarization credentials in a file outside the repository (the script's header lists the variables) and run:

```bash
scripts/build-macos.sh <signing env file> [--universal]
```

Before you open a pull request, run every check ([docs/development.md](docs/development.md) explains how we work):

```bash
pwsh scripts/verify.ps1
```

## How it is built

A Tauri 2 shell with the core written in Rust (`crates/pec-core`), the bundled extensions in their own crate (`crates/pec-ext`), and a React and TypeScript interface in the webview (`src/`).

- [docs/principles.md](docs/principles.md): the rules every change follows.
- [docs/requirements.md](docs/requirements.md): what the product does.
- [docs/architecture.md](docs/architecture.md): how it is built.
- [docs/features/](docs/features/): each feature in detail, with its requirements, decisions, data and interfaces.
- [docs/development.md](docs/development.md): how we work, write code and test.

## Live2D

My Pec's own character, Pip, is drawn by the app and needs nothing extra. Live2D characters use the Live2D Cubism SDK for Web. Cubism Core and the Cubism Web Framework are licensed by Live2D Inc. under their own terms, not under MIT, so they are not in this repository. To build with Live2D characters, download the SDK from [Live2D](https://www.live2d.com/en/sdk/download/web/) into `vendor/cubism/`; without it the app builds and Live2D characters fall back to their still portraits. Anyone who publishes an app built on the Cubism SDK must follow [Live2D's SDK license](https://www.live2d.com/en/sdk/license/).

## Privacy

[PRIVACY.md](PRIVACY.md) says what My Pec stores, where, and what it sends.

## Security

[SECURITY.md](SECURITY.md) says how to report a vulnerability.

## License

My Pec's own code is under the [MIT License](LICENSE), copyright Kodelokus. Third-party components keep their own licenses.
