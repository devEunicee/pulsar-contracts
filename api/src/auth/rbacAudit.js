/**
 * RBAC audit log (Issue #311)
 *
 * Logs every permission check to stdout (structured JSON) so it can be
 * ingested by any log aggregator. In production, emit to a DB or SIEM.
 */

/**
 * @param {{ userId: string, role: string, permission: string, resource: string, method: string, allowed: boolean }} entry
 */
export function auditPermissionCheck(entry) {
  const record = {
    event:      "permission_check",
    ts:         new Date().toISOString(),
    userId:     entry.userId,
    role:       entry.role,
    permission: entry.permission,
    resource:   entry.resource,
    method:     entry.method,
    allowed:    entry.allowed,
  };
  // eslint-disable-next-line no-console
  console.log(JSON.stringify(record));
}

/**
 * Log a role or permission change.
 */
export function auditRoleChange({ changedBy, action, target, permissions }) {
  console.log(JSON.stringify({
    event:       "role_change",
    ts:          new Date().toISOString(),
    changedBy,
    action,      // "create_role" | "assign_permissions"
    target,      // role name
    permissions,
  }));
}
