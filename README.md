# Bullpen

Stock and portfolio research through coding agents. Reports with sources you can check.

<p align="center">
  <img src="assets/screenshots/analysis-thesis.png" alt="A Bullpen research report showing thesis, sources, and final stance" width="720" />
</p>

```bash
brew install --cask puemos/tap/bullpen
```

Or from source: `git clone https://github.com/puemos/bullpen && cd bullpen && cargo run`

---

## How it works

LLM research agents default to one long markdown reply. Bullpen inverts that.

The agent researches freely, but every claim lands as a **source-backed block** — a thesis, a metric, a scenario, a stance — submitted through tools Bullpen controls. The report is assembled from those blocks, not parsed from prose. If a required block is missing or unsourced, finalization fails.

You ask a question or load a portfolio. Pick an agent. Read the report.

---

## What you get

**Thesis and risks** — with source links attached to the claims.

**Scenarios** — base, upside, downside cases when the question calls for it.

**Final stance** — bullish, bearish, mixed, or neutral, with confidence and what would change the view.

**Portfolio reviews** — position-by-position holdings, allocation breakdown, risk factors, rebalancing scenarios. No orders, no position sizing.

**12 data providers** — SEC EDGAR, Alpha Vantage, Polygon, Finnhub, Yahoo Finance, NewsAPI, and more. Add API keys once in Settings; they're stored in your OS keychain, never written to disk.

**Local only** — no account, no telemetry, no sync. Data stays on your machine.

---

## Screenshots

<p align="center">
  <img src="assets/screenshots/analysis-scenario-matrix.png" alt="Scenario matrix showing base, upside, and downside cases" width="720" />
  <br/>
  <em>Scenarios side by side.</em>
</p>

<p align="center">
  <img src="assets/screenshots/analysis-sources.png" alt="Sources panel with clickable citations" width="720" />
  <br/>
  <em>Every source the agent cited, in one place.</em>
</p>

<p align="center">
  <img src="assets/screenshots/analysis-final-stance.png" alt="Final stance with direction and confidence" width="720" />
  <br/>
  <em>Final stance with reasons and what would change it.</em>
</p>

---

## Agents

Bullpen auto-discovers coding agents on your PATH: Claude Code, Codex, Gemini CLI, Qwen, Mistral, Kimi, OpenCode. You only need one.

Bring your own agent with `BULLPEN_CUSTOM_AGENT` and `BULLPEN_CUSTOM_AGENT_ARGS`.

---

## Development

```bash
cd frontend && pnpm install && pnpm dev   # Vite dev server
cd frontend && pnpm build                  # Type-check + build frontend
cargo run                                  # Run the desktop app
cargo test                                 # Run tests
cargo clippy --all-targets --all-features # Lint
```

Architecture: Rust/Tauri backend, Vite/React frontend, SQLite storage. See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

---

## License

MIT or Apache-2.0, at your option.

---

Research tool only. Bullpen does not execute trades, prepare orders, or provide investment advice.
