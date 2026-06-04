const path = require('path');
const fs = require('fs');

// Import MemoryEngine
const MemoryEngine = require('./CORE/memory-engine.js');

async function test() {
    console.log('=== Testing MemoryEngine Directly ===\n');

    const memory = new MemoryEngine();
    await memory.init();

    console.log('Memory ready:', memory.ready);

    // Test getSessions
    const sessions = memory.getSessions();
    console.log('\nSessions:', sessions.length, sessions);

    // Test createSession
    const newId = memory.createSession('Test Session', 1);
    console.log('Created session ID:', newId);

    // Test getSessions again
    const sessions2 = memory.getSessions();
    console.log('Sessions after create:', sessions2.length, sessions2);

    memory.close();
    console.log('\n=== Test Complete ===');
}

test().catch(console.error);