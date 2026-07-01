import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('error_rate');
const paymentDuration = new Trend('payment_duration');

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

// ── Scenario Definitions ──────────────────────────────────────────────────────

export const options = {
  scenarios: {
    // 1. Ramp-up: gradually increase load
    ramp_up: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 50 },
        { duration: '5m', target: 50 },
        { duration: '2m', target: 0 },
      ],
      tags: { scenario: 'ramp_up' },
    },
    // 2. Sustained load
    sustained: {
      executor: 'constant-vus',
      vus: 100,
      duration: '10m',
      startTime: '9m',
      tags: { scenario: 'sustained' },
    },
    // 3. Spike test
    spike: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 500 },
        { duration: '1m',  target: 500 },
        { duration: '30s', target: 0 },
      ],
      startTime: '20m',
      tags: { scenario: 'spike' },
    },
    // 4. Endurance test (run separately via SCENARIO=endurance)
    endurance: {
      executor: 'constant-vus',
      vus: 50,
      duration: '2h',
      startTime: '0s',
      tags: { scenario: 'endurance' },
      exec: 'enduranceTest',
    },
  },

  thresholds: {
    http_req_duration: ['p(95)<2000', 'p(99)<5000'],
    error_rate:        ['rate<0.05'],
    http_req_failed:   ['rate<0.05'],
  },
};

// ── Helpers ───────────────────────────────────────────────────────────────────

function randomOrderId() {
  return `ORDER_${Math.random().toString(36).slice(2, 10).toUpperCase()}`;
}

const headers = { 'Content-Type': 'application/json' };

// ── Default scenario: ramp_up + sustained + spike ─────────────────────────────

export default function () {
  // Process payment
  const paymentPayload = JSON.stringify({
    order_id: randomOrderId(),
    amount: Math.floor(Math.random() * 9000) + 1000,
    merchant_address: 'GMERCHANT000000000000000000000000000000000000000000000',
    payer: 'GPAYER000000000000000000000000000000000000000000000000000',
    token: 'GTOKEN000000000000000000000000000000000000000000000000000',
    description: 'Load test payment',
  });

  const start = Date.now();
  const res = http.post(`${BASE_URL}/payments`, paymentPayload, { headers });
  paymentDuration.add(Date.now() - start);

  const ok = check(res, {
    'payment status 200': (r) => r.status === 200 || r.status === 201,
    'payment has id':     (r) => r.json('order_id') !== undefined,
  });
  errorRate.add(!ok);

  sleep(1);

  // Query payment history
  const histRes = http.get(`${BASE_URL}/merchants/GMERCHANT000000000000000000000000000000000000000000000/payments?limit=10`, { headers });
  check(histRes, { 'history status 200': (r) => r.status === 200 });

  sleep(1);
}

// ── Endurance scenario ────────────────────────────────────────────────────────

export function enduranceTest() {
  const res = http.get(`${BASE_URL}/health`, { headers });
  check(res, { 'health ok': (r) => r.status === 200 });
  sleep(2);
}
