const http = require('http');
const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

const PORT = 8080;

// Path to Fairy Stockfish
const FAIRY_SF_PATH = process.env.FAIRY_SF_PATH || '/opt/homebrew/bin/fairy-stockfish';

// MIME types
const MIME_TYPES = {
    '.html': 'text/html',
    '.css': 'text/css',
    '.js': 'application/javascript',
    '.pgn': 'text/plain',
};

// UCI engine interface
class UCIEngine {
    constructor(enginePath) {
        this.enginePath = enginePath;
        this.process = null;
        this.ready = false;
        this.callbacks = [];
    }

    start() {
        return new Promise((resolve, reject) => {
            try {
                this.process = spawn(this.enginePath);
                
                let output = '';
                this.process.stdout.on('data', (data) => {
                    output += data.toString();
                    const lines = output.split('\n');
                    output = lines.pop(); // Keep incomplete line
                    
                    for (const line of lines) {
                        this.handleLine(line.trim());
                    }
                });

                this.process.stderr.on('data', (data) => {
                    console.error('Engine stderr:', data.toString());
                });

                this.process.on('error', (err) => {
                    console.error('Engine error:', err);
                    reject(err);
                });

                // Initialize UCI
                this.send('uci');
                this.waitFor('uciok').then(() => {
                    this.send('setoption name UCI_Variant value amazon');
                    this.send('isready');
                    return this.waitFor('readyok');
                }).then(() => {
                    this.ready = true;
                    resolve();
                });
            } catch (err) {
                reject(err);
            }
        });
    }

    send(command) {
        if (this.process) {
            this.process.stdin.write(command + '\n');
        }
    }

    handleLine(line) {
        // Check callbacks
        for (let i = this.callbacks.length - 1; i >= 0; i--) {
            const cb = this.callbacks[i];
            if (cb.check(line)) {
                cb.resolve(line);
                this.callbacks.splice(i, 1);
            }
        }
    }

    waitFor(keyword) {
        return new Promise((resolve) => {
            this.callbacks.push({
                check: (line) => line.includes(keyword),
                resolve: resolve
            });
        });
    }

    async analyze(fen, depth = 20) {
        if (!this.ready) {
            throw new Error('Engine not ready');
        }

        return new Promise((resolve) => {
            let bestMove = null;
            let score = null;
            let scoreDepth = 0;

            const infoHandler = {
                check: (line) => {
                    if (line.startsWith('info depth')) {
                        // Parse score from info line
                        const depthMatch = line.match(/depth (\d+)/);
                        const scoreMatch = line.match(/score (cp|mate) (-?\d+)/);
                        const pvMatch = line.match(/pv (\S+)/);
                        
                        if (depthMatch && scoreMatch) {
                            const d = parseInt(depthMatch[1]);
                            if (d >= scoreDepth) {
                                scoreDepth = d;
                                if (scoreMatch[1] === 'cp') {
                                    score = { cp: parseInt(scoreMatch[2]), depth: d };
                                } else {
                                    score = { mate: parseInt(scoreMatch[2]), depth: d };
                                }
                                if (pvMatch) {
                                    bestMove = pvMatch[1];
                                }
                            }
                        }
                    }
                    return false; // Don't remove this handler
                },
                resolve: () => {}
            };

            this.callbacks.push(infoHandler);

            const bestMoveHandler = {
                check: (line) => line.startsWith('bestmove'),
                resolve: (line) => {
                    // Remove info handler
                    const idx = this.callbacks.indexOf(infoHandler);
                    if (idx >= 0) this.callbacks.splice(idx, 1);
                    
                    const match = line.match(/bestmove (\S+)/);
                    if (match) {
                        bestMove = match[1];
                    }
                    resolve({ bestMove, score });
                }
            };

            this.callbacks.push(bestMoveHandler);

            this.send('ucinewgame');
            this.send(`position fen ${fen}`);
            this.send(`go depth ${depth}`);
        });
    }

    stop() {
        if (this.process) {
            this.send('quit');
            this.process.kill();
            this.process = null;
            this.ready = false;
        }
    }
}

// Global engine instance
let engine = null;

async function initEngine() {
    if (!engine) {
        engine = new UCIEngine(FAIRY_SF_PATH);
        try {
            await engine.start();
            console.log('Fairy Stockfish initialized');
        } catch (err) {
            console.error('Failed to start engine:', err.message);
            console.log('Analysis endpoint will not be available');
            engine = null;
        }
    }
    return engine;
}

// HTTP server
const server = http.createServer(async (req, res) => {
    // CORS headers
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

    if (req.method === 'OPTIONS') {
        res.writeHead(204);
        res.end();
        return;
    }

    // API endpoint for analysis
    if (req.method === 'POST' && req.url === '/api/analyze') {
        let body = '';
        req.on('data', chunk => body += chunk);
        req.on('end', async () => {
            try {
                const { fen, depth = 15 } = JSON.parse(body);
                
                if (!engine) {
                    res.writeHead(503, { 'Content-Type': 'application/json' });
                    res.end(JSON.stringify({ error: 'Engine not available' }));
                    return;
                }

                const result = await engine.analyze(fen, depth);
                res.writeHead(200, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify(result));
            } catch (err) {
                res.writeHead(500, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({ error: err.message }));
            }
        });
        return;
    }

    // Static file serving
    let filePath = req.url === '/' ? '/index.html' : req.url;
    filePath = path.join(__dirname, filePath);

    const ext = path.extname(filePath);
    const contentType = MIME_TYPES[ext] || 'application/octet-stream';

    fs.readFile(filePath, (err, data) => {
        if (err) {
            if (err.code === 'ENOENT') {
                res.writeHead(404);
                res.end('Not found');
            } else {
                res.writeHead(500);
                res.end('Server error');
            }
            return;
        }

        res.writeHead(200, { 'Content-Type': contentType });
        res.end(data);
    });
});

// Start server
initEngine().then(() => {
    server.listen(PORT, () => {
        console.log(`Server running at http://localhost:${PORT}`);
        console.log('Press Ctrl+C to stop');
    });
});

// Cleanup on exit
process.on('SIGINT', () => {
    console.log('\nShutting down...');
    if (engine) engine.stop();
    process.exit();
});
