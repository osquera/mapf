# MAPF Backend Server

Rust backend for mapf.dev arena with WASM solver verification using wasmtime.

## Features

- **WASM Component Model Execution**: Uses wasmtime with Component Model support
- **Instruction Counting**: Deterministic fuel-based instruction metering
- **Authentication**: API key-based authentication for submissions
- **Database**: PostgreSQL for storing submissions and leaderboard
- **Validation**: Comprehensive MAPF solution validation (ported from TypeScript)

## Setup

1. Install PostgreSQL:
   ```bash
   # Ubuntu/Debian
   sudo apt install postgresql

   # macOS
   brew install postgresql
   ```

2. Create database:
   ```bash
   task db:create
   ```

3. Run migrations:
   ```bash
   task db:migrate
   ```

4. Configure environment:
   ```bash
   cp .env.example .env
   # Edit .env with your settings
   ```

5. Start server:
   ```bash
   task dev:backend
   ```

## API Endpoints

### Public Endpoints

- **POST /api/verify** - Verify a WASM solver without storing (testing)
  - Body: `{ wasmBytes: Uint8Array, map: MapData, starts: Coordinate[], goals: Coordinate[] }`
  - Returns: Validation result with stats

- **GET /api/leaderboard** - Get leaderboard entries
  - Query params: `map_name` (optional), `limit` (default: 100)
  - Returns: Array of verified results

### Authenticated Endpoints

- **POST /api/auth/register** - Create user and generate API key
  - Body: `{ username: string, email: string, key_name: string }`
  - Returns: API key (save this!)

- **POST /api/submit** - Submit verified solver to leaderboard
  - Header: `Authorization: Bearer <api_key>`
  - Body: `{ solver_name: string, map_name: string, scenario_id: string, wasmBytes: Uint8Array, map: MapData, starts: Coordinate[], goals: Coordinate[] }`
  - Returns: Submission ID and verification ID

## Docker Deployment

```bash
# Build and run with docker-compose
task docker:run

# Or build manually
task docker:build
docker run -p 3000:3000 --env-file .env mapf-server:latest
```

## Architecture

```
backend/
├── src/
│   ├── main.rs           # Server setup
│   ├── config.rs         # Configuration
│   ├── db.rs             # Database models & queries
│   ├── auth.rs           # API key authentication
│   ├── validation.rs     # MAPF solution validation (ported from TS)
│   ├── executor.rs       # Wasmtime Component Model executor
│   └── api/
│       ├── auth.rs       # Auth endpoints
│       ├── solver.rs     # Verification & submission
│       └── leaderboard.rs # Leaderboard queries
├── migrations/           # SQL migrations
├── Cargo.toml
├── Dockerfile
└── docker-compose.yml
```

## Database Schema

- **users**: User accounts
- **api_keys**: API keys for authentication
- **solver_submissions**: Submitted solvers with metadata
- **verification_results**: Verification results for leaderboard

## Development

```bash
# Run all tests
cargo test

# Check code
cargo clippy

# Format code
cargo fmt

# Run with auto-reload (install cargo-watch)
cargo watch -x run
```
