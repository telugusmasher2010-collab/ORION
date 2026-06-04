const apiKey = "AIzaSyCXPcALQDFKDch25pzKy-v4uRFp_3ky9VQ";
const url = `https://generativelanguage.googleapis.com/v1beta/models?key=${apiKey}`;

async function checkModels() {
    console.log("◆ DIAGNOSTIC: Fetching available models for your key...");
    try {
        const response = await fetch(url);
        const data = await response.json();
        
        if (response.ok) {
            console.log("✅ API KEY IS ACTIVE!");
            console.log("\n--- SUPPORTED MODELS ---");
            data.models.forEach(m => {
                console.log(`- ${m.name.split('/').pop()} (${m.supportedGenerationMethods.join(', ')})`);
            });
        } else {
            console.log("❌ API KEY ERROR!");
            console.log(`Status: ${response.status}`);
            console.log(`Reason: ${data.error?.message || JSON.stringify(data)}`);
        }
    } catch (err) {
        console.log(`❌ CONNECTION ERROR: ${err.message}`);
    }
}

checkModels();
