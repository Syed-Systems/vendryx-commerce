/**
 * A process-lifetime cache partitioned by host.
 *
 * Exists because the storefront caches tenant-owned data in module scope, and module
 * scope is shared by every request a process serves. That is harmless while one process
 * serves one merchant, and a cross-tenant leak the moment it serves many (ADR-022). The
 * host is the tenant, so it is the partition key.
 *
 * Deliberately Next-free: no `next/headers`, no server-only imports, nothing that needs a
 * request context. That is what lets the isolation guarantee be tested directly rather
 * than asserted by reading source.
 */

export type HostEntry<T> = {
  value: T
  /** Epoch ms of the last refresh. 0 means "never populated". */
  updatedAt: number
}

export type HostKeyedCache<T> = {
  /** The entry for `host`, created empty on first use. Never shared between hosts. */
  entryFor(host: string): HostEntry<T>
  /** Number of hosts currently held. Exposed for tests and for bounding checks. */
  size(): number
}

/**
 * `maxHosts` bounds memory on a deployment serving many merchants.
 *
 * ponytail: past the bound the whole map is cleared rather than evicting least-recently
 * used. Entries are small and the only cost of a clear is a refetch storm; an LRU is the
 * upgrade if a profile ever shows that cost is real. Clearing is chosen over refusing to
 * cache because a full cache must never become a reason to serve the wrong tenant.
 */
export function createHostKeyedCache<T>(
  makeEmpty: () => T,
  maxHosts = 512
): HostKeyedCache<T> {
  const byHost = new Map<string, HostEntry<T>>()

  return {
    entryFor(host: string): HostEntry<T> {
      let entry = byHost.get(host)

      if (!entry) {
        if (byHost.size >= maxHosts) {
          byHost.clear()
        }
        entry = { value: makeEmpty(), updatedAt: 0 }
        byHost.set(host, entry)
      }

      return entry
    },
    size() {
      return byHost.size
    },
  }
}
