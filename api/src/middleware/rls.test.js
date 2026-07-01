import assert from "node:assert/strict";
import { describe, it, mock } from "node:test";
import { rlsMiddleware, logAccess } from "./rls.js";

describe("rlsMiddleware", () => {
  it("skips when no req.user", async () => {
    const pool = { connect: mock.fn() };
    const mw = rlsMiddleware(pool);
    const next = mock.fn();
    await mw({ user: null }, {}, next);
    assert.equal(pool.connect.mock.calls.length, 0);
    assert.equal(next.mock.calls.length, 1);
  });

  it("sets app.current_user and app.current_role on the client", async () => {
    const queries = [];
    const client = {
      query: mock.fn((sql, params) => { queries.push({ sql, params }); return Promise.resolve(); }),
      release: mock.fn(),
    };
    const pool = { connect: mock.fn(() => Promise.resolve(client)) };
    const mw = rlsMiddleware(pool);
    const next = mock.fn();
    const res = { on: mock.fn() };
    const req = { user: { address: "GABC", role: "pulsar_merchant" } };

    await mw(req, res, next);

    assert.equal(queries[0].params[0], "GABC");
    assert.equal(queries[1].params[0], "pulsar_merchant");
    assert.equal(next.mock.calls.length, 1);
    assert.equal(req.dbClient, client);
  });

  it("releases client on response finish", async () => {
    let finishCb;
    const client = {
      query: mock.fn(() => Promise.resolve()),
      release: mock.fn(),
    };
    const pool = { connect: mock.fn(() => Promise.resolve(client)) };
    const mw = rlsMiddleware(pool);
    const res = { on: mock.fn((event, cb) => { if (event === "finish") finishCb = cb; }) };
    const req = { user: { address: "GABC", role: "pulsar_admin" } };
    await mw(req, res, () => {});

    finishCb();
    assert.equal(client.release.mock.calls.length, 1);
    assert.equal(req.dbClient, null);
  });
});

describe("logAccess", () => {
  it("inserts a row with correct fields", async () => {
    const client = { query: mock.fn(() => Promise.resolve()) };
    await logAccess(client, {
      table: "payments", operation: "SELECT",
      address: "GABC", role: "pulsar_customer", allowed: true,
    });
    const [sql, params] = client.query.mock.calls[0].arguments;
    assert.match(sql, /INSERT INTO rls_access_log/);
    assert.equal(params[0], "payments");
    assert.equal(params[1], "SELECT");
    assert.equal(params[2], "GABC");
    assert.equal(params[3], "pulsar_customer");
    assert.equal(params[5], true);
  });
});
