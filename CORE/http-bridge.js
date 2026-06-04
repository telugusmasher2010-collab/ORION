/**
 * ORION — HTTP Bridge (Node.js)
 * Called by Rust backend to make streaming HTTP requests.
 * Receives request config via stdin (JSON), outputs SSE chunks via stdout.
 */

let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => { input += chunk; });
process.stdin.on('end', async () => {
    try {
        const config = JSON.parse(input);
        await makeRequest(config);
    } catch (err) {
        process.stderr.write(JSON.stringify({ error: err.message }) + '\n');
        process.exit(1);
    }
});

async function makeRequest(config) {
    const { method, url, headers, body, stream } = config;

    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 60000);

    try {
        const response = await fetch(url, {
            method: method || 'POST',
            headers: { 'Content-Type': 'application/json', ...headers },
            body: body ? JSON.stringify(body) : undefined,
            signal: controller.signal,
        });

        clearTimeout(timeout);

        if (!response.ok) {
            const errText = await response.text();
            process.stderr.write(JSON.stringify({
                error: `HTTP ${response.status}: ${errText}`
            }) + '\n');
            process.exit(1);
        }

        if (!stream || !response.body) {
            const text = await response.text();
            process.stdout.write(text + '\n');
            process.stdout.on('drain', () => process.exit(0));
            return;
        }

        // SSE streaming
        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = '';

        while (true) {
            const { done, value } = await reader.read();
            if (done) break;

            buffer += decoder.decode(value, { stream: true });
            const lines = buffer.split('\n');
            buffer = lines.pop() || '';

            for (const line of lines) {
                const trimmed = line.trim();
                if (!trimmed || trimmed === 'data: [DONE]') continue;

                let data = trimmed;
                if (trimmed.startsWith('data: ')) {
                    data = trimmed.substring(6);
                }

                try {
                    const json = JSON.parse(data);
                    if (json.error) {
                        process.stderr.write(JSON.stringify({
                            error: json.error.message || json.error
                        }) + '\n');
                        process.exit(1);
                        return;
                    }

                    // OpenAI-compatible: choices[0].delta.content
                    let text = null;
                    if (json.choices && json.choices[0]) {
                        text = json.choices[0].delta?.content || json.choices[0].text;
                    }
                    // Gemini: candidates[0].content.parts[0].text
                    if (!text && json.candidates && json.candidates[0]) {
                        text = json.candidates[0].content?.parts?.[0]?.text;
                    }
                    // Ollama: message.content
                    if (!text && json.message && json.message.content) {
                        text = json.message.content;
                    }

                    if (text) {
                        process.stdout.write(JSON.stringify({ chunk: text }) + '\n');
                    }
                } catch (e) {
                    // Skip malformed lines
                }
            }
        }

        process.stdout.write(JSON.stringify({ done: true }) + '\n');
        process.exit(0);
    } catch (err) {
        clearTimeout(timeout);
        if (err.name === 'AbortError') {
            process.stderr.write(JSON.stringify({ error: 'Request timed out (60s)' }) + '\n');
        } else {
            process.stderr.write(JSON.stringify({ error: err.message }) + '\n');
        }
        process.exit(1);
    }
}
