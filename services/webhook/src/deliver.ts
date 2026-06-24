import https from "https";
import http from "http";
import { URL } from "url";

export interface WebhookPayload {
  event: string;
  contractId: string;
  ledger: number;
  timestamp: number;
  data: Record<string, unknown>;
}

const MAX_ATTEMPTS = 5;
const BASE_DELAY_MS = 1_000;

export async function deliver(url: string, payload: WebhookPayload): Promise<void> {
  const body = JSON.stringify(payload);
  for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt++) {
    try {
      await post(url, body);
      return;
    } catch (err) {
      if (attempt === MAX_ATTEMPTS) {
        console.error(`[webhook] Failed to deliver to ${url} after ${MAX_ATTEMPTS} attempts:`, err);
        return;
      }
      const delay = BASE_DELAY_MS * 2 ** (attempt - 1);
      console.warn(`[webhook] Attempt ${attempt} failed for ${url}. Retrying in ${delay}ms…`);
      await sleep(delay);
    }
  }
}

function post(url: string, body: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const parsed = new URL(url);
    const lib = parsed.protocol === "https:" ? https : http;
    const req = lib.request(
      {
        hostname: parsed.hostname,
        port: parsed.port,
        path: parsed.pathname + parsed.search,
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Content-Length": Buffer.byteLength(body),
          "User-Agent": "pulsar-webhook/0.1",
        },
      },
      (res) => {
        if (res.statusCode && res.statusCode >= 200 && res.statusCode < 300) {
          resolve();
        } else {
          reject(new Error(`HTTP ${res.statusCode}`));
        }
        res.resume();
      }
    );
    req.on("error", reject);
    req.write(body);
    req.end();
  });
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
