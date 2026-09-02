import { expect, test } from "vitest"

import { createHostKeyedCache } from "../../packages/storefront-core/src/lib/util/host-keyed-cache"

// The storefront caches tenant-owned data (regions, and their tenant-owned region ids) in
// module scope, which is shared by every request a process serves. Unkeyed, that hands one
// merchant's data to the next merchant whose request lands on the process — the leak
// ADR-022 makes live by putting many tenants on one process.
//
// These assert the partition itself, so a future refactor cannot quietly drop it.

test("two hosts never share a cache entry", () => {
  const cache = createHostKeyedCache(() => new Map<string, string>())

  cache.entryFor("merchant-a.stheno.shop").value.set("bd", "reg_a")
  const b = cache.entryFor("merchant-b.stheno.shop")

  expect(b.value.has("bd")).toBe(false)
  expect(b.value.size).toBe(0)
  expect(cache.entryFor("merchant-a.stheno.shop").value.get("bd")).toBe("reg_a")
})

test("the same host keeps its entry across calls", () => {
  const cache = createHostKeyedCache(() => new Map<string, string>())

  cache.entryFor("merchant-a.stheno.shop").value.set("bd", "reg_a")

  expect(cache.entryFor("merchant-a.stheno.shop").value.get("bd")).toBe("reg_a")
  expect(cache.size()).toBe(1)
})

test("a fresh entry is marked never-populated so the caller refetches", () => {
  const cache = createHostKeyedCache(() => new Map<string, string>())

  // getRegionMap treats updatedAt as an age; 0 must read as stale, not as "just now".
  expect(cache.entryFor("merchant-a.stheno.shop").updatedAt).toBe(0)
})

test("the host bound is enforced, and clearing never merges tenants", () => {
  const cache = createHostKeyedCache(() => new Map<string, string>(), 2)

  cache.entryFor("a.stheno.shop").value.set("bd", "reg_a")
  cache.entryFor("b.stheno.shop").value.set("bd", "reg_b")
  expect(cache.size()).toBe(2)

  // Third host trips the bound: the map is cleared, not evicted host-by-host.
  const c = cache.entryFor("c.stheno.shop")
  expect(c.value.size).toBe(0)
  expect(cache.size()).toBe(1)

  // The critical property: a evicted then re-requested comes back EMPTY, never carrying
  // b's data. Dropping a cache entry is a refetch; merging two is a leak.
  const a = cache.entryFor("a.stheno.shop")
  expect(a.value.size).toBe(0)
  expect(a.updatedAt).toBe(0)
})

test("hosts that differ only by case or port are the caller's problem, not silently merged", () => {
  const cache = createHostKeyedCache(() => new Map<string, string>())

  // middleware.ts lowercases and strips the port before calling in. This asserts the cache
  // itself does no normalisation, so that responsibility stays in one place rather than
  // being half-done in two.
  cache.entryFor("a.stheno.shop").value.set("bd", "reg_a")

  expect(cache.entryFor("A.stheno.shop").value.size).toBe(0)
  expect(cache.entryFor("a.stheno.shop:8443").value.size).toBe(0)
})
