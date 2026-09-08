# Marketing Content Audit — Stheno

**Date:** 2026-09-01  
**Scope:** the marketing app, product specifications, entitlement definitions, security and payment
documentation, and the currently configured public-domain target.

## Verdict

The previous marketing homepage mixed documented product direction with unverified commercial,
customer, integration, and performance claims. It was not safe to use as the public source of
truth for plans or legal commitments.

The rewrite keeps the product-specific value proposition, but separates:

- documented capabilities and plan entitlements;
- release-gated targets and future integrations;
- commercial terms that require a service order; and
- legal terms that require facts absent from this repository.

## Source-of-truth claims

| Audience | Supported statement | Evidence |
|---|---|---|
| Store owners | Stheno is designed for catalogue, inventory, guest checkout, social-order handoff, order handling, notifications, and basic reporting. | PRODUCT.md, PRD.md |
| Ecommerce builders | Trial and Free use a supported brand kit and subdomain; Pro includes a developer-built bespoke storefront and custom domain after payment. | ENTITLEMENTS.md, PRODUCT.md, DECISIONS.md ADR-013 |
| Developers | Stheno is an open-source Rust/Postgres commerce engine with Medusa v2 Store/Admin compatibility targets, durable workflows, and exact decimal money handling. | PRD.md, API_SPEC.md, WORKFLOWS.md, PAYMENTS.md |
| Plan terms | Trial is 14 days after a verified store request; Free is indefinite; Pro is a 6- or 12-month commitment. | ENTITLEMENTS.md |
| Data handling | The documented design includes data inventory, pseudonymisation-based erasure, defined retention periods, payment-token boundaries, tenant isolation, and audit logging. | SECURITY.md, PAYMENTS.md |

## Removed or no longer rendered

| Claim type | Why it is not publishable from the current evidence |
|---|---|
| “Saves 2+ hours daily,” annual savings, order velocity, inventory accuracy, and response-time figures | No attributable measurement, study, or customer evidence in the repository. |
| “Verified merchant” testimonials and five-star ratings | The named brands and testimonials are demo copy, not verified customer evidence. |
| Named payment gateways and couriers, nationwide delivery, and automatic labels | Plan specifications gate payment/courier capability but do not prove those named providers or coverage are available. |
| 0% transaction fees, competitor 2% fees, price points, annual discounts, and ROI calculator | The entitlement specification defines capabilities and commitments, not public prices, transaction-fee policy, competitor pricing, or annual savings. |
| “Measured” 32 ms TTFB, sub-8 ms saga commits, 18 MB heap, and global-edge claims | Product documentation gives release targets and explicitly requires measured baselines before performance claims. |
| A generated Stheno SDK or published live API credentials | The API specification says both frontends use Medusa's SDK for 1.0; the old snippet was not a verified integration. |

## SEO audit — source level

| Area | Finding | Status |
|---|---|---|
| Domain | https://stheno.shop, robots.txt, and sitemap.xml did not resolve during this audit. | Blocking deployment issue |
| Indexability | Vite SPA routes rely on client-side rendering. Page-specific SEO metadata and legal-route directives are not available in the initial HTML. | High priority architecture issue |
| Metadata | The original title, description, and schema included unsupported price/offers data. | Replaced |
| Structured data | SoftwareApplication, Organization, and WebSite JSON-LD are now limited to verifiable values. | Improved |
| Sitemap | No marketing sitemap or robots file existed. | Added |
| Content quality | Homepage content was merchant-only, short on builder/developer intent, and included unsupported proof points. | Features and plan pages added |
| Legal trust signals | No privacy or terms routes existed. | Product-specific publication drafts added; legal facts still required |

## Publication blockers

The following information must be supplied and approved before the Privacy notice or Terms of
Service can be called final or placed in a crawlable production release:

1. Contracting legal entity, registered address, company registration and tax/VAT identifiers.
2. Privacy contact, DPO/representative where required, and a working rights-request channel.
3. Governing law, venue, dispute process, consumer-law approach, and age policy.
4. Current price list, currency, taxes, invoicing, payment schedules, renewal, suspension, and
   refund terms.
5. Actual hosting regions, subprocessors, analytics/cookie tooling, email provider, payment
   providers, couriers, and international-transfer safeguards.
6. The current operational status of every feature and provider promoted publicly.

## Implementation follow-up

- Move the marketing site to static generation or server rendering before relying on the new routes
  for organic search. Vite's SPA fallback is usable by people but leaves page-level metadata and
  structured data client-rendered.
- Verify the production domain, TLS, redirects, robots.txt, sitemap URLs, page titles, canonical
  tags, security headers, and mobile rendering after deployment.
- Add real, consented customer evidence only with a named source and approval date.
- Maintain a release-claims register: every number, compatibility statement, provider name, and
  plan feature should record evidence and a review date.
