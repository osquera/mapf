# Copilot Instructions

Goal: Build mapf.dev, a MAPF arena where researchers upload WASM solvers, get a browser smoke test, then a server re-verification that feeds a leaderboard.

## Project Structure & Architecture
- **Root**: `Taskfile.yaml` orchestrates all workflows. `Cargo.toml` defines the Rust workspace.
- **Frontend** (`frontend/`): SvelteKit app.
  - **WASM Integration**: Solvers are built to `frontend/src/lib/wasm/` and copied to `frontend/static/wasm/` for runtime loading.
  - **Map Parsing**: `src/lib/parser.ts` handles MovingAI `.map` and `.scen` formats.
- **Solvers** (`solvers/`): Rust crates.
  - `mapf-astar`: Reference implementation.
  - `mapf-core`: Shared logic and types.
  - `wit/`: WASM Component Model definitions (`mapf-solver.wit`).
- **Maps** (`maps/`): Canonical MovingAI benchmark maps and scenarios.

## Critical Workflows
**ALWAYS use `task` for running commands.**
- **Setup**: `task setup` (installs npm deps, Playwright browsers).
- **Dev**: `task dev` (starts SvelteKit dev server).
- **Test**: `task test` (runs ALL tests: Rust + Frontend Unit + E2E).
  - `task test:rust`: Cargo tests.
  - `task test:unit`: Vitest.
  - `task test:e2e`: Playwright.
- **Build**: `task build` (builds WASM + Frontend).
  - `task build:wasm`: Compiles Rust solvers to WASM and places artifacts in frontend.

## Development Rules
- **Tests First**: Write tests before implementation.
  - Frontend: `frontend/tests/` (E2E) and `src/**/*.test.ts` (Unit).
  - Rust: `#[test]` modules within crates.
- **WASM Contract**: Solvers must adhere to `solvers/wit/mapf-solver.wit`.
- **Determinism**: Browser smoke tests run in Web Workers with deterministic RNG.
- **Map Format**: Strictly follow MovingAI `.map` and `.scen` formats.

## Tech Stack
- **Frontend**: SvelteKit, TypeScript, Tailwind CSS, Vitest, Playwright.
- **Backend/Solvers**: Rust, WASM (wasm32-unknown-unknown), wasm-pack.
- **Build Tool**: Taskfile (go-task).

## Key Files
- `Taskfile.yaml`: Command reference.
- `solvers/wit/mapf-solver.wit`: The API contract for solvers.
- `frontend/src/lib/parser.ts`: Map parsing logic.
- `frontend/src/lib/components/`: UI components (e.g., `MapViewer.svelte`).
