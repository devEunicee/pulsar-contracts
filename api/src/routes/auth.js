/**
 * Password management routes (#314)
 *
 * POST /api/auth/password/reset-request  – request a reset link
 * POST /api/auth/password/reset          – submit new password with token
 * POST /api/auth/password/change         – authenticated password change
 */

import { Router } from "express";
import {
  hashPassword,
  verifyPassword,
  validateStrength,
  isPasswordReused,
  recordPasswordHistory,
  generateResetToken,
  validateResetToken,
  consumeResetToken,
  checkResetRateLimit,
} from "../services/passwordManager.js";
import { pool } from "../db.js";

const router = Router();

/**
 * POST /api/auth/password/reset-request
 * Body: { user_id }
 *
 * Always returns 202 to avoid user enumeration.
 */
router.post("/reset-request", async (req, res, next) => {
  try {
    const { user_id } = req.body;
    if (!user_id) {
      return res.status(422).json({ error: { code: "InvalidInput", message: "user_id required" } });
    }

    const identifier = req.ip || user_id;
    const blocked = await checkResetRateLimit(identifier);
    if (blocked) {
      return res.status(429).json({ error: { code: "RateLimited", message: "Too many reset attempts" } });
    }

    // Only generate a token if the user actually exists (but don't reveal this)
    const { rows } = await pool.query(
      "SELECT user_id FROM user_credentials WHERE user_id = $1",
      [user_id]
    );
    if (rows.length) {
      const token = await generateResetToken(user_id);
      // In production, send `token` via email. Here we just log it.
      console.info(`[password-reset] token for ${user_id}: ${token}`);
    }

    res.status(202).json({ message: "If that account exists, a reset link has been sent." });
  } catch (err) {
    next(err);
  }
});

/**
 * POST /api/auth/password/reset
 * Body: { token, new_password }
 */
router.post("/reset", async (req, res, next) => {
  try {
    const { token, new_password } = req.body;
    if (!token || !new_password) {
      return res.status(422).json({ error: { code: "InvalidInput", message: "token and new_password required" } });
    }

    const { valid, errors } = validateStrength(new_password);
    if (!valid) {
      return res.status(422).json({ error: { code: "WeakPassword", message: errors.join("; ") } });
    }

    const userId = await validateResetToken(token);
    if (!userId) {
      return res.status(400).json({ error: { code: "InvalidToken", message: "Token is invalid or expired" } });
    }

    if (await isPasswordReused(userId, new_password)) {
      return res.status(422).json({ error: { code: "PasswordReused", message: "Cannot reuse a recent password" } });
    }

    const hash = await hashPassword(new_password);
    const client = await pool.connect();
    try {
      await client.query("BEGIN");
      await client.query(
        `UPDATE user_credentials SET password_hash = $1, updated_at = NOW() WHERE user_id = $2`,
        [hash, userId]
      );
      await consumeResetToken(token);
      await recordPasswordHistory(userId, hash);
      // Session invalidation: callers should delete all active sessions for userId.
      await client.query("COMMIT");
    } catch (err) {
      await client.query("ROLLBACK");
      throw err;
    } finally {
      client.release();
    }

    res.json({ message: "Password reset successfully" });
  } catch (err) {
    next(err);
  }
});

/**
 * POST /api/auth/password/change
 * Body: { user_id, current_password, new_password }
 */
router.post("/change", async (req, res, next) => {
  try {
    const { user_id, current_password, new_password } = req.body;
    if (!user_id || !current_password || !new_password) {
      return res.status(422).json({ error: { code: "InvalidInput", message: "user_id, current_password, new_password required" } });
    }

    const { rows } = await pool.query(
      "SELECT password_hash FROM user_credentials WHERE user_id = $1",
      [user_id]
    );
    if (!rows.length || !(await verifyPassword(current_password, rows[0].password_hash))) {
      return res.status(401).json({ error: { code: "Unauthorized", message: "Invalid credentials" } });
    }

    const { valid, errors } = validateStrength(new_password);
    if (!valid) {
      return res.status(422).json({ error: { code: "WeakPassword", message: errors.join("; ") } });
    }

    if (await isPasswordReused(user_id, new_password)) {
      return res.status(422).json({ error: { code: "PasswordReused", message: "Cannot reuse a recent password" } });
    }

    const hash = await hashPassword(new_password);
    await pool.query(
      `UPDATE user_credentials SET password_hash = $1, updated_at = NOW() WHERE user_id = $2`,
      [hash, user_id]
    );
    await recordPasswordHistory(user_id, hash);

    res.json({ message: "Password changed successfully" });
  } catch (err) {
    next(err);
  }
});

export default router;
