<script>
    // Simple docs page
</script>

<div class="docs-container">
    <h1>Documentation & Roadmap</h1>

    <section>
        <h2>Workflows & Architecture</h2>
        <p>
            The MAPF Arena is built using <strong>SvelteKit</strong> (frontend) and <strong>Rust</strong> (solvers), 
            orchestrated by a <code>Taskfile.yaml</code> in the root.
        </p>

        <h3>Key Commands</h3>
        <ul>
            <li><code>task dev</code>: Starts the frontend development server.</li>
            <li><code>task test</code>: Runs all tests (Rust unit tests, Frontend unit tests, and E2E tests).</li>
            <li><code>task build:wasm</code>: Compiles the built-in Rust solver (`mapf-astar`) to WASM.</li>
            <li><code>task build:test-component</code>: Compiles the example Component Model solver.</li>
        </ul>

        <h3>Architecture</h3>
        <p>
            The system runs solvers entirely in the browser using <strong>Web Workers</strong>. 
            Two types of WASM modules are supported:
        </p>
        <ol>
            <li>
                <strong>Legacy (Bindgen)</strong>: Uses `wasm-bindgen` to generate a JS/WASM pair. 
                This is used for the built-in solver.
            </li>
            <li>
                <strong>Component Model (Option 2)</strong>: Uses the WASM Component Model (`.wasm` single file).
                These are transpiled on-the-fly in the browser using <code>@bytecodealliance/jco</code>.
            </li>
        </ol>
    </section>

    <section>
        <h2>Solver Interface</h2>
        <p>
            Solvers must implement the WIT interface defined in <code>solvers/wit/mapf-solver.wit</code>. 
            The system expects a <strong>WASM Component</strong> (not a core WASM module) that exports this interface.
        </p>

        <h3>Function Signature</h3>
        <pre>solve: func(
    map-data: list&lt;u8&gt;,         // Flattened grid (row-major). 1 = passable, 0 = blocked.
    width: u32,                  // Grid width
    height: u32,                 // Grid height
    starts: list&lt;coordinate&gt;,   // Start positions [&#123;x,y&#125;, ...]
    goals: list&lt;coordinate&gt;     // Goal positions [&#123;x,y&#125;, ...]
) -> result&lt;solution, string&gt;;</pre>

        <h3>Data Types</h3>
        <ul>
            <li><strong>Coordinate</strong>: <code>{'record { x: u32, y: u32 }'}</code></li>
            <li><strong>Path</strong>: <code>{'record { steps: list<coordinate> }'}</code> (Must include start and goal)</li>
            <li><strong>Solution</strong>: <code>{'record { paths: list<path>, cost: u64 }'}</code></li>
        </ul>
    </section>

    <section>
        <h2>How to Build a Solver (Rust)</h2>
        <p>To create a compatible solver using Rust:</p>
        
        <ol>
            <li>
                <strong>Setup Crate</strong>: Create a <code>lib</code> crate with <code>crate-type = ["cdylib"]</code>.
            </li>
            <li>
                <strong>Add Dependencies</strong>: In <code>Cargo.toml</code>, add:
                <pre>[dependencies]
wit-bindgen = "0.36.0"</pre>
            </li>
            <li>
                <strong>Implement Interface</strong>: Use <code>wit_bindgen::generate!</code> to reference the WIT file and implement the <code>Guest</code> trait.
            </li>
            <li>
                <strong>Build Core WASM</strong>:
                <pre>cargo build --target wasm32-unknown-unknown --release</pre>
            </li>
            <li>
                <strong>Componentize</strong>: Convert the core WASM to a Component. You can use <a href="https://github.com/bytecodealliance/jco">jco</a> or <code>wasm-tools</code>.
                <pre># Embed the WIT interface
jco embed wit/mapf-solver.wit target/wasm32-unknown-unknown/release/my_solver.wasm -o embedded.wasm

# Create the component
jco new embedded.wasm -o my-solver-component.wasm</pre>
            </li>
            <li>
                <strong>Upload</strong>: Upload the resulting <code>my-solver-component.wasm</code> file to the Arena.
            </li>
        </ol>
    </section>

    <section class="todo-section">
        <h2>Roadmap & Missing Features</h2>
        <div class="todo-list">
            <div class="todo-item done">
                <input type="checkbox" checked disabled />
                <span>Map & Scenario Selection (MovingAI format)</span>
            </div>
            <div class="todo-item done">
                <input type="checkbox" checked disabled />
                <span>Browser-based WASM Runner (Web Workers)</span>
            </div>
            <div class="todo-item done">
                <input type="checkbox" checked disabled />
                <span>Custom WASM Component Upload & JCO Transpilation</span>
            </div>
            <div class="todo-item">
                <input type="checkbox" disabled />
                <span><strong>Backend Algorithm Support</strong>: Frontend sends `algorithm`/`heuristic` options, but Rust solvers currently ignore them.</span>
            </div>
            <div class="todo-item">
                <input type="checkbox" disabled />
                <span><strong>Large Map Optimization</strong>: Move map parsing (JS) to a worker thread to prevent UI freeze on large maps.</span>
            </div>
            <div class="todo-item">
                <input type="checkbox" disabled />
                <span><strong>Persistent Leaderboard</strong>: Connect the "Leaderboard" page to a backend API/Database.</span>
            </div>
        </div>
    </section>
</div>

<style>
    .docs-container {
        max-width: 800px;
        margin: 0 auto;
        padding: 2rem;
        color: #e0e0e0;
    }

    h1, h2, h3 {
        color: #fff;
    }

    h1 { margin-bottom: 2rem; border-bottom: 1px solid #333; padding-bottom: 1rem; }
    h2 { margin-top: 2rem; color: #64ffda; }
    
    section {
        background: #1e1e2f;
        padding: 1.5rem;
        border-radius: 8px;
        margin-bottom: 2rem;
    }

    ul, ol {
        margin-left: 1.5rem;
        line-height: 1.6;
    }

    code {
        background: #2a2a40;
        padding: 0.2rem 0.4rem;
        border-radius: 4px;
        font-family: 'Fira Code', monospace;
        color: #ff79c6;
    }

    pre {
        background: #111;
        padding: 1rem;
        border-radius: 6px;
        overflow-x: auto;
    }

    .todo-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .todo-item {
        display: flex;
        align-items: flex-start;
        gap: 0.8rem;
        background: #252538;
        padding: 0.8rem;
        border-radius: 4px;
    }

    .todo-item.done {
        opacity: 0.6;
        text-decoration: line-through;
    }

    .todo-item input {
        margin-top: 0.3rem;
        accent-color: #64ffda;
    }
</style>
