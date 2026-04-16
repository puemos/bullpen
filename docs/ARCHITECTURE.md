# Architecture

CrazyLines mirrors LaReview's layered shape while replacing diff review concepts with stock analysis concepts.

## Layers

- `src/domain`: pure types for analyses, runs, entities, sources, metrics, blocks, and final stances.
- `src/infra/db`: SQLite persistence and report assembly.
- `src/infra/acp/analysis_generator`: ACP client lifecycle, agent process management, progress streaming.
- `src/infra/acp/analysis_mcp_server`: app-owned MCP server that persists structured submissions.
- `src/commands`: Tauri IPC commands used by the frontend.
- `frontend/src`: React UI for Ask, Reports, and Settings.

## Run Lifecycle

1. The user submits a free-text request from Ask.
2. `generate_analysis` creates `Analysis` and `AnalysisRun` rows.
3. The backend spawns the selected ACP agent.
4. The ACP session mounts a stdio MCP server named `crazylines-analysis`.
5. The system prompt instructs the agent to research with all available tools but submit output only through CrazyLines tools.
6. MCP tool calls persist sources, metrics, blocks, and stance as they arrive.
7. `finalize_analysis` validates the report and marks the run complete.
8. The Reports page renders the persisted typed report.

## Safety Defaults

- Research-only product posture.
- No portfolio personalization or trade execution.
- Source and metric metadata are first-class data.
- Finalization is blocked if required evidence is missing.

