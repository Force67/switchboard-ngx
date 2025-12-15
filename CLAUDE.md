# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Switchboard NGX is an experiment in orchestrating large language model providers with a clean Rust backend and a Bun-powered SolidJS front-end. The project consists of two main applications:

- **Frontend** (`apps/web`): SolidJS + TypeScript UI served via Bun and Vite
- **Backend** (`backend`): Rust workspace with reusable crates for configuration, orchestration, authentication, and API handling

## Development Commands

### Frontend (apps/web)
```bash
cd apps/web
bun install          # Install dependencies
bun run dev          # Start development server (targets http://localhost:3000)
bun run build        # Build for production
bun run preview      # Preview production build
```

The frontend uses environment variables for configuration:
- `VITE_API_BASE`: Backend API base URL (default: `http://localhost:7070`)
- `VITE_DEFAULT_MODEL`: Default model to use
- `VITE_GITHUB_REDIRECT_PATH`: GitHub OAuth callback path (default: `/auth/callback`)
- `VITE_ENABLE_DEV_LOGIN`: Enable development session auto-bootstrap (default: `true` in dev)

### Backend (backend)
```bash
cd backend
# Development commands
cargo run --bin switchboard-backend    # Start HTTP server (default: port 7070)
cargo run --bin switchboard-backend serve  # Explicitly start server
cargo run --bin switchboard-backend console  # Interactive console mode

# Database management
cargo run --bin switchboard-backend dump-data    # Dump folders and chats from database
cargo run --bin switchboard-backend clear-data   # Clear all folders and chats
cargo run --bin switchboard-backend seed-data    # Seed database with test data

# Standard Rust commands
cargo build
cargo test
cargo clippy
cargo fmt
```

### Nix Development (Optional)
```bash
# Use reproducible development shells
nix develop .#web        # Frontend development shell
nix develop .#backend    # Backend development shell
nix develop .            # Root development shell
```

## Architecture

### Backend Architecture

The backend follows a modular crate-based architecture:

- **`server`**: Main binary entry point with CLI interface (`server/src/main.rs`)
- **`backend-api`**: HTTP API layer using Axum with comprehensive REST endpoints
- **`orchestrator`**: LLM provider orchestration and model management (`crates/orchestrator/src/lib.rs`)
- **`auth`**: Authentication service supporting GitHub OAuth and password-based auth (`crates/auth/src/lib.rs`)
- **`config`**: Configuration management with TOML support and environment overrides
- **`runtime`**: Shared runtime services and telemetry initialization

#### Key Backend Features
- **Multi-provider LLM orchestration** with OpenRouter integration via the `denkwerk` library
- **Real-time WebSocket communication** for live chat updates
- **Comprehensive API** with REST endpoints for chats, folders, messages, attachments, notifications, and permissions
- **Authentication system** with GitHub OAuth and session management
- **Database abstraction** using SQLx supporting SQLite (default) and PostgreSQL

### Frontend Architecture

The frontend uses SolidJS with a component-based architecture:

- **`src/App.tsx`**: Main application with authentication, WebSocket integration, and state management
- **`src/components/`**: UI components including Sidebar, MainArea, and various chat components
- **`src/contexts/`**: React-like contexts for theme and application state
- **`src/api/`**: API service layer for backend communication
- **Real-time updates** via WebSocket integration with automatic reconnection
- **Model selection** with @mention syntax for targeting specific models

## Configuration

### Backend Configuration

Copy `backend/crates/config/switchboard.example.toml` to `backend/crates/config/switchboard.toml` or set environment variables:

```toml
[orchestrator]
default_model = "openrouter/meta-llama/llama-3.1-70b-instruct"

[orchestrator.openrouter]
api_key = "sk-your-openrouter-api-key"  # Or set OPENROUTER_API_KEY env var
base_url = "https://openrouter.ai/api/v1"
request_timeout_seconds = 30

[database]
url = "sqlite://switchboard.db"  # Or postgresql://user:pass@localhost/switchboard

[auth]
session_ttl_seconds = 86400

[auth.github]
client_id = "your-github-client-id"
client_secret = "your-github-client-secret"
```

### Environment Variable Overrides

All configuration can be overridden using environment variables with the pattern `SWITCHBOARD__SECTION__FIELD`:
- `SWITCHBOARD__DATABASE__URL`
- `SWITCHBOARD__AUTH__GITHUB__CLIENT_ID`
- `SWITCHBOARD__ORCHESTRATOR__OPENROUTER__API_KEY`

## API Endpoints

### Core Endpoints
- `GET /health`: Health check
- `GET /api/models`: List available OpenRouter models
- `POST /api/chat`: Send chat completion request

### Authentication
- `GET /api/auth/github/login`: Get GitHub OAuth authorization URL
- `POST /api/auth/github/callback`: Exchange OAuth code for session token
- `GET /api/auth/dev/token`: Get development session token

### Chat Management
- `GET /api/chats`: List user's chats
- `POST /api/chats`: Create new chat
- `GET /api/chats/:chat_id`: Get chat details
- `PUT /api/chats/:chat_id`: Update chat
- `DELETE /api/chats/:chat_id`: Delete chat

### Message Handling
- `GET /api/chats/:chat_id/messages`: Get chat messages
- `POST /api/chats/:chat_id/messages`: Send new message
- WebSocket at `/ws` for real-time message updates

### Folder Management
- `GET /api/folders`: List folders
- `POST /api/folders`: Create folder
- `PUT /api/folders/:folder_id`: Update folder
- `DELETE /api/folders/:folder_id`: Delete folder

## Testing

### Backend Testing
```bash
cd backend
cargo test                    # Run all tests
cargo test --package auth     # Test specific package
cargo test -- --nocapture     # Show test output
```

### Frontend Testing
The frontend currently focuses on manual testing through the development server. Test setup can be added as needed.

## Development Workflow

1. **Backend Development**: Start with `cargo run --bin switchboard-backend serve`
2. **Frontend Development**: Start with `cd apps/web && bun run dev`
3. **Configuration**: Set up OpenRouter API key and optional GitHub OAuth
4. **Database**: Default SQLite database works for development; PostgreSQL for production
5. **API Documentation**: Available at `/docs` when backend is running

## Key Implementation Details

- **WebSocket Communication**: The frontend uses WebSocket connections for real-time chat updates with automatic subscription management
- **Model Orchestration**: Backend uses the `denkwerk` library for OpenRouter integration with fallback model handling
- **Authentication**: Session-based authentication with configurable TTL and GitHub OAuth integration
- **State Management**: Frontend uses SolidJS signals and effects for reactive state management
- **Error Handling**: Comprehensive error handling throughout the stack with user-friendly error messages