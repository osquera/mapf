export interface TranspiledComponent {
    jsUrl: string;
    /** The core WASM blob URL, baked into the JS but returned here for reference/cleanup */
    wasmUrl: string;
}

/**
 * Transpiles a WASM Component into ES Modules executable in the browser.
 * @param buffer The raw bytes of the .wasm component
 */
export async function transpileComponent(buffer: ArrayBuffer): Promise<TranspiledComponent> {
    if (import.meta.env.SSR) {
        throw new Error('transpileComponent cannot be called on the server');
    }

    const { transpile } = await import('@bytecodealliance/jco');
        
    const name = 'component';
    
    // Transpile the component
    // We omit 'instantiation' to use the default ESM integration mode.
    // Previous attempts with 'async' or 'true' caused validation errors in the internal bindgen.
    const result = await transpile(new Uint8Array(buffer), { 
        name,
        noNodejsCompat: true, 
        map: []
    });

    // In the browser (via `src/browser.js`), jco returns `files` as an array of tuples `[name, bytes][]`.
    // In Node.js (via `src/cmd/transpile.js`), it returns a `Record<name, bytes>`.
    // We normalize to an array of entries to handle both environments.
    const outputFiles = result.files as unknown as (Record<string, Uint8Array> | [string, Uint8Array][]);
    const entries = Array.isArray(outputFiles) 
        ? outputFiles 
        : Object.entries(outputFiles);
    
    // Find the main JS file and the core WASM file
    // We search dynamically to be robust against naming variations
    const jsEntry = entries.find(([n]) => n && n.endsWith('.js'));
    const wasmEntry = entries.find(([n]) => n && n.endsWith('.wasm'));
    
    if (!jsEntry || !wasmEntry) {
        console.error('Missing output files. Available:', entries.map(e => e[0]));
        throw new Error('Transpilation failed: missing output files');
    }

    const [jsName, jsBytes] = jsEntry;
    const [wasmName, wasmBytes] = wasmEntry;

    // Explicitly cast to Uint8Array for Blob constructor
    const wasmArray = new Uint8Array(wasmBytes as unknown as ArrayBuffer);
    
    // Create Blob for the Core WASM
    const wasmBlob = new Blob([wasmArray], { type: 'application/wasm' });
    const wasmUrl = URL.createObjectURL(wasmBlob);
    
    // Modify the JS to import the WASM from the Blob URL instead of a relative path
    let jsSource = new TextDecoder().decode(jsBytes);
    
    // Debug: Log the JS source to see what we're matching against if it fails
    // console.log('Transpiled JS Source Preview:', jsSource.substring(0, 500));
    
    // JCO output usually looks like: new URL('./component.core.wasm', import.meta.url)
    // We replace it with our Blob URL
    // We use a regex that handles the filename dynamically
    const regex = new RegExp(`new URL\\(['"]\\./${wasmName}['"],\\s*import\\.meta\\.url\\)`, 'g');
    
    if (!regex.test(jsSource)) {
        console.warn(`JCO Loader: Regex did not match any URL instantiation for ${wasmName}. this might fail.`);
        console.warn('Regex:', regex);
    }
    
    jsSource = jsSource.replace(regex, `new URL('${wasmUrl}', import.meta.url)`);
    
    // Create Blob for the JS
    const jsBlob = new Blob([jsSource], { type: 'application/javascript' });
    const jsUrl = URL.createObjectURL(jsBlob);
    
    return { jsUrl, wasmUrl };
}
