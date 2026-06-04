const fs = require('fs');
const initSqlJs = require('sql.js');

async function test() {
    console.log('=== Testing ORION Database ===\n');

    const SQL = await initSqlJs();
    const db = new SQL.Database(fs.readFileSync('./MEMORY/orion.db'));

    // Check tables
    console.log('--- Tables ---');
    const tables = db.exec("SELECT name FROM sqlite_master WHERE type='table'");
    console.log('Tables:', tables[0].values.map(r => r[0]));

    // Check sessions
    console.log('\n--- Sessions ---');
    try {
        const sessions = db.exec('SELECT * FROM sessions');
        if (sessions[0]) {
            console.log('Columns:', sessions[0].columns);
            console.log('Rows:', sessions[0].values);
        } else {
            console.log('No sessions found!');
        }
    } catch (e) {
        console.log('Sessions error:', e.message);
    }

    // Check projects
    console.log('\n--- Projects ---');
    try {
        const projects = db.exec('SELECT * FROM projects');
        if (projects[0]) {
            console.log('Columns:', projects[0].columns);
            console.log('Rows:', projects[0].values);
        } else {
            console.log('No projects found');
        }
    } catch (e) {
        console.log('Projects error:', e.message);
    }

    db.close();
    console.log('\n=== Test Complete ===');
}

test().catch(console.error);