/**
 * RBAC — roles and permissions (Issue #311)
 *
 * Predefined roles: admin | merchant | customer
 * Custom roles can be added at runtime via createRole().
 */

// ── permission catalogue ──────────────────────────────────────────────────────

export const PERMISSIONS = Object.freeze({
  // payments
  "payments:read:own":  "Read own payment records",
  "payments:read:all":  "Read all payment records",
  "payments:write":     "Create payments",
  // refunds
  "refunds:read:own":   "Read own refund records",
  "refunds:read:all":   "Read all refund records",
  "refunds:write":      "Initiate / approve refunds",
  // merchants
  "merchants:read:own": "Read own merchant profile",
  "merchants:read:all": "Read all merchant profiles",
  "merchants:write":    "Register / update merchants",
  // admin
  "admin:stats":        "Access global stats",
  "admin:config":       "Modify admin config",
  "roles:manage":       "Create / modify custom roles",
});

// ── built-in role definitions ─────────────────────────────────────────────────

/** @type {Map<string, Set<string>>} */
const roleRegistry = new Map([
  ["admin", new Set([
    "payments:read:all", "refunds:read:all", "merchants:read:all",
    "refunds:write", "merchants:write",
    "admin:stats", "admin:config", "roles:manage",
  ])],
  ["merchant", new Set([
    "payments:read:own", "refunds:read:own", "refunds:write",
    "merchants:read:own", "merchants:write",
  ])],
  ["customer", new Set([
    "payments:read:own", "payments:write",
    "refunds:read:own",  "refunds:write",
    "merchants:read:all",
  ])],
]);

// ── role management ───────────────────────────────────────────────────────────

/**
 * Create a custom role.
 * @param {string} name
 * @param {string[]} permissions  subset of keys from PERMISSIONS
 */
export function createRole(name, permissions) {
  if (roleRegistry.has(name)) throw new RbacError(`Role '${name}' already exists`);
  const unknown = permissions.filter((p) => !(p in PERMISSIONS));
  if (unknown.length) throw new RbacError(`Unknown permissions: ${unknown.join(", ")}`);
  roleRegistry.set(name, new Set(permissions));
}

/**
 * Assign additional permissions to an existing role.
 */
export function assignPermissions(role, permissions) {
  const set = roleRegistry.get(role);
  if (!set) throw new RbacError(`Role '${role}' not found`);
  permissions.forEach((p) => {
    if (!(p in PERMISSIONS)) throw new RbacError(`Unknown permission: ${p}`);
    set.add(p);
  });
}

/** Return the permission set for a role (or empty set for unknown). */
export function getPermissions(role) {
  return new Set(roleRegistry.get(role) ?? []);
}

/** Check if a role has a permission. */
export function hasPermission(role, permission) {
  return getPermissions(role).has(permission);
}

export class RbacError extends Error {
  constructor(msg) { super(msg); this.status = 403; this.code = "FORBIDDEN"; }
}
