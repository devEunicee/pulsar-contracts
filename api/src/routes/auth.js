/**
 * Auth routes (Issue #309)
 *
 * POST /api/auth/login   — validate credentials, issue token pair
 * POST /api/auth/refresh — rotate refresh token
 * POST /api/auth/logout  — revoke refresh token
 */

import { Router } from "express";
import { issueTokens, rotateRefreshToken, revokeRefreshToken, JwtError } from "../auth/jwt.js";

const router = Router();
const PRIVATE_KEY = process.env.JWT_PRIVATE_KEY ?? "";

/**
 * POST /api/auth/login
 * Body: { address: string, signature: string }
 *
 * In production replace the stub below with real signature verification
 * (e.g. verify a Stellar transaction signed by the address).
 */
router.post("/login", (req, res, next) => {
  try {
    const { address, role = "pulsar_customer" } = req.body;
    if (!address) {
      return res.status(422).json({ error: { code: "InvalidInput", message: "address is required" } });
    }
    // TODO: verify Stellar signature before issuing tokens
    const tokens = issueTokens({ userId: address, address, role }, { privateKey: PRIVATE_KEY });
    res.json(tokens);
  } catch (err) {
    next(err);
  }
});

/**
 * POST /api/auth/refresh
 * Body: { refreshToken: string }
 */
router.post("/refresh", (req, res, next) => {
  try {
    const { refreshToken } = req.body;
    if (!refreshToken) {
      return res.status(422).json({ error: { code: "InvalidInput", message: "refreshToken is required" } });
    }
    const tokens = rotateRefreshToken(refreshToken, { privateKey: PRIVATE_KEY });
    res.json(tokens);
  } catch (err) {
    if (err instanceof JwtError) {
      return res.status(401).json({ error: { code: err.code, message: err.message } });
    }
    next(err);
  }
});

/**
 * POST /api/auth/logout
 * Body: { refreshToken: string }
 */
router.post("/logout", (req, res) => {
  const { refreshToken } = req.body ?? {};
  if (refreshToken) revokeRefreshToken(refreshToken);
  res.json({ message: "Logged out" });
});

export default router;
