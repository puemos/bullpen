# Repository Guidelines

## Project Structure & Module Organization

Bullpen is a Rust/Tauri desktop app with a Vite React frontend. Rust application code lives in `src/`: `domain` contains core analysis types, `infra/db` handles SQLite persistence, `infra/acp` manages ACP agent integration and the analysis MCP server, and `commands` exposes Tauri IPC commands. Frontend code lives in `frontend/src`, with hooks in `hooks`, shared state in `store`, and TypeScript types in `types`. Support files are in `capabilities/`, `gen/schemas/`, `icons/`, and `docs/`.

## Build, Test, and Development Commands

- `cd frontend && pnpm install`: install frontend dependencies.
- `cd frontend && pnpm dev`: run the Vite dev server on its default port.
- `cd frontend && pnpm build`: type-check and build the frontend.
- `cargo run`: run the Tauri desktop app.
- `cargo check`: validate Rust compilation.
- `cargo test`: run Rust tests.
- `cargo fmt`: format Rust code with rustfmt.
- `cargo clippy --all-targets --all-features`: run Rust lint checks.

## CI Validation (Zero Tolerance)

Before committing, all checks must pass with **zero warnings**:

**Frontend:**
```bash
cd frontend && pnpm check:ci   # Biome lint + format (must pass cleanly)
cd frontend && pnpm build      # TypeScript type-check (no errors)
```

**Rust:**
```bash
cargo fmt --check                                    # Formatting (no diffs)
cargo clippy --all-targets --all-features -- -D warnings  # Lint (warnings = errors)
cargo test                                           # All tests pass
```

Do not push code that produces warnings. Fix all clippy lints, biome issues, and type errors before committing.

## Coding Style & Naming Conventions

Rust uses edition 2024 and the stable toolchain defined in `rust-toolchain.toml`. Keep modules small and aligned with the existing layers: domain logic should not depend on Tauri, SQLite, or ACP process details. Use `snake_case` for Rust modules, functions, and fields; use `PascalCase` for Rust types and React components. Frontend TypeScript follows the existing style: ES modules, functional React components, single quotes, and two-space indentation in JSX/TSX.

## UI & Theme Guidelines

The frontend uses an editorial design language. Keep new surfaces consistent with this system — do not introduce competing styles.

- **Primitives**: import `Eyebrow`, `SectionHeader`, `HairlineDivider`, `MetaRow`, `Dot` from `@/components/ui/editorial`. Do not redefine these locally.
- **Hairlines, not shadows**: use `border-t border-border` for section breaks and `divide-y divide-border` for lists. Do not add `shadow-sm`/`shadow-xs`/`shadow-md` to buttons, inputs, cards, or containers. Zero radius is authoritative (`--radius: 0px`).
- **Numbered sections**: top-level sections use `SectionHeader` with a two-digit mono `number` (`"01"`, `"02"`) and an eyebrow `label`. Avoid ad-hoc `<h2 className="text-xl font-semibold">`.
- **Eyebrow labels**: uppercase, 10.5px, `tracking-[0.14em]` to `tracking-[0.18em]`, muted foreground. Use for metadata rows, column heads, and any label that precedes a heading.
- **Typography scale**:
  - Display headlines: 34–84px, `font-semibold`, `tracking-[-0.02em]` to `tracking-[-0.035em]`, `leading-[0.95]` to `leading-[1.05]`.
  - Body prose: 14–15.5px, `leading-[1.55]` to `leading-[1.65]`, constrained to `max-w-[62ch]` (or `max-w-[60ch]` for editorial reading).
  - Hero/thesis paragraphs: 20–22px, `leading-[1.45]`.
  - Structural container: `max-w-3xl` for prose, `max-w-5xl` for dense report layouts.
- **Numbers**: always `tabular-nums`. Indices and counts render in `font-mono` at 10.5–11.5px, zero-padded (`String(n).padStart(2, "0")`). Dates render as `Intl.DateTimeFormat` with `{ month: 'short', day: 'numeric', year: 'numeric' }`.
- **Color restraint**: one stance-derived accent per report page (see `getStanceAccent` in `features/report-viewer/badge-styles.tsx`). Reserve `text-primary` / primary-backed motion for *actively running* states only. Do not use colored status dots as a decorative vocabulary; prefer tracked monospace status labels (`RUNNING`, `DONE`, `FAILED`).
- **Actions**: primary action is solid foreground (`border border-foreground bg-foreground text-background`) with a hover inversion; secondary/tertiary actions are text-style (icon + label in muted foreground, hover to foreground) separated by a vertical hairline. Reserve `variant="destructive"` for confirm steps, not triggers — delete triggers hover to `text-destructive` on bare text.
- **Inputs**: bare, with hairline top/bottom borders on editorial surfaces (composer) and standard bordered inputs elsewhere. No `shadow-xs`.
- **Exception — live agent output**: `ProgressTimeline`, `AgentTimeline`, `ToolCallCard`, and `MarkdownMessage` are log/terminal surfaces. They keep a monospace, chat-style identity and are deliberately *not* in the editorial grammar. Do not force editorial headers onto them.
- **Validation**: check UI changes with `cd frontend && pnpm build`, and when layout or stickiness changes, do a `cargo run` smoke test — the sticky section nav and group headers scroll inside the `TabsContent` container, not `window`.

## Testing Guidelines

The current test coverage is Rust-focused. Add unit tests near the code under `#[cfg(test)] mod tests`, following the pattern in `src/infra/db/mod.rs`. Prefer deterministic tests using temporary directories or in-memory state rather than the user app data directory. There is no frontend test runner configured yet, so validate UI-facing changes with `cd frontend && pnpm build` and, when relevant, a manual `cargo run` smoke test.

## Commit & Pull Request Guidelines

Use one-line commit subjects in the form `action(scope): outcome`, such as `refactor(frontend): split app into feature modules` or `fix(acp): clean up stopped runs`. Keep unrelated changes in separate commits. Pull requests should describe the user-visible change, list validation commands run, link any related issue, and include screenshots or short recordings for UI changes.

## Security & Configuration Tips

Do not commit local databases, credentials, or machine-specific ACP agent paths. Prefer documented environment overrides such as `BULLPEN_DB_PATH`, `CODEX_ACP_BIN`, and `BULLPEN_CUSTOM_AGENT` for local configuration.
