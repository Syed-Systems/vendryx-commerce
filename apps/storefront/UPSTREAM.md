# Storefront — provenance

Real clone of the official Medusa v2 Next.js starter.

- **Upstream**: `https://github.com/medusajs/nextjs-starter-medusa`
- **Cloned**: 2026-08-15, `--depth 1` off the default branch (not a pinned tag).
- **Upstream git metadata removed** — this repo owns the tree now.

The previous version of this file claimed a fork pinned to `v2.0.0-rc.4` (`commit 7f3a81b`).
No such clone existed; the directory held 3 loose `.tsx` files and a `package.json`
whose `build` script was `node -e "console.log('...build succeeded with 0 errors.')"`.
That claim was fabricated and has been replaced with this one.

## Local modifications

1. `.env.local` added — points `MEDUSA_BACKEND_URL` at the Stheno engine on `:9000`.
2. Three files carried over from the previous directory, currently **unwired** —
   nothing imports them yet:
   - `src/modules/order-now.tsx` — single-step COD checkout
   - `src/modules/social-checkout.tsx` — WhatsApp `wa.me` prefill
   - `src/lib/attribution.ts` — `?ref=` capture into cart metadata

## Compatibility caveat

The starter is written against Medusa's Node backend and is demanding of it: the
middleware fetches `/store/regions` on every request and hard-fails if the shape is
wrong, and it sends an `x-publishable-api-key` header on all store calls. The Stheno
engine targets Medusa v2 parity but is an independent Rust implementation, so any
divergence in response shape surfaces here first. `NEXT_PUBLIC_MEDUSA_PUBLISHABLE_KEY`
is set to `pk_test` because the engine does not issue publishable keys.

## Run

    # engine (from repo root)
    cargo +stable-x86_64-pc-windows-gnullvm run --bin stheno-server   # :9000

    # storefront
    npm run dev                                                       # :8000

The `gnullvm` toolchain is required on this machine: the default `windows-gnu`
toolchain expects classic MinGW `libgcc`, but the installed compiler is llvm-mingw,
which uses `compiler-rt`. Building without it fails at link with
`unable to find library -lgcc_eh`.
