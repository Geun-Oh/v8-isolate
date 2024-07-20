import ivm, { ExternalCopy } from 'isolated-vm';
import { randomUUID } from 'crypto';
import express from 'express';

const isolatePool: Array<ivm.Isolate> = [];
const POOL_SIZE = 10;
const PORT = 3000;
const app = express();

type Event = Record<string, any>;

for (let i = 0; i < POOL_SIZE; i++) {
    const isolate = new ivm.Isolate({ memoryLimit: 128 })
    isolatePool.push(isolate)
}

async function processRequest(event: Event) {
    const startTime = performance.now();
    const startMemory = process.memoryUsage().rss;

    const isolate = isolatePool[Math.floor(Math.random() * POOL_SIZE)]
    const context = isolate.createContextSync();

    const jail = context.global;
    const requestId = randomUUID();

    jail.setSync('global', jail.derefInto());
    jail.setSync('log', function(...args: any[]) {
        console.log(...args);
    });
    
    context.evalSync(`log("${requestId}")`)

    const stringResult = context.evalClosureSync(`
    return JSON.stringify({
        statusCode: 200,
        body: {
            message: "Hello from isolated env",
            requestId: $0,
            timestamp: new Date().toISOString()
        }
    })
    `, [requestId])

    console.log(stringResult)

    const result = JSON.parse(stringResult)

    const endTime = performance.now();
    const endMemory = process.memoryUsage().rss;

    const executionTime = endTime - startTime;
    const memoryUsed = endMemory - startMemory;

    console.log(`Execution time: ${executionTime.toFixed(2)} ms`);
    console.log(`Memory used: ${(memoryUsed / 1024 / 1024).toFixed(2)} MB`);

    const parsedResult = result.body;
    parsedResult.executionTime = `${executionTime.toFixed(2)} ms`;
    parsedResult.memoryUsed = `${(memoryUsed / 1024 / 1024).toFixed(2)} MB`;

    return {
        ...result,
        body: JSON.stringify(parsedResult)
    };
}

app.get('/', async (req, res) => {
    try {
        const result = await processRequest(req.query);
        res.status(result.statusCode).json(JSON.parse(result.body));
    } catch (err) {
        console.error('Error processing request:', err);
        res.status(500).json({ error: 'Internal server error' });
    }
});

app.listen(PORT, () => {
    console.log(`Server running at http://localhost:${PORT}`);
});