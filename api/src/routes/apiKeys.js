/**
 * apiKeys.js — REST routes for API Key Management (#315).
 *
 * All routes require the caller's Stellar address via the X-Owner header
 * (in production this would be replaced by a proper JWT/session; the
 * X-Owner header is a placeholder that mirrors how the JWT dependency
 * (#309) will eventually authenticate the management plane).
 *
 * Endpoints:
 *   POST   /api/keys                        Create a new API key
 *   GET    /api/keys                        List keys for authenticated owner
 *   GET    /api/keys/:id                    Get a single key
 *   PUT    /api/keys/:id                    Update name / scopes / rate_limit / expires_at
 *   POST   /api/keys/:id/rotate             Rotate (revoke + re-issue)
 *   DELETE /api/keys/:id                    Revoke a key
 *   GET    /api/keys/:id/activity           Key activity log
 *   GET    /api/keys/scopes                 List valid scopes
 */

import { Router } from "express";
import { ApiKeyService, VALID_SCOPES } from "../services/apiKeys.js";

const router = Router();
const service = new ApiKeyService();

// ── Owner resolution ──────────────────────────────────────────────────────────

/**
 * Extract the caller's Stellar address from the X-Owner header.
 * Returns a 401 response if the header is missing.
 */
function getOwner(req, res) {
  const owner = req.headers["x-owner"];
  if (!owner) {
    res.status(401).json({
      error: {
        code: "MissingOwner",
        message:
          "X-Owner header with a valid Stellar address is required. " +
          "This will be replaced by JWT authentication (#309).",
      },
    });
    return null;
  }
  return owner;
}

// ── Routes ────────────────────────────────────────────────────────────────────

/**
 * GET /api/keys/scopes
 * Returns the list of valid permission scopes.
 */
router.get("/scopes", (_req, res) => {
  res.json({ scopes: VALID_SCOPES });
});

/**
 * POST /api/keys
 * Body: { name, scopes: string[], rate_limit?, expires_at? }
 * Returns: { key: ApiKeyRecord, plaintext: string }
 *
 * The plaintext key is returned ONCE. Store it securely — it cannot be
 * retrieved again.
 */
router.post("/", async (req, res, next) => {
  try {
    const owner = getOwner(req, res);
    if (!owner) return;

    const { name, scopes, rate_limit, expires_at } = req.body;

    if (!name || typeof name !== "string") {
      return res.status(422).json({
        error: { code: "InvalidInput", message: "name is required" },
      });
    }
    if (!Array.isArray(scopes) || scopes.length === 0) {
      return res.status(422).json({
        error: { code: "InvalidInput", message: "scopes must be a non-empty array" },
      });
    }

    const result = await service.create({
      owner,
      name,
      scopes,
      rateLimit: rate_limit ?? 1000,
      expiresAt: expires_at ? new Date(expires_at) : null,
    });

    res.status(201).json(result);
  } catch (err) {
    if (err.message.startsWith("Unknown scopes")) {
      return res.status(422).json({ error: { code: "InvalidScope", message: err.message } });
    }
    next(err);
  }
});

/**
 * GET /api/keys
 * Returns all keys for the authenticated owner (hashes masked).
 */
router.get("/", async (req, res, next) => {
  try {
    const owner = getOwner(req, res);
    if (!owner) return;
    const keys = await service.listByOwner(owner);
    res.json({ keys });
  } catch (err) {
    next(err);
  }
});

/**
 * GET /api/keys/:id
 * Returns a single key for the authenticated owner.
 */
router.get("/:id", async (req, res, next) => {
  try {
    const owner = getOwner(req, res);
    if (!owner) return;
    const key = await service.getById(req.params.id, owner);
    if (!key) {
      return res.status(404).json({
        error: { code: "NotFound", message: "API key not found" },
      });
    }
    res.json({ key });
  } catch (err) {
    next(err);
  }
});

/**
 * PUT /api/keys/:id
 * Body: { name?, scopes?, rate_limit?, expires_at? }
 * Returns the updated key record.
 */
router.put("/:id", async (req, res, next) => {
  try {
    const owner = getOwner(req, res);
    if (!owner) return;

    const { name, scopes, rate_limit, expires_at } = req.body;
    const updates = {};
    if (name !== undefined) updates.name = name;
    if (scopes !== undefined) updates.scopes = scopes;
    if (rate_limit !== undefined) updates.rateLimit = rate_limit;
    if (expires_at !== undefined) {
      updates.expiresAt = expires_at ? new Date(expires_at) : null;
    }

    const key = await service.update(req.params.id, owner, updates);
    res.json({ key });
  } catch (err) {
    if (err.message === "No fields to update") {
      return res.status(422).json({ error: { code: "InvalidInput", message: err.message } });
    }
    if (err.message.startsWith("Unknown scopes")) {
      return res.status(422).json({ error: { code: "InvalidScope", message: err.message } });
    }
    if (err.message.includes("not found")) {
      return res.status(404).json({ error: { code: "NotFound", message: err.message } });
    }
    next(err);
  }
});

/**
 * POST /api/keys/:id/rotate
 * Atomically revokes the current key and issues a replacement with the same
 * name, scopes, rate limit, and expiry.
 *
 * Returns: { key: ApiKeyRecord, plaintext: string }
 * The plaintext is returned ONCE — store it securely.
 */
router.post("/:id/rotate", async (req, res, next) => {
  try {
    const owner = getOwner(req, res);
    if (!owner) return;
    const result = await service.rotate(req.params.id, owner);
    res.status(201).json(result);
  } catch (err) {
    if (err.message.includes("not found")) {
      return res.status(404).json({ error: { code: "NotFound", message: err.message } });
    }
    next(err);
  }
});

/**
 * DELETE /api/keys/:id
 * Immediately revokes the key. Returns 204 on success.
 */
router.delete("/:id", async (req, res, next) => {
  try {
    const owner = getOwner(req, res);
    if (!owner) return;
    await service.revoke(req.params.id, owner);
    res.status(204).end();
  } catch (err) {
    if (err.message.includes("not found")) {
      return res.status(404).json({ error: { code: "NotFound", message: err.message } });
    }
    next(err);
  }
});

/**
 * GET /api/keys/:id/activity
 * Query: { limit? }
 * Returns recent activity log entries for the key.
 */
router.get("/:id/activity", async (req, res, next) => {
  try {
    const owner = getOwner(req, res);
    if (!owner) return;
    const limit = Math.min(parseInt(req.query.limit ?? "50", 10), 500);
    const activity = await service.getActivity(req.params.id, owner, limit);
    res.json({ activity });
  } catch (err) {
    if (err.message.includes("not found")) {
      return res.status(404).json({ error: { code: "NotFound", message: err.message } });
    }
    next(err);
  }
});

export default router;
