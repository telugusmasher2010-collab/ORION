console.log('Testing electron module resolution...');

// Check require cache
const Module = require('module');
const path = require('path');

// Check if electron is in the cache
const electronPath = require.resolve('electron');
console.log('Resolved electron path:', electronPath);

// Check the module's paths
const mod = new Module('test');
mod.paths = Module._nodeModulePaths(process.cwd());
console.log('Module paths (first 3):', mod.paths.slice(0, 3));

// Try to manually load the electron module
try {
    // In Electron, the electron module should be a built-in
    const builtins = Module.builtinModules || [];
    console.log('electron is builtin:', builtins.includes('electron'));
} catch (e) {
    console.log('Error checking builtins:', e.message);
}

// Check if there's an electron.js in the project root
const fs = require('fs');
const projectRoot = process.cwd();
const files = fs.readdirSync(projectRoot).filter(f => f.startsWith('electron'));
console.log('electron* files in project root:', files);

// Try to require the electron internal module directly
try {
    // In Electron 35, the module might be loaded via a different mechanism
    const electronModule = process._linkedBinding('electron_browser_app');
    console.log('electron_browser_app:', typeof electronModule);
} catch (e) {
    console.log('No electron_browser_app binding:', e.message);
}

process.exit(0);
