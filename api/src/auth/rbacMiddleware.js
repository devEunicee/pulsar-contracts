/**
 * RBAC authorization middleware (Issue #311)
 *
 * Usage:
 *   router.get("/stats", jwtMiddleware, requirePermission("admin:stats"), handler);
 *   router.get("/payments", jwtMiddleware, requireRole("admin", "merchant"), handler);
 */

import { hasPermission, RbacError } from "./rbac.js";
import { auditPermissionCheck } from "./rbacAudit.js";

/**
 * Require the authenticated user to have a specific permission.
 * @param {string} permission
 */
export function requirePermission(permission) {
  return function rbacCheck(req, res, next) {
    const role = req.user?.role;
    const allowed = role ? hasPermission(role, permission) : false;

    auditPermissionCheck({
      userId:     req.user?.sub ?? "anonymous",
      role:       role ?? "none",
      permission,
      resource:   req.path,
      method:     req.method,
      allowed,
    });

    if (!allowed) {
      return res.status(403).json({
        error: { code: "FORBIDDEN", message: `Permission '${permission}' required` },
      });
    }
    next();
  };
}

/**
 * Require the authenticated user to have one of the listed roles.
 * @param {...string} roles
 */
export function requireRole(...roles) {
  return function rbacRoleCheck(req, res, next) {
    const userRole = req.user?.role;
    if (!userRole || !roles.includes(userRole)) {
      return res.status(403).json({
        error: { code: "FORBIDDEN", message: `One of roles [${roles.join(", ")}] required` },
      });
    }
    next();
  };
}
