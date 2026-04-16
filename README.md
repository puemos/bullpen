# CrazyLines

CrazyLines is a local desktop workbench for ACP-powered stock market research. It follows the same core idea as LaReview: the agent can research broadly, but it must submit typed blocks through app-owned MCP tools instead of returning one opaque markdown answer.

The first implementation is research-only. It does not execute trades, prepare orders, size positions, or provide personalized investment advice.

## What It Does

- Accepts free-text prompts such as `Compare NVDA to AMD` or `Analyze the energy sector`.
- Starts a configured ACP agent.
- Mounts a `crazylines-analysis` MCP server with structured submission tools.
- Persists reports locally in SQLite.
- Renders a simple three-page UI:
  - Ask: prompt, agent selection, live process stream.
  - Reports: submitted blocks, metrics, sources, and final stance.
  - Settings: detected agents and guardrail summary.

## Structured MCP Tools

The agent must use these tools:

- `submit_research_plan`
- `submit_entity_resolution`
- `submit_source`
- `submit_metric_snapshot`
- `submit_analysis_block`
- `submit_final_stance`
- `finalize_analysis`

Finalization fails unless the run has a thesis block, risks block, at least one source, a final stance, and source-backed material blocks.

## Development

```bash
cd frontend
pnpm install
pnpm build
cd ..
cargo check
cargo test
```

Run the desktop app:

```bash
cargo run
```

## Agent Configuration

CrazyLines discovers the same kinds of ACP agents as LaReview:

- Codex via `npx -y @zed-industries/codex-acp@latest`
- Claude via `npx -y @zed-industries/claude-code-acp`
- Gemini/Qwen/Mistral/Kimi via their `--experimental-acp` commands
- OpenCode via `opencode acp`

Environment overrides:

- `CODEX_ACP_BIN`
- `CODEX_ACP_PACKAGE`
- `CLAUDE_ACP_BIN`
- `GEMINI_ACP_BIN`
- `QWEN_ACP_BIN`
- `MISTRAL_ACP_BIN`
- `KIMI_ACP_BIN`
- `OPENCODE_ACP_BIN`
- `CRAZYLINES_CUSTOM_AGENT`
- `CRAZYLINES_CUSTOM_AGENT_ARGS`

The local database defaults to the OS app data directory. Override it with `CRAZYLINES_DB_PATH`.

