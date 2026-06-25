#!/usr/bin/env node
// migrate.js — simple SQL migration runner
// Usage:
//   node migrate.js up          Apply all pending migrations
//   node migrate.js down        Roll back the last applied migration
//   node migrate.js status      Show applied/pending migrations
//   node migrate.js dry-run     Show pending migrations without applying

"use strict";

const fs = require("fs");
const path = require("path");
const { Client } = require("pg");

const MIGRATIONS_DIR = path.join(__dirname, "migrations");
const LOCK_CHANNEL = "migration_lock";

const db = new Client({ connectionString: process.env.DATABASE_URL });

async function ensureHistoryTable() {
  await db.query(`
    CREATE TABLE IF NOT EXISTS migration_history (
      id         SERIAL PRIMARY KEY,
      name       TEXT NOT NULL UNIQUE,
      applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
    )
  `);
}

async function acquireLock() {
  const { rows } = await db.query(
    "SELECT pg_try_advisory_lock(hashtext($1))", [LOCK_CHANNEL]
  );
  if (!rows[0].pg_try_advisory_lock) {
    throw new Error("Another migration process is running. Aborting.");
  }
}

async function releaseLock() {
  await db.query("SELECT pg_advisory_unlock(hashtext($1))", [LOCK_CHANNEL]);
}

function loadMigrationFiles() {
  return fs
    .readdirSync(MIGRATIONS_DIR)
    .filter((f) => f.endsWith(".sql"))
    .sort();
}

async function getApplied() {
  const { rows } = await db.query("SELECT name FROM migration_history ORDER BY id");
  return new Set(rows.map((r) => r.name));
}

async function applyMigration(file) {
  const sql = fs.readFileSync(path.join(MIGRATIONS_DIR, file), "utf8");
  const upSection = sql.split("-- DOWN")[0].replace("-- UP", "").trim();
  await db.query("BEGIN");
  try {
    await db.query(upSection);
    await db.query("INSERT INTO migration_history (name) VALUES ($1)", [file]);
    await db.query("COMMIT");
    console.log(`  ✓ applied ${file}`);
  } catch (err) {
    await db.query("ROLLBACK");
    throw err;
  }
}

async function rollbackMigration(file) {
  const sql = fs.readFileSync(path.join(MIGRATIONS_DIR, file), "utf8");
  const parts = sql.split("-- DOWN");
  if (parts.length < 2 || !parts[1].trim()) {
    throw new Error(`No DOWN section in ${file}`);
  }
  const downSection = parts[1].trim();
  await db.query("BEGIN");
  try {
    await db.query(downSection);
    await db.query("DELETE FROM migration_history WHERE name = $1", [file]);
    await db.query("COMMIT");
    console.log(`  ✓ rolled back ${file}`);
  } catch (err) {
    await db.query("ROLLBACK");
    throw err;
  }
}

async function cmdUp() {
  const files = loadMigrationFiles();
  const applied = await getApplied();
  const pending = files.filter((f) => !applied.has(f));
  if (!pending.length) { console.log("Nothing to apply."); return; }
  console.log(`Applying ${pending.length} migration(s)...`);
  for (const f of pending) await applyMigration(f);
}

async function cmdDown() {
  const { rows } = await db.query(
    "SELECT name FROM migration_history ORDER BY id DESC LIMIT 1"
  );
  if (!rows.length) { console.log("No migrations to roll back."); return; }
  console.log(`Rolling back ${rows[0].name}...`);
  await rollbackMigration(rows[0].name);
}

async function cmdStatus() {
  const files = loadMigrationFiles();
  const applied = await getApplied();
  console.log("\nMigration status:");
  for (const f of files) {
    console.log(`  [${applied.has(f) ? "✓" : " "}] ${f}`);
  }
}

async function cmdDryRun() {
  const files = loadMigrationFiles();
  const applied = await getApplied();
  const pending = files.filter((f) => !applied.has(f));
  if (!pending.length) { console.log("No pending migrations."); return; }
  console.log("Pending migrations (dry-run, not applied):");
  pending.forEach((f) => console.log(`  - ${f}`));
}

async function main() {
  const cmd = process.argv[2];
  if (!["up", "down", "status", "dry-run"].includes(cmd)) {
    console.error("Usage: node migrate.js <up|down|status|dry-run>");
    process.exit(1);
  }
  await db.connect();
  await ensureHistoryTable();
  if (cmd !== "dry-run" && cmd !== "status") await acquireLock();
  try {
    if (cmd === "up")       await cmdUp();
    if (cmd === "down")     await cmdDown();
    if (cmd === "status")   await cmdStatus();
    if (cmd === "dry-run")  await cmdDryRun();
  } finally {
    if (cmd !== "dry-run" && cmd !== "status") await releaseLock();
    await db.end();
  }
}

main().catch((err) => { console.error(err.message); process.exit(1); });
