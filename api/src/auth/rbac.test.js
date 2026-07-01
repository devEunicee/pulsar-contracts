import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  hasPermission, getPermissions, createRole, assignPermissions, RbacError, PERMISSIONS,
} from "./rbac.js";

describe("built-in roles", () => {
  it("admin has all admin permissions", () => {
    assert.ok(hasPermission("admin", "admin:stats"));
    assert.ok(hasPermission("admin", "admin:config"));
    assert.ok(hasPermission("admin", "payments:read:all"));
  });

  it("merchant has merchant-specific permissions only", () => {
    assert.ok( hasPermission("merchant", "payments:read:own"));
    assert.ok(!hasPermission("merchant", "payments:read:all"));
    assert.ok(!hasPermission("merchant", "admin:stats"));
  });

  it("customer has view-only and own-write permissions", () => {
    assert.ok( hasPermission("customer", "payments:read:own"));
    assert.ok( hasPermission("customer", "payments:write"));
    assert.ok(!hasPermission("customer", "payments:read:all"));
    assert.ok(!hasPermission("customer", "admin:config"));
  });
});

describe("createRole", () => {
  it("creates a custom role with given permissions", () => {
    createRole("analyst", ["payments:read:all", "merchants:read:all"]);
    assert.ok(hasPermission("analyst", "payments:read:all"));
    assert.ok(!hasPermission("analyst", "admin:config"));
  });

  it("throws on duplicate role name", () => {
    assert.throws(() => createRole("analyst", []), (e) => e instanceof RbacError);
  });

  it("throws on unknown permissions", () => {
    assert.throws(
      () => createRole("bogus_role", ["not:a:real:permission"]),
      (e) => e instanceof RbacError
    );
  });
});

describe("assignPermissions", () => {
  it("adds permissions to existing role", () => {
    createRole("viewer", ["merchants:read:all"]);
    assignPermissions("viewer", ["payments:read:own"]);
    assert.ok(hasPermission("viewer", "payments:read:own"));
  });

  it("throws on unknown role", () => {
    assert.throws(() => assignPermissions("no_such_role", []), (e) => e instanceof RbacError);
  });
});

describe("getPermissions", () => {
  it("returns empty set for unknown role", () => {
    const perms = getPermissions("ghost");
    assert.equal(perms.size, 0);
  });
});
