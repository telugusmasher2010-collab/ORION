// Simple test - does require('electron') return the API?
const electron = require('electron');
if (typeof electron === 'object' && electron.app) {
    console.log('SUCCESS: Electron API loaded');
    console.log('app version:', electron.app.getVersion());
    process.exit(0);
} else {
    console.log('FAIL: require("electron") returned:', typeof electron);
    process.exit(1);
}
