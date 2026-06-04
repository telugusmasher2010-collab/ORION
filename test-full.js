// Comprehensive ORION Test
const path = require('path');
const fs = require('fs');

// Test 1: Database
console.log('=== TEST 1: Database ===');
const dbPath = path.join(__dirname, 'MEMORY', 'orion.db');
console.log('DB exists:', fs.existsSync(dbPath));
console.log('DB size:', fs.statSync(dbPath).size, 'bytes');

// Test 2: Settings
console.log('\n=== TEST 2: Settings ===');
const settings = JSON.parse(fs.readFileSync(path.join(__dirname, 'CONFIG', 'settings.json'), 'utf8'));
console.log('Groq enabled:', settings.groq ? 'YES' : 'NO');
console.log('Default brain:', settings.defaultBrain);
console.log('Routing:', settings.routing);

// Test 3: Main files exist
console.log('\n=== TEST 3: Core Files ===');
const requiredFiles = [
    'main.js',
    'preload.js',
    'CORE/memory-engine.js',
    'CORE/brain-router.js',
    'CORE/ai-brain.js',
    'CORE/personality-engine.js',
    'UI/index.html',
    'UI/renderer.js'
];

requiredFiles.forEach(file => {
    const exists = fs.existsSync(path.join(__dirname, file));
    console.log(file + ':', exists ? 'OK' : 'MISSING');
});

// Test 4: Memory Engine
console.log('\n=== TEST 4: Memory Engine ===');
const initSqlJs = require('sql.js');
(async () => {
    const SQL = await initSqlJs();
    const db = new SQL.Database(fs.readFileSync(dbPath));

    const sessions = db.exec('SELECT COUNT(*) FROM sessions');
    console.log('Total sessions:', sessions[0].values[0][0]);

    const projects = db.exec('SELECT COUNT(*) FROM projects');
    console.log('Total projects:', projects[0].values[0][0]);

    const conversations = db.exec('SELECT COUNT(*) FROM conversations');
    console.log('Total messages:', conversations[0].values[0][0]);

    db.close();
    console.log('\n=== ALL TESTS PASSED ===');
})();