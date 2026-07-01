/**
 * Soft-delete service (#308)
 *
 * Wraps the DB client to provide soft-delete and restore helpers.
 * All read queries must filter WHERE deleted_at IS NULL (done in the
 * repository layer by passing `includeDeleted: false` which is the default).
 *
 * Tables supported: merchants | payments | refunds
 */

import { pool } from "../db.js";

const TABLE_PK = {
  merchants: "address",
  payments:  "order_id",
  refunds:   "refund_id",
};

/**
 * Soft-delete a single record.
 * @param {string} table  - one of 'merchants' | 'payments' | 'refunds'
 * @param {string} id     - primary-key value
 * @returns {Promise<boolean>} true if a row was updated
 */
export async function softDelete(table, id) {
  const pk = TABLE_PK[table];
  if (!pk) throw new Error(`Unknown table: ${table}`);

  const { rowCount } = await pool.query(
    `UPDATE ${table} SET deleted_at = NOW() WHERE ${pk} = $1 AND deleted_at IS NULL`,
    [id]
  );
  return rowCount > 0;
}

/**
 * Restore (un-delete) a soft-deleted record.
 * Admin only — caller must enforce authorization before invoking.
 * @param {string} table
 * @param {string} id
 * @returns {Promise<boolean>} true if a row was restored
 */
export async function restore(table, id) {
  const pk = TABLE_PK[table];
  if (!pk) throw new Error(`Unknown table: ${table}`);

  const { rowCount } = await pool.query(
    `UPDATE ${table} SET deleted_at = NULL WHERE ${pk} = $1 AND deleted_at IS NOT NULL`,
    [id]
  );
  return rowCount > 0;
}

/**
 * Cascading soft-delete: delete a merchant and all their payments + refunds.
 * Wrapped in a transaction so it is all-or-nothing.
 * @param {string} merchantAddress
 */
export async function softDeleteMerchantCascade(merchantAddress) {
  const client = await pool.connect();
  try {
    await client.query("BEGIN");

    // 1. Soft-delete refunds that belong to the merchant's payments
    await client.query(
      `UPDATE refunds r
          SET r.deleted_at = NOW()
         FROM payments p
        WHERE r.order_id = p.order_id
          AND p.merchant_address = $1
          AND r.deleted_at IS NULL`,
      [merchantAddress]
    );

    // 2. Soft-delete payments
    await client.query(
      `UPDATE payments SET deleted_at = NOW()
        WHERE merchant_address = $1 AND deleted_at IS NULL`,
      [merchantAddress]
    );

    // 3. Soft-delete merchant
    await client.query(
      `UPDATE merchants SET deleted_at = NOW()
        WHERE address = $1 AND deleted_at IS NULL`,
      [merchantAddress]
    );

    await client.query("COMMIT");
  } catch (err) {
    await client.query("ROLLBACK");
    throw err;
  } finally {
    client.release();
  }
}

/**
 * Build a WHERE clause fragment that excludes soft-deleted rows
 * unless includeDeleted is explicitly true.
 * @param {boolean} includeDeleted
 * @returns {string}
 */
export function notDeletedClause(includeDeleted = false) {
  return includeDeleted ? "" : "AND deleted_at IS NULL";
}
