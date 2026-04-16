# Repository Guidelines

## Project Structure & Module Organization

CrazyLines is a Rust/Tauri desktop app with a Vite React frontend. Rust application code lives in `src/`: `domain` contains core analysis types, `infra/db` handles SQLite persistence, `infra/acp` manages ACP agent integration and the analysis MCP server, and `commands` exposes Tauri IPC commands. Frontend code lives in `frontend/src`, with hooks in `hooks`, shared state in `store`, and TypeScript types in `types`. Support files are in `capabilities/`, `gen/schemas/`, `icons/`, and `docs/`.

## Build, Test, and Development Commands

- `cd frontend && pnpm install`: install frontend dependencies.
- `cd frontend && pnpm dev`: run the Vite dev server on its default port.
- `cd frontend && pnpm build`: type-check and build the frontend.
- `cargo run`: run the Tauri desktop app.
- `cargo check`: validate Rust compilation.
- `cargo test`: run Rust tests.
- `cargo fmt`: format Rust code with rustfmt.
- `cargo clippy --all-targets --all-features`: run Rust lint checks.

## Coding Style & Naming Conventions

Rust uses edition 2024 and the stable toolchain defined in `rust-toolchain.toml`. Keep modules small and aligned with the existing layers: domain logic should not depend on Tauri, SQLite, or ACP process details. Use `snake_case` for Rust modules, functions, and fields; use `PascalCase` for Rust types and React components. Frontend TypeScript follows the existing style: ES modules, functional React components, single quotes, and two-space indentation in JSX/TSX.

## Testing Guidelines

The current test coverage is Rust-focused. Add unit tests near the code under `#[cfg(test)] mod tests`, following the pattern in `src/infra/db/mod.rs`. Prefer deterministic tests using temporary directories or in-memory state rather than the user app data directory. There is no frontend test runner configured yet, so validate UI-facing changes with `cd frontend && pnpm build` and, when relevant, a manual `cargo run` smoke test.

## Commit & Pull Request Guidelines

This repository has no commit history yet, so there is no established message convention. Use short, imperative subjects such as `Add analysis export command` or `Fix ACP run cleanup`, and keep unrelated changes in separate commits. Pull requests should describe the user-visible change, list validation commands run, link any related issue, and include screenshots or short recordings for UI changes.

## Security & Configuration Tips

Do not commit local databases, credentials, or machine-specific ACP agent paths. Prefer documented environment overrides such as `CRAZYLINES_DB_PATH`, `CODEX_ACP_BIN`, and `CRAZYLINES_CUSTOM_AGENT` for local configuration.
