/**
 * Tests for request cancellation in the Pulsar API client.
 * Run with: node --test api/src/client.test.js
 */
import { describe, it, beforeEach, mock } from "node:test";
import assert from "node:assert/strict";
import { createApiClient, CancelledError, ApiError } from "./client.js";

// ── Helpers ───────────────────────────────────────────────────────────────────

function makeAbortableFetch(resolveWith, { rejectWith } = {}) {
  return async (url, init = {}) => {
    return new Promise((resolve, reject) => {
      if (init.signal?.aborted) {
        const err = new DOMException("Aborted", "AbortError");
        reject(err);
        return;
      }
      const onAbort = () => reject(new DOMException("Aborted", "AbortError"));
      init.signal?.addEventListener("abort", onAbort, { once: true });

      if (rejectWith) {
        reject(rejectWith);
        return;
      }
      resolve({
        ok: true,
        json: async () => resolveWith,
      });
    });
  };
}

const client = createApiClient({ baseUrl: "" });

// ── Tests ─────────────────────────────────────────────────────────────────────

describe("CancelledError", () => {
  it("is an Error", () => {
    const err = new CancelledError();
    assert.ok(err instanceof Error);
    assert.equal(err.name, "CancelledError");
  });
});

describe("ApiError", () => {
  it("carries status and code", () => {
    const err = new ApiError(404, "MerchantNotFound", "Not found");
    assert.equal(err.status, 404);
    assert.equal(err.code, "MerchantNotFound");
    assert.equal(err.message, "Not found");
  });
});

describe("createApiClient", () => {
  beforeEach(() => {
    // Reset globalThis.fetch to a default no-op
    globalThis.fetch = makeAbortableFetch({ ok: true });
  });

  it("cancellable() returns signal and cancel", () => {
    const { signal, cancel } = client.cancellable();
    assert.ok(signal instanceof AbortSignal);
    assert.equal(signal.aborted, false);
    cancel();
    assert.equal(signal.aborted, true);
  });

  it("throws CancelledError when signal is aborted before request", async () => {
    const { signal, cancel } = client.cancellable();
    cancel();
    globalThis.fetch = makeAbortableFetch({});
    await assert.rejects(
      () => client.getMerchant("G123", { signal }),
      (err) => {
        assert.ok(err instanceof CancelledError);
        return true;
      }
    );
  });

  it("throws CancelledError when aborted mid-flight", async () => {
    let doAbort;
    globalThis.fetch = (url, init = {}) =>
      new Promise((_resolve, reject) => {
        doAbort = () => reject(new DOMException("Aborted", "AbortError"));
        init.signal?.addEventListener("abort", doAbort, { once: true });
      });

    const { signal, cancel } = client.cancellable();
    const promise = client.getMerchant("G123", { signal });
    cancel();
    await assert.rejects(promise, (err) => {
      assert.ok(err instanceof CancelledError);
      return true;
    });
  });

  it("resolves with data on successful request", async () => {
    const merchant = { name: "Test Store", active: true };
    globalThis.fetch = makeAbortableFetch(merchant);
    const { signal } = client.cancellable();
    const result = await client.getMerchant("G123", { signal });
    assert.deepEqual(result, merchant);
  });

  it("throws ApiError on HTTP error response", async () => {
    globalThis.fetch = async () => ({
      ok: false,
      status: 404,
      statusText: "Not Found",
      json: async () => ({ error: { code: "MerchantNotFound", message: "Not found" } }),
    });
    await assert.rejects(
      () => client.getMerchant("G_MISSING", {}),
      (err) => {
        assert.ok(err instanceof ApiError);
        assert.equal(err.status, 404);
        assert.equal(err.code, "MerchantNotFound");
        return true;
      }
    );
  });

  it("listPayments builds query string correctly", async () => {
    let capturedUrl;
    globalThis.fetch = async (url) => { capturedUrl = url; return { ok: true, json: async () => [] }; };
    await client.listPayments({ merchant: "G1", limit: 5, status: "Completed" });
    assert.ok(capturedUrl.includes("merchant=G1"));
    assert.ok(capturedUrl.includes("limit=5"));
    assert.ok(capturedUrl.includes("status=Completed"));
  });

  it("listPayments excludes null/undefined params", async () => {
    let capturedUrl;
    globalThis.fetch = async (url) => { capturedUrl = url; return { ok: true, json: async () => [] }; };
    await client.listPayments({ merchant: "G1", cursor: null, date_start: undefined });
    assert.ok(!capturedUrl.includes("cursor"));
    assert.ok(!capturedUrl.includes("date_start"));
  });
});
