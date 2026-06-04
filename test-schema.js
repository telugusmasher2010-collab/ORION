// Check conversations table schema
const fs = require('fs');
const initSqlJs = require('sql.js');

(async () => {
    const SQL = await initSqlJs();
    const db = new SQL.Database(fs.readFileSync('./MEMORY/orion.db'));

    console.log('=== Conversations Table Schema ===');
    const schema = db.exec("PRAGMA table_info(conversations)");
    if (schema[0]) {
        console.log('Columns:');
        schema[0].values.forEach(row => {
            console.log(' ', row[1], '-', row[2]);
        });
    } else {
        console.log('Table does not exist!');
    }

    db.close();
})();