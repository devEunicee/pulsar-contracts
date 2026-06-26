import express from "express";
import { Pool } from "pg";

const REPORT_TABLES: Record<string, string> = {
  payments: "payments",
  refunds: "refunds",
};

export function createAnalyticsRouter(databaseUrl: string): express.Router {
  const router = express.Router();
  const db = new Pool({ connectionString: databaseUrl });

  router.get("/analytics/summary", async (req, res) => {
    try {
      const { rows: paymentRows } = await db.query(
        "SELECT COUNT(*)::int AS payment_count, COALESCE(SUM(amount), 0)::numeric AS total_volume, COALESCE(AVG(amount), 0)::numeric AS avg_transaction_value FROM payments"
      );
      const { rows: refundRows } = await db.query(
        "SELECT COUNT(*)::int AS refund_count, COALESCE(SUM(amount), 0)::numeric AS refund_volume FROM refunds"
      );
      const { rows: topMerchants } = await db.query(
        `SELECT merchant_id, COALESCE(SUM(amount), 0)::numeric AS volume
           FROM payments
          GROUP BY merchant_id
          ORDER BY volume DESC
          LIMIT 10`
      );

      const payments = paymentRows[0];
      const refunds = refundRows[0];
      const refundRate = payments.payment_count === 0
        ? 0
        : Number(refunds.refund_count) / Number(payments.payment_count);

      res.json({
        payment_count: payments.payment_count,
        total_payment_volume: payments.total_volume,
        average_transaction_value: payments.avg_transaction_value,
        refund_count: refunds.refund_count,
        total_refund_volume: refunds.refund_volume,
        refund_rate: refundRate,
        top_merchants_by_volume: topMerchants,
      });
    } catch (err) {
      console.error(err);
      res.status(500).json({ error: "Failed to compute analytics summary" });
    }
  });

  router.get("/analytics/trends", async (req, res) => {
    try {
      const { startDate, endDate } = req.query;
      const start = startDate ? String(startDate) : "2024-01-01";
      const end = endDate ? String(endDate) : new Date().toISOString().slice(0, 10);
      const { rows } = await db.query(
        `SELECT date_trunc('day', created_at)::date AS day,
                COUNT(*)::int AS payment_count,
                COALESCE(SUM(amount),0)::numeric AS payment_volume
           FROM payments
          WHERE created_at BETWEEN $1::date AND $2::date + INTERVAL '1 day'
          GROUP BY 1
          ORDER BY 1`,
        [start, end]
      );
      res.json({ trends: rows });
    } catch (err) {
      console.error(err);
      res.status(500).json({ error: "Failed to compute trends" });
    }
  });

  router.get("/analytics/segments", async (req, res) => {
    try {
      const { startDate, endDate } = req.query;
      const start = startDate ? String(startDate) : "2024-01-01";
      const end = endDate ? String(endDate) : new Date().toISOString().slice(0, 10);
      const { rows } = await db.query(
        `WITH customer_totals AS (
           SELECT customer_id,
                  COALESCE(SUM(amount),0)::numeric AS total_spend,
                  COUNT(*)::int AS payment_count
             FROM payments
            WHERE created_at BETWEEN $1::date AND $2::date + INTERVAL '1 day'
            GROUP BY customer_id
         )
         SELECT CASE
                  WHEN total_spend < 100 THEN 'low'
                  WHEN total_spend < 1000 THEN 'medium'
                  ELSE 'high'
                END AS segment,
                COUNT(*)::int AS customer_count,
                COALESCE(SUM(total_spend),0)::numeric AS total_segment_spend,
                COALESCE(AVG(payment_count),0)::numeric AS avg_payments_per_customer
           FROM customer_totals
          GROUP BY segment
          ORDER BY segment`,
        [start, end]
      );
      res.json({ segments: rows });
    } catch (err) {
      console.error(err);
      res.status(500).json({ error: "Failed to compute customer segments" });
    }
  });

  router.get("/analytics/custom-report", async (req, res) => {
    try {
      const { metric = "payments", merchant_id, startDate, endDate } = req.query;
      const start = startDate ? String(startDate) : "2024-01-01";
      const end = endDate ? String(endDate) : new Date().toISOString().slice(0, 10);
      const table = REPORT_TABLES[String(metric)] ?? "payments";
      let query = `SELECT date_trunc('day', created_at)::date AS day,
                          COUNT(*)::int AS count,
                          COALESCE(SUM(amount),0)::numeric AS volume
                     FROM ${table}
                    WHERE created_at BETWEEN $1::date AND $2::date + INTERVAL '1 day'`;
      const params: Array<string> = [start, end];

      if (merchant_id) {
        query += ` AND merchant_id = $3`;
        params.push(String(merchant_id));
      }

      query += ` GROUP BY 1 ORDER BY 1`;
      const { rows } = await db.query(query, params);
      res.json({ report: rows });
    } catch (err) {
      console.error(err);
      res.status(500).json({ error: "Failed to generate custom report" });
    }
  });

  return router;
}
