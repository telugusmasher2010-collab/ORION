console.log('Starting test...');
console.log('process.versions:', JSON.stringify(process.versions));
console.log('process.type:', process.type);

try {
    const electron = require('electron');
    console.log('require result type:', typeof electron);
    if (typeof electron === 'object') {
        console.log('app:', typeof electron.app);
        console.log('ipcMain:', typeof electron.ipcMain);
    } else {
        console.log('Got string:', electron);
    }
} catch (e) {
    console.error('require error:', e.message);
}

process.exit(0);
