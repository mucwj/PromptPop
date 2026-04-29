# Contributing to PromptPop

Thanks for taking a look at PromptPop. This project is a local-first desktop
prompt launcher built with Svelte, TypeScript, Tauri 2, Rust, and SQLite.

## Development Setup

Prerequisites:

- Node.js 20 or newer
- npm
- Rust stable
- macOS for the full desktop workflow

Install dependencies:

```sh
npm install
```

Run the web UI:

```sh
npm run dev
```

Run the Tauri app:

```sh
npm run tauri:dev
```

Run the standard verification suite:

```sh
npm run verify
```

Build a macOS `.app` bundle:

```sh
npm run tauri:build:app
```

## Pull Requests

- Keep changes focused and reversible.
- Include tests when behavior changes.
- Update README or docs when user-facing behavior changes.
- Run `npm run verify` before opening a PR.

PromptPop stores user prompts locally. Please avoid adding telemetry, cloud
sync, or network behavior without a clear opt-in design and documentation.
