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

The development UI now includes a minimal playground for sending prompts to the backend orchestrator. It targets `http://localhost:7070` by default; override with `VITE_API_BASE` when running `bun run dev` if the backend lives elsewhere (e.g. `VITE_API_BASE=https://staging.example bun run dev`). Set `VITE_GITHUB_REDIRECT_PATH` when your GitHub OAuth callback differs from the default `/auth/callback`.

### Backend

```bash
cd backend
cargo run --bin switchboard-backend
```

The backend currently initialises configuration and the orchestration layer, then waits for a shutdown signal. Real request handling and provider integrations can be layered on top of the provided crates.

The HTTP surface includes a basic chat endpoint for exercising the OpenRouter integration:

```http
POST /api/chat
Authorization: Bearer <session-token>
Content-Type: application/json

{
  "prompt": "Hello world",
  "model": "optional-model-identifier"
}
```

Responses contain the generated message, optional reasoning traces, and token usage metadata. Additional endpoints power the UI:

| Endpoint | Method | Purpose |
| --- | --- | --- |
| `/api/auth/github/login` | `GET` | Returns the GitHub authorize URL (includes CSRF state). |
| `/api/auth/github/callback` | `POST` | Exchanges the OAuth code for a Switchboard session token. |
| `/api/models` | `GET` | Lists available OpenRouter models for the dropdown selector. |

All authenticated endpoints expect the session token issued during GitHub login in the `Authorization: Bearer` header.

### GitHub SSO

Create a GitHub OAuth application and configure its callback URL (e.g. `http://localhost:3000/auth/callback`). Populate `SWITCHBOARD__AUTH__GITHUB__CLIENT_ID` and `SWITCHBOARD__AUTH__GITHUB__CLIENT_SECRET` in `.env` at the repo root; the backend reads everything from environment variables. The frontend automatically redirects to the GitHub flow and completes the exchange via the callback endpoint.

By default the backend connects to an in-project SQLite database at `sqlite://switchboard.db`. Supply `SWITCHBOARD__DATABASE__URL` to target PostgreSQL instead, e.g. `postgres://user:pass@localhost/switchboard`.

### OpenRouter Provider

Switchboard NGX ships with an initial OpenRouter integration powered by the [`denkwerk`](https://github.com/Force67/denkwerk) library. Configure it via environment variables (e.g. `SWITCHBOARD__ORCHESTRATOR__OPENROUTER__API_KEY` for your key, `SWITCHBOARD__ORCHESTRATOR__OPENROUTER__BASE_URL`, and `SWITCHBOARD__ORCHESTRATOR__OPENROUTER__REQUEST_TIMEOUT_SECONDS`). The provider search path can be set with `SWITCHBOARD__ORCHESTRATOR__PROVIDER_SEARCH_PATH__0=providers`.

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
