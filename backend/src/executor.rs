/// WASM Component Model executor using wasmtime
/// Loads and executes MAPF solvers with instruction counting and timeout

use anyhow::{anyhow, Context, Result};
use std::time::{Duration, Instant};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::validation::{Coordinate, GridMap, Solution};

/// Stats from solver execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SolverStats {
    pub instruction_count: Option<u64>,
    pub execution_time_ms: u64,
    pub fuel_consumed: Option<u64>,
}

/// Result from solver execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SolverResult {
    pub solution: Option<Solution>,
    pub error: Option<String>,
    pub stats: SolverStats,
}

/// WASM executor with sandboxing and resource limits
pub struct WasmExecutor {
    engine: Engine,
    timeout: Duration,
    fuel_limit: u64,
}

impl WasmExecutor {
    pub fn new(timeout_secs: u64, instruction_limit: u64) -> Result<Self> {
        // Configure engine with fuel metering for instruction counting
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.consume_fuel(true);
        config.epoch_interruption(true);

        let engine = Engine::new(&config)?;

        Ok(Self {
            engine,
            timeout: Duration::from_secs(timeout_secs),
            fuel_limit: instruction_limit,
        })
    }

    /// Execute a WASM solver component
    pub async fn execute(
        &self,
        wasm_bytes: &[u8],
        map: &GridMap,
        starts: &[Coordinate],
        goals: &[Coordinate],
    ) -> Result<SolverResult> {
        let start_time = Instant::now();

        // Create store with fuel
        let mut store = Store::new(&self.engine, ServerWasiState::new()?);
        store.set_fuel(self.fuel_limit)?;
        store.set_epoch_deadline(1);

        // Start epoch thread for timeout
        let engine = self.engine.clone();
        let timeout = self.timeout;
        std::thread::spawn(move || {
            std::thread::sleep(timeout);
            engine.increment_epoch();
        });

        // Load component
        let component = Component::from_binary(&self.engine, wasm_bytes)
            .context("Failed to load WASM component")?;

        // Create linker and add WASI
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)?;

        // Instantiate component
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .context("Failed to instantiate component")?;

        // Get the solve function
        // Based on mapf-solver.wit: solve(map-data, width, height, starts, goals) -> result<solution, string>
        let solve_fn = instance
            .get_typed_func::<(Vec<u8>, u32, u32, Vec<(i32, i32)>, Vec<(i32, i32)>), (Result<Vec<Vec<(i32, i32)>>, String>,)>(&mut store, "solve")
            .context("Failed to get solve function")?;

        // Convert inputs
        let map_data = map.tiles.clone();
        let starts_tuples: Vec<(i32, i32)> = starts.iter().map(|c| (c.x, c.y)).collect();
        let goals_tuples: Vec<(i32, i32)> = goals.iter().map(|c| (c.x, c.y)).collect();

        // Call solver
        let result = solve_fn
            .call_async(
                &mut store,
                (map_data, map.width, map.height, starts_tuples, goals_tuples),
            )
            .await;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let fuel_consumed = self.fuel_limit - store.get_fuel().unwrap_or(0);

        // Handle result
        match result {
            Ok((solver_result,)) => match solver_result {
                Ok(paths) => {
                    // Convert paths to Solution
                    let solution = Solution {
                        paths: paths
                            .into_iter()
                            .map(|path| crate::validation::Path {
                                steps: path
                                    .into_iter()
                                    .map(|(x, y)| Coordinate { x, y })
                                    .collect(),
                            })
                            .collect(),
                    };

                    Ok(SolverResult {
                        solution: Some(solution),
                        error: None,
                        stats: SolverStats {
                            instruction_count: Some(fuel_consumed),
                            execution_time_ms,
                            fuel_consumed: Some(fuel_consumed),
                        },
                    })
                }
                Err(err_msg) => Ok(SolverResult {
                    solution: None,
                    error: Some(err_msg),
                    stats: SolverStats {
                        instruction_count: Some(fuel_consumed),
                        execution_time_ms,
                        fuel_consumed: Some(fuel_consumed),
                    },
                }),
            },
            Err(e) => {
                let error_msg = if e.to_string().contains("epoch") {
                    format!("Solver timeout after {}s", timeout.as_secs())
                } else if e.to_string().contains("fuel") {
                    "Solver exceeded instruction limit".to_string()
                } else {
                    format!("Execution error: {}", e)
                };

                Ok(SolverResult {
                    solution: None,
                    error: Some(error_msg),
                    stats: SolverStats {
                        instruction_count: Some(fuel_consumed),
                        execution_time_ms,
                        fuel_consumed: Some(fuel_consumed),
                    },
                })
            }
        }
    }
}

/// WASI state for the component
struct ServerWasiState {
    ctx: WasiCtx,
}

impl ServerWasiState {
    fn new() -> Result<Self> {
        let ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .build();
        Ok(Self { ctx })
    }
}

impl WasiView for ServerWasiState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        unimplemented!("Resource table not needed for basic execution")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = WasmExecutor::new(30, 10_000_000_000);
        assert!(executor.is_ok());
    }
}
