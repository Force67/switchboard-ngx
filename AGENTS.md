# Repository Guidelines

## Project Structure & Module Organization
- `apps/web`: SolidJS + Vite + Bun frontend; UI code in `src/` (themes in `theme.css`/`themes`, API helpers in `api.ts`, stores/contexts under `components` and `contexts`).
- `backend`: Rust workspace with crates (`auth`, `chats`, `config`, `database`, `gateway`, `orchestrator`, `runtime`, `users`) and apps `apps/server` (binary `switchboard-backend`) plus `apps/test-app`; database migrations live in `backend/migrations`; config sample is `backend/crates/config/switchboard.example.toml`; provider descriptors sit in `providers/` (e.g., `openrouter.json`).

## Build, Test, and Development Commands
- Frontend: `cd apps/web && bun install && bun run dev` for local UI; `bun run build` for production bundles (CI mirrors this).
- Backend: `cd backend && cargo build --workspace` to compile everything; `cargo run --bin switchboard-backend serve` starts the HTTP API (other subcommands: `Console`, `SeedData`, `DumpData`, `ClearData`).
- Quality gates: `cd backend && cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --workspace` runs the Rust suites. Optional `nix develop .#web` or `nix develop .#backend` for pinned shells.

## Coding Style & Naming Conventions
- Rust: keep rustfmt defaults (4-space indent); use `snake_case` for functions/fields and `PascalCase` for types; prefer small cohesive crates and leave wiring in binaries (`apps/server`).
- TypeScript/Solid: 2-space indent; `PascalCase` components/hooks, camelCase signals/state; keep API endpoints/constants in `src/api.ts` and reuse theme tokens from `theme.css`/`themes` rather than inlining.

## Testing Guidelines
- Rust tests rely on `tokio::test`, `sqlx`, and temporary SQLite databases in each crate’s `tests/` directory; add coverage when changing chat/user/service logic to avoid regressions.
- Frontend currently lacks automated tests; sanity-check flows via `bun run dev` against a running backend (`VITE_API_BASE` overrides host) and run `bun run build` before PRs.

## Commit & Pull Request Guidelines
- Commits should be short and imperative like existing history (`load env from repo root`, `hack: make gateway compile`); scoped prefixes such as `backend:` or `web:` help readability—avoid lingering `wip` once reviewed.
- PRs need a summary, linked issue, configs/env added (`switchboard.toml`, `VITE_*`, DB migrations), and manual test notes; include UI screenshots/GIFs for visual changes and call out any breaking API shifts.

## Security & Configuration
- Copy `backend/crates/config/switchboard.example.toml` to `switchboard.toml` (or set `SWITCHBOARD_CONFIG`); keep secrets out of git. Set `SWITCHBOARD__AUTH__GITHUB__CLIENT_ID/SECRET`, `SWITCHBOARD__DATABASE__URL` (default SQLite is `sqlite://switchboard.db`), and OpenRouter keys in env or config files.
- Frontend consumes `VITE_API_BASE` and optional `VITE_GITHUB_REDIRECT_PATH`/`VITE_DEFAULT_MODEL`; ensure they track the backend host/port. Do not commit real provider credentials; rotate any leaked keys immediately.
