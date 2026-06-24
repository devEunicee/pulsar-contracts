import express from "express";
import { registerWebhook, getWebhook, deleteWebhook } from "./store";
import { checkDependencies, ComponentStatus } from "./health";

export function createRouter(rpcUrl: string, contractId: string): express.Router {
  const router = express.Router();
  router.use(express.json());

  const metrics = {
    totalHealthCalls: 0,
    totalDeepCalls: 0,
    totalReadinessCalls: 0,
    totalLivenessCalls: 0,
    totalMetricsCalls: 0,
    averageResponseMs: 0,
    lastResponseMs: 0,
  };

  function recordMetrics(route: keyof Omit<typeof metrics, "averageResponseMs" | "lastResponseMs">, durationMs: number) {
    metrics[route] += 1;
    metrics.lastResponseMs = durationMs;
    metrics.averageResponseMs = Math.round((metrics.averageResponseMs * (metrics.totalHealthCalls + metrics.totalDeepCalls + metrics.totalReadinessCalls + metrics.totalLivenessCalls + metrics.totalMetricsCalls - 1) + durationMs) / (metrics.totalHealthCalls + metrics.totalDeepCalls + metrics.totalReadinessCalls + metrics.totalLivenessCalls + metrics.totalMetricsCalls));
  }

  function buildComponentSummary(components: Record<string, ComponentStatus>) {
    return Object.entries(components).reduce((acc, [key, value]) => {
      acc[key] = {
        status: value.status,
        durationMs: value.durationMs,
        details: value.details,
        error: value.error,
      };
      return acc;
    }, {} as Record<string, unknown>);
  }

  router.get("/health", (_req, res) => {
    const start = Date.now();
    const payload = {
      status: "ok",
      timestamp: new Date().toISOString(),
      uptimeSeconds: Math.floor(process.uptime()),
      dependencies: {
        database: "unverified",
        rpc: "unverified",
        contract: "unverified",
      },
    };
    const durationMs = Date.now() - start;
    recordMetrics("totalHealthCalls", durationMs);
    res.set("X-Health-Duration-Ms", durationMs.toString()).json(payload);
  });

  router.get("/health/deep", async (_req, res) => {
    const start = Date.now();
    const components = await checkDependencies(contractId, rpcUrl);
    const status = components.database.status === "ok" && components.rpc.status === "ok" && components.contract.status === "ok" ? "ok" : "unhealthy";
    const payload = {
      status,
      timestamp: new Date().toISOString(),
      components: buildComponentSummary(components),
    };
    const durationMs = Date.now() - start;
    recordMetrics("totalDeepCalls", durationMs);
    res.status(status === "ok" ? 200 : 503).set("X-Health-Duration-Ms", durationMs.toString()).json(payload);
  });

  router.get("/health/liveness", (_req, res) => {
    const start = Date.now();
    const payload = { status: "alive" };
    const durationMs = Date.now() - start;
    recordMetrics("totalLivenessCalls", durationMs);
    res.status(200).set("X-Health-Duration-Ms", durationMs.toString()).json(payload);
  });

  router.get("/health/readiness", async (_req, res) => {
    const start = Date.now();
    const components = await checkDependencies(contractId, rpcUrl);
    const ready = components.database.status === "ok" && components.rpc.status === "ok" && components.contract.status === "ok";
    const payload = {
      status: ready ? "ready" : "not ready",
      timestamp: new Date().toISOString(),
      components: buildComponentSummary(components),
    };
    const durationMs = Date.now() - start;
    recordMetrics("totalReadinessCalls", durationMs);
    res.status(ready ? 200 : 503).set("X-Health-Duration-Ms", durationMs.toString()).json(payload);
  });

  router.get("/health/metrics", (_req, res) => {
    const start = Date.now();
    const payload = {
      metrics,
      generatedAt: new Date().toISOString(),
    };
    const durationMs = Date.now() - start;
    recordMetrics("totalMetricsCalls", durationMs);
    res.status(200).set("X-Health-Duration-Ms", durationMs.toString()).json(payload);
  });

  // POST /webhooks — register or update a webhook URL for a merchant
  router.post("/webhooks", (req, res) => {
    const { merchantAddress, url } = req.body as { merchantAddress?: string; url?: string };
    if (!merchantAddress || !url) {
      res.status(400).json({ error: "merchantAddress and url are required" });
      return;
    }
    try {
      new URL(url);
    } catch {
      res.status(400).json({ error: "url is not a valid URL" });
      return;
    }
    const reg = registerWebhook(merchantAddress, url);
    res.status(201).json(reg);
  });

  // GET /webhooks/:merchantAddress — retrieve registration
  router.get("/webhooks/:merchantAddress", (req, res) => {
    const reg = getWebhook(req.params.merchantAddress);
    if (!reg) {
      res.status(404).json({ error: "Not found" });
      return;
    }
    res.json(reg);
  });

  // DELETE /webhooks/:merchantAddress — remove registration
  router.delete("/webhooks/:merchantAddress", (req, res) => {
    const deleted = deleteWebhook(req.params.merchantAddress);
    res.status(deleted ? 204 : 404).end();
  });

  return router;
}
