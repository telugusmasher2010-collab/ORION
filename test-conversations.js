// Check conversations in detail
const fs = require('fs');
const initSqlJs = require('sql.js');

(async () => {
    const SQL = await initSqlJs();
    const db = new SQL.Database(fs.readFileSync('./MEMORY/orion.db'));

    console.log('=== Conversations Table ===');
    const conv = db.exec('SELECT * FROM conversations LIMIT 10');
    if (conv[0]) {
        console.log('Columns:', conv[0].columns);
        console.log('First 10 rows:');
        conv[0].values.forEach(row => {
            console.log('  ', row);
        });
    } else {
        console.log('No conversations found!');
    }

    // Check sessions with messages
    console.log('\n=== Sessions with Messages ===');
    const sessionsWithConv = db.exec(`
        SELECT s.id, s.title, COUNT(c.id) as msg_count
        FROM sessions s
        LEFT JOIN conversations c ON s.id = c.session_id
        GROUP BY s.id
        ORDER BY s.id DESC
    `);
    if (sessionsWithConv[0]) {
        console.log(sessionsWithConv[0].values);
    }

    db.close();
})();