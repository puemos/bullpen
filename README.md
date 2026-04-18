# Bullpen

Stock and portfolio research through coding agents. Reports with sources.

![screenshot](assets/screenshots/analysis-thesis.png)

## Install

```
brew install --cask puemos/tap/bullpen
```

Or build from source:
```
git clone https://github.com/puemos/bullpen
cd bullpen && cargo run
```

## How it works

The agent researches freely but submits every claim as a source-backed block through tools Bullpen controls. The report is assembled from those blocks, not parsed from prose. Missing or unsourced blocks fail finalization.

## Features

1. Thesis and risks with source links
2. Scenarios (base, upside, downside)
3. Final stance with confidence
4. Portfolio reviews (holdings, allocation, risk, rebalancing)
5. CSV import for portfolios
6. 12 data providers (SEC EDGAR, Polygon, Finnhub, Alpha Vantage, Yahoo Finance, NewsAPI, etc.)
7. Export to HTML or Markdown
8. Local storage, no account, no telemetry

## Data providers

Tavily, Brave Search, SEC EDGAR, Alpha Vantage, Financial Modeling Prep, Finnhub, Polygon, Yahoo Finance, NewsAPI, Finviz, StockTwits, Hacker News.

API keys stored in OS keychain. Providers without keys are excluded.

## Agents

Auto-discovers: Claude Code, Codex, Gemini CLI, Qwen, Mistral, Kimi, OpenCode.

Custom agent: `BULLPEN_CUSTOM_AGENT`, `BULLPEN_CUSTOM_AGENT_ARGS`.

## Development

```
cd frontend && pnpm install && pnpm dev   # dev server
cd frontend && pnpm build                  # build frontend
cargo run                                  # run app
cargo test                                 # tests
cargo clippy --all-targets --all-features # lint
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## License

MIT or Apache-2.0.

---

Research tool. Does not execute trades or provide investment advice.
