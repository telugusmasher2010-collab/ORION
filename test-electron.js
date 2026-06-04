try {
    const electron = require('electron');
    console.log('electron type:', typeof electron);
    console.log('electron keys:', Object.keys(electron).slice(0, 5));
    console.log('app:', typeof electron.app);
    console.log('ipcMain:', typeof electron.ipcMain);
} catch (e) {
    console.error('Error:', e.message);
}
setTimeout(() => process.exit(0), 1000);
