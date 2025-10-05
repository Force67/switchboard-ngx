# Switchboard NGX

Switchboard NGX is an experiment in orchestrating large language model providers with a clean Rust backend and a Bun-powered SolidJS front-end. The workspace is split into two main applications:

- `apps/web` – SolidJS + TypeScript UI served via Bun and Vite. It will become the console for configuring orchestration rules, registering providers, and monitoring health.
- `backend` – A Rust workspace that exposes reusable crates for configuration and orchestration with a slim runtime binary at `backend/server`.

## Getting Started

### Requirements

- [Bun](https://bun.sh/) (for the web application)
- Rust toolchain (`rustc`, `cargo`, `rustfmt`, `clippy`)
- Optional: [Nix](https://nixos.org/download.html) for reproducible development shells (`nix develop .#web` or `nix develop .#backend`).

### Frontend

```bash
cd apps/web
bun install
bun run dev
```

### Backend

```bash
cd backend
cargo run --bin switchboard-backend
```

The backend currently initialises configuration and the orchestration layer, then waits for a shutdown signal. Real request handling and provider integrations can be layered on top of the provided crates.

By default the backend connects to an in-project SQLite database at `sqlite://switchboard.db`. Supply `SWITCHBOARD__DATABASE__URL` to target PostgreSQL instead, e.g. `postgres://user:pass@localhost/switchboard`.

### Authentication

The backend ships with an authentication service powered by `sqlx`:

- Email + password identities are stored with Argon2 hashes.
- GitHub OAuth can be enabled by setting `SWITCHBOARD__AUTH__GITHUB__CLIENT_ID` and `SWITCHBOARD__AUTH__GITHUB__CLIENT_SECRET`, then exchanging OAuth codes via the `switchboard-auth` crate.
- Session tokens are persisted in the database with a default TTL of 24 hours (configurable via `SWITCHBOARD__AUTH__SESSION_TTL_SECONDS`).

## Nix Flakes

- Root `flake.nix` offers `default`, `web`, and `backend` development shells.
- Each application also includes its own `flake.nix` for isolated environments (`apps/web/flake.nix` and `backend/flake.nix`).

## Next Steps

1. Flesh out orchestration APIs and HTTP surface area in the backend (e.g. with `axum` or `warp`).
2. Design provider descriptors and persistence for orchestration state.
3. Connect the SolidJS UI to backend endpoints for real-time configuration.
