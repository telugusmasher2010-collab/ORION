console.log('process.type:', process.type);
console.log('process.resourcesPath:', process.resourcesPath);

// Check if we're in the main process
const electron = require('electron');
console.log('electron type:', typeof electron);
console.log('electron:', JSON.stringify(electron).substring(0, 100));

if (typeof electron === 'object' && electron.app) {
    console.log('Electron API loaded successfully!');
    console.log('app.isReady:', electron.app.isReady());
} else {
    console.log('Electron API NOT loaded - require returned:', typeof electron);

    // Try accessing the internal electron module
    try {
        const internal = process._linkedBinding('electron_common_features');
        console.log('internal binding:', typeof internal);
    } catch (e) {
        console.log('No internal binding:', e.message);
    }
}

process.exit(0);
