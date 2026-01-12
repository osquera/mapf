# mapf.dev — MAPF Arena

A competitive arena for Multi-Agent Path Finding (MAPF) solvers with hybrid frontend/backend architecture.

**Frontend**: Cloudflare Pages (browser smoke tests)  
**Backend**: Hetzner server (deterministic verification with wasmtime)

## Quick Start

```bash
# Prerequisites: Rust, Node.js 20+, wasm-pack, Task, PostgreSQL

# Setup everything
task setup

# Start frontend + backend (parallel)
task dev:fullstack

# Or individually:
task dev              # Frontend only (port 5173)
task dev:backend      # Backend only (port 3000)
```

## Architecture

### Frontend (Cloudflare Pages)
- **SvelteKit** app with browser-based WASM execution for built-in solver
- **Built-in solver** runs in browser using wasm-bindgen (reference A* implementation)
- **Custom solvers** upload directly to backend (Cloudflare can't run JCO)
- **MapViewer** component for visualizing solutions
- **No node:fs or JCO** - fully compatible with Cloudflare Pages edge runtime

### Backend (Rust on Hetzner)
- **Axum** web server with PostgreSQL database
- **Wasmtime** executes WASM with instruction counting (fuel metering)
- **Validation** ported from TypeScript (cardinal moves, collision detection)
- **API key auth** for leaderboard submissions
- **Deterministic benchmarking** - precise instruction counts, not wall-clock time

## Project Structure

```
├── backend/                     # Rust API server
│   ├── src/
│   │   ├── main.rs              # Axum server setup
│   │   ├── executor.rs          # Wasmtime WASM execution
│   │   ├── validation.rs        # MAPF solution validation
│   │   ├── auth.rs              # API key authentication
│   │   ├── db.rs                # Database models & queries
│   │   └── api/                 # HTTP endpoints
│   ├── migrations/              # SQL migrations
│   ├── Dockerfile               # Production container
│   └── docker-compose.yml       # Local dev environment
├── frontend/                    # SvelteKit app
│   ├── src/
│   │   ├── lib/
│   │   │   ├── api/             # Backend API client
│   │   │   ├── solver/          # WASM runner + JCO loader
│   │   │   ├── parser.ts        # MovingAI format parser
│   │   │   └── validation.ts    # Browser validation
│   │   └── routes/
│   │       ├── arena/           # Solver arena UI
│   │       ├── upload/          # Solver upload page
│   │       └── leaderboard/     # Rankings
│   └── static/
│       ├── maps/                # Benchmark maps (copied at build)
│       └── wasm/                # Built-in solvers
├── solvers/
│   ├── wit/mapf-solver.wit      # WASM Component Model contract
│   ├── mapf-core/               # Shared Rust types
│   └── mapf-astar/              # Reference A* solver
├── maps/
│   └── mapf-map/                # MovingAI benchmark maps
│       └── scen-even/           # Scenario files
└── Taskfile.yaml                # Build orchestration
```

## Workflow

1. **User uploads WASM solver** in browser (or uses built-in solver)
2. **Built-in solver**: Runs browser smoke test with wasm-bindgen
3. **Custom solver**: Skips browser, sends directly to backend
4. **Server verification**: Backend runs solver with wasmtime fuel metering
5. **Validation**: Checks MAPF rules (cardinal moves, no collisions, etc.)
6. **Leaderboard**: Valid submissions stored with instruction counts

## API Endpoints

### Public
- `GET /health` - Health check
- `POST /api/verify` - Test WASM solver (no auth)
- `GET /api/leaderboard?map_name=...&limit=100` - Get rankings

### Authenticated (requires `Authorization: Bearer <api_key>`)
- `POST /api/auth/register` - Create user + API key
- `POST /api/submit` - Submit solver to leaderboard

## Database Setup

```bash
# Create database
task db:create

# Run migrations
task db:migrate

# Or use Docker
cd backend
docker-compose up -d postgres
```

## Solver Contract

Solvers implement [solvers/wit/mapf-solver.wit](solvers/wit/mapf-solver.wit):

```wit
solve: func(
    map-data: list<u8>,
    width: u32,
    height: u32,
    starts: list<coordinate>,
    goals: list<coordinate>,
) -> result<solution, string>
```

Dual execution:
- **Browser**: Built-in solver runs with wasm-bindgen (wasm-pack output)
- **Server**: Custom solvers run with Wasmtime Component Model natively

## Maps

Uses [MovingAI benchmark format](https://movingai.com/benchmarks/):
- `.map` files: Grid maps with passable (`.`) and blocked (`@`) cells
- `.scen` files: Scenarios with start/goal pairs and optimal costs

## Deployment

### Frontend (Cloudflare Pages)
```bash
task build:frontend
# Deploy frontend/.svelte-kit/cloudflare to Cloudflare Pages
```

### Backend (Hetzner)
```bash
# Build Docker image
task docker:build

# Or build for production
task build:backend
# Binary: target/release/mapf-server
```

See [backend/README.md](backend/README.md) for detailed deployment instructions.

## Development

- **Tests first**: Write tests before implementation
- **No JCO in production**: Custom WASM solvers run on backend only
- **Validation consistency**: Same rules in TypeScript (browser) and Rust (server)
- **Deterministic fuel**: Instruction counting via Wasmtime fuel metering

See [.github/copilot-instructions.md](.github/copilot-instructions.md) for full architecture rules.

## License

MIT

