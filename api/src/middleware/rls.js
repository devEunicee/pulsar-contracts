/**
 * Row-Level Security middleware (Issue #306)
 *
 * Sets `app.current_user` on every DB connection so that PostgreSQL RLS
 * policies can filter rows by the authenticated caller's address.
 *
 * Usage:
 *   app.use(rlsMiddleware(pool));
 *
 * The middleware expects `req.user` to be set by the auth middleware that
 * runs before it (e.g., JWT middleware from issue #309).
 */

/**
 * @param {import('pg').Pool} pool  - shared pg connection pool
 */
export function rlsMiddleware(pool) {
  return async function setRlsContext(req, res, next) {
    const user = req.user;
    if (!user) return next();

    // Acquire a dedicated client for this request so the SET is scoped to it.
    req.dbClient = await pool.connect();
    try {
      await req.dbClient.query(
        "SELECT set_config('app.current_user', $1, true)",
        [user.address]
      );
      await req.dbClient.query(
        "SELECT set_config('app.current_role', $1, true)",
        [user.role]
      );
    } catch (err) {
      req.dbClient.release();
      req.dbClient = null;
      return next(err);
    }

    // Release client after response is finished
    res.on("finish", () => {
      if (req.dbClient) {
        req.dbClient.release();
        req.dbClient = null;
      }
    });

    next();
  };
}

/**
 * Log an access attempt to rls_access_log.
 *
 * @param {import('pg').PoolClient} client
 * @param {{ table: string, operation: string, address: string, role: string, allowed: boolean }} entry
 */
export async function logAccess(client, { table, operation, address, role, allowed }) {
  await client.query(
    `INSERT INTO rls_access_log (table_name, operation, user_address, user_role, accessed_at, allowed)
     VALUES ($1, $2, $3, $4, $5, $6)`,
    [table, operation, address, role, Math.floor(Date.now() / 1000), allowed]
  );
}
