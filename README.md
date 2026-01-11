# mapf.dev — MAPF Arena

A competitive arena for Multi-Agent Path Finding (MAPF) solvers.

## Quick Start

```bash
# Prerequisites: Rust, Node.js 20+, wasm-pack

# Run Rust tests
cargo test --workspace

# Build WASM
wasm-pack build solvers/mapf-core --target web
wasm-pack build solvers/mapf-astar --target web

# Frontend (once created)
cd frontend && npm install && npm run dev
```

## Project Structure

```
├── .github/
│   ├── copilot-instructions.md  # AI agent guidance
│   └── workflows/ci.yml         # CI pipeline
├── maps/
│   └── mapf-map/                # MovingAI benchmark maps (.map)
│       └── scen-even/           # Scenario files (.scen)
├── solvers/
│   ├── wit/mapf-solver.wit      # WASM component model contract
│   ├── mapf-core/               # Map/scenario parser (Rust)
│   └── mapf-astar/              # Reference A* solver (Rust)
├── frontend/                    # SvelteKit app (TODO)
├── benchmarks/                  # Benchmark harness (TODO)
└── infra/                       # Server verification (TODO)
```

## Solver Contract

Solvers must implement the WIT interface in [solvers/wit/mapf-solver.wit](solvers/wit/mapf-solver.wit):

```wit
solve: func(
    map-data: list<u8>,
    width: u32,
    height: u32,
    starts: list<coordinate>,
    goals: list<coordinate>,
) -> result<solution, string>
```

## Maps

Uses the [MovingAI benchmark format](https://movingai.com/benchmarks/):
- `.map` files: Grid maps with passable (`.`) and blocked (`@`) cells
- `.scen` files: Scenario definitions with start/goal pairs and optimal costs

## Development

See [.github/copilot-instructions.md](.github/copilot-instructions.md) for architecture decisions and workflow rules (tests-first, sandboxing, etc.).

## License

MIT
