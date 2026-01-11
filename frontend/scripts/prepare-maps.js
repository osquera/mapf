import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Paths
const PROJECT_ROOT = path.resolve(__dirname, '../..');
const MAPS_SRC = path.join(PROJECT_ROOT, 'maps/mapf-map');
const STATIC_DEST = path.join(PROJECT_ROOT, 'frontend/static/maps');

// Ensure destination exists
if (fs.existsSync(STATIC_DEST)) {
    fs.rmSync(STATIC_DEST, { recursive: true, force: true });
}
fs.mkdirSync(STATIC_DEST, { recursive: true });

const index = {
    maps: [],
    scenarios: {}
};

// 1. Copy Maps (.map)
const mapFiles = fs.readdirSync(MAPS_SRC).filter(f => f.endsWith('.map'));
for (const file of mapFiles) {
    fs.copyFileSync(path.join(MAPS_SRC, file), path.join(STATIC_DEST, file));
    index.maps.push(file);
}

// 2. Copy Scenarios (.scen)
// Looking specifically in scen-even/ as per file structure
const SCEN_SRC = path.join(MAPS_SRC, 'scen-even');
if (fs.existsSync(SCEN_SRC)) {
    const scenDest = path.join(STATIC_DEST, 'scen-even');
    fs.mkdirSync(scenDest, { recursive: true });

    const scenFiles = fs.readdirSync(SCEN_SRC).filter(f => f.endsWith('.scen'));
    
    for (const file of scenFiles) {
        fs.copyFileSync(path.join(SCEN_SRC, file), path.join(scenDest, file));
        
        // Parse scenario to find which map it belongs to
        // Format: version 1\n... \nbucket map_filename ...
        const content = fs.readFileSync(path.join(SCEN_SRC, file), 'utf-8');
        const match = content.match(/\t([^\t]+\.map)\t/);
        
        if (match) {
            const mapName = match[1];
            if (!index.scenarios[mapName]) {
                index.scenarios[mapName] = [];
            }
            index.scenarios[mapName].push(`scen-even/${file}`);
        }
    }
}

// Write index
fs.writeFileSync(path.join(STATIC_DEST, 'index.json'), JSON.stringify(index, null, 2));

console.log(`✅ Maps copied to ${STATIC_DEST}`);
console.log(`📊 Index generated with ${index.maps.length} maps and ${Object.keys(index.scenarios).length} scenario groups.`);
