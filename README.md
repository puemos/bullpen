# Bullpen

> Local desktop workbench for ACP-powered equity research. Agents research broadly — but submit typed, source-backed blocks through app-owned MCP tools, never a single opaque markdown answer.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-black)](#license)
[![Rust](https://img.shields.io/badge/rust-edition%202024-black)](rust-toolchain.toml)
[![Tauri](https://img.shields.io/badge/tauri-desktop-black)](https://tauri.app)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-black)]()

<p align="center">
  <video src="assets/videos/new-analysis.mp4" width="880" controls></video>
  <br/>
  <em>Prompt → ACP agent → typed MCP submissions → final stance, in one run.</em>
</p>

> ⚠️ **Research only.** Bullpen does not execute trades, prepare orders, size positions, or provide personalized investment advice.

---

## Why Bullpen

LLM research agents default to one long markdown reply. Bullpen inverts that: the agent runs freely, but every claim lands as a **typed block** — a thesis, a metric, a source, a scenario, a stance — submitted through MCP tools Bullpen controls. The report is assembled from those blocks, not parsed from prose. Runs that lack a thesis, risks, sources, or a final stance **fail to finalize**.

## How It Works

1. **Ask** — free-text prompt, pick an ACP agent, watch the process stream live.
2. **Agent submits typed blocks** — the agent calls `submit_*` tools on the `bullpen-analysis` MCP server. Each call is validated and persisted to SQLite as it arrives.
3. **Read the report** — thesis, stance, scenarios, projections, and every source behind the call.

---

## Quick Start

```bash
# prerequisites: Rust stable (see rust-toolchain.toml), pnpm, Node 20+
git clone https://github.com/puemos/bullpen && cd bullpen
cd frontend && pnpm install && pnpm build && cd ..
cargo run
```

Bullpen auto-discovers any ACP agent on your PATH (Codex, Claude, Gemini, Qwen, Mistral, Kimi, OpenCode).

## The MCP Contract

The agent must submit through these tools. Finalization fails unless the checked requirements are met.

| Tool                      | Purpose                             | Required to finalize          |
| ------------------------- | ----------------------------------- | :---------------------------- |
| `submit_research_plan`    | Up-front plan of attack             |                               |
| `submit_entity_resolution`| Disambiguate tickers / entities     |                               |
| `submit_source`           | Cite a URL + retrieval context      | ≥ 1                           |
| `submit_metric_snapshot`  | Structured KPI / metric point       |                               |
| `submit_analysis_block`   | Thesis, risks, scenarios, etc.      | thesis + risks, source-backed |
| `submit_final_stance`     | Rating + confidence                 | ✓                             |
| `finalize_analysis`       | Seal the run                        | —                             |

## Agent Configuration

Bullpen discovers ACP agents via standard commands. You only need one.

<details>
<summary><strong>Built-in agents</strong> (no config needed if installed)</summary>

- Codex — `npx -y @zed-industries/codex-acp@latest`
- Claude — `npx -y @zed-industries/claude-code-acp`
- Gemini / Qwen / Mistral / Kimi — via each CLI's `--experimental-acp`
- OpenCode — `opencode acp`
</details>

<details>
<summary><strong>Override binaries</strong></summary>

`CODEX_ACP_BIN`, `CODEX_ACP_PACKAGE`, `CLAUDE_ACP_BIN`, `GEMINI_ACP_BIN`, `QWEN_ACP_BIN`, `MISTRAL_ACP_BIN`, `KIMI_ACP_BIN`, `OPENCODE_ACP_BIN`
</details>

<details>
<summary><strong>Custom agent</strong></summary>

`BULLPEN_CUSTOM_AGENT`, `BULLPEN_CUSTOM_AGENT_ARGS`
</details>

<details>
<summary><strong>Storage</strong></summary>

`BULLPEN_DB_PATH` — defaults to the OS app data directory.
</details>

## Screens

<p align="center">
  <img width="260" src="assets/screenshots/analysis-thesis.png" alt="Thesis" />
  <img width="260" src="assets/screenshots/analysis-scenario-matrix.png" alt="Scenario matrix" />
  <img width="260" src="assets/screenshots/analysis-projection.png" alt="Projections" />
  <br/>
  <img width="260" src="assets/screenshots/analysis-sources.png" alt="Sources" />
  <img width="260" src="assets/screenshots/analysis-data-points.png" alt="Data points" />
  <img width="260" src="assets/screenshots/analysis-final-stance.png" alt="Final stance" />
</p>

## Development

| Command                                      | Purpose                          |
| -------------------------------------------- | -------------------------------- |
| `cd frontend && pnpm dev`                    | Vite dev server                  |
| `cd frontend && pnpm build`                  | Type-check + build frontend      |
| `cargo run`                                  | Run the Tauri desktop app        |
| `cargo check`                                | Validate Rust compilation        |
| `cargo test`                                 | Run Rust tests                   |
| `cargo fmt`                                  | Format Rust with rustfmt         |
| `cargo clippy --all-targets --all-features`  | Lint                             |

## Architecture

Rust/Tauri backend + Vite/React frontend. Domain types in `src/domain`, SQLite in `src/infra/db`, ACP + MCP server in `src/infra/acp`, Tauri IPC in `src/commands`. See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## License

Licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
