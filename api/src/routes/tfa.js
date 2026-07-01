/**
 * 2FA routes (Issue #313)
 *
 * POST /api/auth/2fa/enroll          — begin TOTP enrollment
 * POST /api/auth/2fa/activate        — confirm first TOTP code (enables 2FA)
 * POST /api/auth/2fa/verify/totp     — verify TOTP code
 * POST /api/auth/2fa/verify/sms      — send SMS OTP
 * POST /api/auth/2fa/verify/sms/code — verify SMS OTP code
 * POST /api/auth/2fa/recover         — consume backup code
 * POST /api/auth/2fa/trust-device    — issue device-trust token
 */

import { Router } from "express";
import { jwtMiddleware } from "../auth/jwtMiddleware.js";
import {
  enrollTotp, activateTotp, verifyTotpCode,
  sendSmsOtp, verifySmsOtp,
  consumeBackupCode, trustDevice, TfaError,
} from "../auth/tfa.js";
import { getTotpUri } from "../auth/totp.js";

const router = Router();

// All 2FA routes require a valid JWT
router.use(jwtMiddleware);

router.post("/enroll", (req, res, next) => {
  try {
    const userId = req.user.sub;
    const { secret, backupCodes } = enrollTotp(userId);
    const uri = getTotpUri(secret, `Pulsar:${req.user.address}`);
    res.json({ secret, uri, backupCodes });
  } catch (err) { next(err); }
});

router.post("/activate", (req, res, next) => {
  try {
    activateTotp(req.user.sub, req.body.code);
    res.json({ message: "2FA activated" });
  } catch (err) { handleTfaError(err, res, next); }
});

router.post("/verify/totp", (req, res, next) => {
  try {
    verifyTotpCode(req.user.sub, req.body.code);
    res.json({ verified: true });
  } catch (err) { handleTfaError(err, res, next); }
});

router.post("/verify/sms", async (req, res, next) => {
  try {
    await sendSmsOtp(req.user.sub, req.body.phoneNumber);
    res.json({ sent: true });
  } catch (err) { next(err); }
});

router.post("/verify/sms/code", (req, res, next) => {
  try {
    verifySmsOtp(req.user.sub, req.body.code);
    res.json({ verified: true });
  } catch (err) { handleTfaError(err, res, next); }
});

router.post("/recover", (req, res, next) => {
  try {
    consumeBackupCode(req.user.sub, req.body.backupCode);
    res.json({ verified: true });
  } catch (err) { handleTfaError(err, res, next); }
});

router.post("/trust-device", (req, res, next) => {
  try {
    const token = trustDevice(req.user.sub);
    res.json({ deviceToken: token, expiresIn: 30 * 24 * 60 * 60 });
  } catch (err) { next(err); }
});

function handleTfaError(err, res, next) {
  if (err instanceof TfaError) {
    return res.status(err.status).json({ error: { code: err.code, message: err.message } });
  }
  next(err);
}

export default router;
