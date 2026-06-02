import express from "express";
import { registerWebhook, getWebhook, deleteWebhook } from "./store";

export function createRouter(): express.Router {
  const router = express.Router();
  router.use(express.json());

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
