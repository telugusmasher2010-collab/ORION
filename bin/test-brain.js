const fetch = require('node-fetch'); // Using node-fetch for compatibility
const fs = require('fs');
const path = require('path');

const settingsPath = path.join(__dirname, '..', 'CONFIG', 'settings.json');
const settings = JSON.parse(fs.readFileSync(settingsPath, 'utf-8'));
const apiKey = settings.gemini.apiKey;

async function testBrain() {
    console.log('◆ ORION Brain Diagnostic Started...');
    console.log('Testing API Key:', apiKey.substring(0, 5) + '...');

    const models = ['gemini-1.5-flash', 'gemini-1.5-pro', 'gemini-pro'];
    
    for (const model of models) {
        console.log(`\n--- Testing Model: ${model} ---`);
        const url = `https://generativelanguage.googleapis.com/v1beta/models/${model}:generateContent?key=${apiKey}`;
        
        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    contents: [{ parts: [{ text: 'Say "ORION System Active" in Telugu' }] }]
                })
            });
            
            const data = await response.json();
            
            if (response.ok) {
                console.log(`✅ SUCCESS: ${model} is working!`);
                console.log(`Response: ${data.candidates[0].content.parts[0].text}`);
            } else {
                console.log(`❌ FAILED: ${model} returned ${response.status}`);
                console.log(`Error: ${data.error?.message || JSON.stringify(data)}`);
            }
        } catch (err) {
            console.log(`❌ ERROR: Could not connect to ${model}. ${err.message}`);
        }
    }
}

testBrain();
