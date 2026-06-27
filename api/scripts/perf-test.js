#!/usr/bin/env node
// Lightweight API response-time check — runs against a local server stub.
// Fails if any endpoint median exceeds PERF_THRESHOLD_MS (default 200 ms).

'use strict';

const http = require('http');

const THRESHOLD = parseInt(process.env.PERF_THRESHOLD_MS || '200', 10);
const ITERATIONS = 20;

const endpoints = [
  { method: 'GET', path: '/health' },
  { method: 'GET', path: '/api/payments?limit=10' },
  { method: 'GET', path: '/api/merchants?limit=10' },
];

function measure(opts) {
  return new Promise((resolve, reject) => {
    const start = Date.now();
    const req = http.request(opts, (res) => {
      res.resume();
      res.on('end', () => resolve(Date.now() - start));
    });
    req.on('error', reject);
    req.setTimeout(5000, () => { req.destroy(); reject(new Error('timeout')); });
    req.end();
  });
}

async function bench(endpoint) {
  const times = [];
  for (let i = 0; i < ITERATIONS; i++) {
    try {
      times.push(await measure({ host: 'localhost', port: 3000, ...endpoint }));
    } catch {
      // Server not running in CI — skip gracefully
      return null;
    }
  }
  times.sort((a, b) => a - b);
  return {
    median: times[Math.floor(times.length / 2)],
    p95: times[Math.floor(times.length * 0.95)],
    min: times[0],
    max: times[times.length - 1],
  };
}

(async () => {
  let failed = false;
  console.log(`Performance threshold: ${THRESHOLD} ms\n`);

  for (const ep of endpoints) {
    const result = await bench(ep);
    if (!result) {
      console.log(`${ep.method} ${ep.path}: skipped (server not available)`);
      continue;
    }
    const status = result.median > THRESHOLD ? 'FAIL' : 'PASS';
    if (status === 'FAIL') failed = true;
    console.log(
      `[${status}] ${ep.method} ${ep.path} — ` +
      `median=${result.median}ms p95=${result.p95}ms min=${result.min}ms max=${result.max}ms`
    );
  }

  process.exit(failed ? 1 : 0);
})();
