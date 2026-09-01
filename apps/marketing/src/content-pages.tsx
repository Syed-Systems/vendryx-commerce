const planRows = [
  ["Trial", "14 days after an email-verified merchant submits a store request", "Full core operations, unlimited products, up to 10 staff seats, supported brand kit and subdomain"],
  ["Free", "Indefinite after the trial", "Core operations, up to 100 new products, owner plus two staff seats, a supported brand kit and subdomain, and a 30-day analytics window"],
  ["Pro", "6- or 12-month commitment", "Unlimited products, up to 10 staff seats, full analytics history, payment gateway, courier dispatch, fraud/risk scoring, custom domain, and bespoke storefront service after payment"],
] as const

const dataProcessingRows = [
  ["Merchant account and store request details", "Authentication, launch review, provisioning, support, and plan administration.", "Until erasure or the documented retention period applies."],
  ["Buyer and order data", "Guest checkout, fulfilment, customer service, fraud correlation, and merchant operations.", "Order and financial records: 10 years; buyer profiles: until erasure or 3 years after the last order."],
  ["Cart, notification, workflow, and operational data", "Operate checkout, notify staff, recover workflows, secure the service, and investigate incidents.", "Incomplete carts: 30/90 days; workflow data: 30 days, or 90 days for order flows; logs/traces: 30 days."],
  ["Payment provider data", "Process payment status and reconcile transactions without receiving card numbers or CVV.", "Provider webhook events: 90 days; payment-provider request records: 30 days."],
] as const

function ContentHeader({ eyebrow, title, intro }: { eyebrow: string; title: string; intro: string }) {
  return (
    <header className="content-page-header section-shell">
      <nav className="breadcrumbs-nav" aria-label="Breadcrumb navigation">
        <ol className="breadcrumbs-list" itemScope itemType="https://schema.org/BreadcrumbList">
          <li itemProp="itemListElement" itemScope itemType="https://schema.org/ListItem">
            <a href="/" itemProp="item">
              <span itemProp="name">Home</span>
            </a>
            <meta itemProp="position" content="1" />
          </li>
          <li itemProp="itemListElement" itemScope itemType="https://schema.org/ListItem">
            <span className="breadcrumb-separator" aria-hidden="true">/</span>
            <span className="breadcrumb-current" aria-current="page" itemProp="name">{eyebrow}</span>
            <meta itemProp="position" content="2" />
          </li>
        </ol>
      </nav>
      <p className="eyebrow">{eyebrow}</p>
      <h1>{title}</h1>
      <p>{intro}</p>
    </header>
  )
}

function PlanTable() {
  return (
    <div className="plan-table-wrap">
      <table className="plan-table plan-table-plans">
        <colgroup><col className="plan-table-name" /><col className="plan-table-term" /><col className="plan-table-included" /></colgroup>
        <thead><tr><th scope="col">Plan</th><th scope="col">Term</th><th scope="col">Included</th></tr></thead>
        <tbody>{planRows.map(([name, term, included]) => <tr key={name}><th scope="row" data-label="Plan">{name}</th><td data-label="Term">{term}</td><td data-label="Included">{included}</td></tr>)}</tbody>
      </table>
    </div>
  )
}

function DataProcessingTable() {
  return (
    <div className="plan-table-wrap">
      <table className="plan-table plan-table-data">
        <colgroup><col className="plan-table-data-type" /><col className="plan-table-data-purpose" /><col className="plan-table-data-retention" /></colgroup>
        <thead><tr><th scope="col">Data</th><th scope="col">Why it is used</th><th scope="col">Designed retention</th></tr></thead>
        <tbody>{dataProcessingRows.map(([data, purpose, retention]) => <tr key={data}><th scope="row" data-label="Data">{data}</th><td data-label="Why it is used">{purpose}</td><td data-label="Designed retention">{retention}</td></tr>)}</tbody>
      </table>
    </div>
  )
}

export function FeaturesPage() {
  return (
    <article className="content-page">
      <ContentHeader
        eyebrow="Stheno capabilities"
        title="Commerce operations for merchants, builders, and developers."
        intro="Stheno connects merchant operations with a Rust commerce engine, without requiring every merchant, builder, or developer to work the same way."
      />
      <section className="content-section section-shell" aria-labelledby="merchant-operations">
        <div className="content-section-heading"><p className="eyebrow">For store owners</p><h2 id="merchant-operations">Run everyday commerce work from one merchant workspace.</h2></div>
        <div className="content-card-grid">
          <section className="content-card"><h3>Catalogue and stock</h3><p>Manage products, variants, categories, prices, stock locations, and inventory reservations. Spreadsheet import maps headers, discards unmapped columns, previews the result, and supports CSV export.</p></section>
          <section className="content-card"><h3>Guest checkout and COD</h3><p>Buyers can place an order without an account. Cash on delivery is available in every plan; payment gateways are a Pro capability.</p></section>
          <section className="content-card"><h3>Social order handoff</h3><p>WhatsApp can open with a pre-filled order message. Messenger and Instagram use a copy-then-open handoff. Each social handoff creates a draft order for staff review.</p></section>
          <section className="content-card"><h3>Orders and reporting</h3><p>Track orders through their lifecycle, maintain inventory, receive web-push operational notifications, and review orders, revenue, top products, and order source. Analytics history varies by plan.</p></section>
        </div>
      </section>
      <section className="content-section section-shell content-section-tone" aria-labelledby="build-with-stheno">
        <div className="content-section-heading"><p className="eyebrow">For ecommerce builders</p><h2 id="build-with-stheno">Keep the commerce core stable while storefront presentation can change.</h2></div>
        <div className="content-prose"><p>Trial and Free stores receive a supported brand kit and subdomain. Pro includes a developer-built bespoke storefront and custom domain after payment. Stheno keeps routing, data fetching, checkout behaviour, and payment integrations in the commerce core.</p><p>A bespoke storefront is an independently built frontend module, not a recolour of the default storefront. It can change design while using the same catalogue, cart, checkout, and operational workflows.</p></div>
      </section>
      <section className="content-section section-shell" aria-labelledby="developer-platform">
        <div className="content-section-heading"><p className="eyebrow">For developers</p><h2 id="developer-platform">A commerce engine with explicit operational boundaries.</h2></div>
        <div className="content-card-grid content-card-grid-three">
          <section className="content-card"><h3>Rust and Postgres</h3><p>Stheno is an open-source, Rust-based commerce engine using Postgres for persistence. Money is represented with exact decimal values, not floating-point numbers.</p></section>
          <section className="content-card"><h3>A published API contract</h3><p>The Store and Admin surfaces are specified in OpenAPI, with typed SDK generation specified as part of the developer surface, so integrations are written against a documented contract rather than inferred from responses.</p></section>
          <section className="content-card"><h3>Durable workflows</h3><p>Multi-step operations are modelled as durable workflows with compensation and idempotency requirements. Payment provider calls use an idempotency key and reconciliation path for ambiguous outcomes.</p></section>
        </div>
      </section>
      <section className="content-section section-shell content-disclosure" aria-label="Capability availability"><h2>Availability matters.</h2><p>Feature descriptions are based on Stheno product and entitlement specifications. A plan, integration, and delivery date are confirmed in the applicable service order; configuration can vary by tenant and region.</p></section>
    </article>
  )
}

export function PricingPage() {
  return (
    <article className="content-page">
      <ContentHeader eyebrow="Plans and entitlement levels" title="A clear path from first store request to paid operations." intro="Stheno separates core commerce, which remains available to every merchant, from services with an ongoing operating cost. The service order takes precedence over this summary." />
      <section className="content-section section-shell" aria-labelledby="plan-comparison"><h2 id="plan-comparison">Plan overview</h2><PlanTable /></section>
      <section className="content-section section-shell content-section-tone" aria-labelledby="plan-rules"><div className="content-section-heading"><p className="eyebrow">How plan changes work</p><h2 id="plan-rules">Your existing commerce data stays yours.</h2></div><div className="content-prose"><p>The trial begins when an email-verified merchant submits a store request. At trial expiry, the account moves to Free; no existing products, staff records, or order data are deleted or hidden.</p><p>If a merchant is above a Free quota after a downgrade, existing data remains readable, editable, and sellable. The limit applies to new creation until the merchant upgrades or reduces usage. Pro is offered on a 6- or 12-month commitment.</p></div></section>
      <section className="content-section section-shell content-disclosure" aria-label="Commercial terms note"><h2>Commercial terms</h2><p>Pro pricing, payment schedule, currency, and regional availability are stated before a merchant accepts a paid plan. A merchant may request a renewal or new paid term at any time; the applicable term, start date, and invoice are confirmed before it takes effect. Taxes are determined from billing details, applicable registrations, and law, and are shown before payment where Stheno must collect them.</p></section>
    </article>
  )
}

export function PrivacyPage() {
  return (
    <article className="content-page legal-page">
      <ContentHeader eyebrow="Privacy notice - publication draft" title="How Stheno is designed to handle merchant and shopper data." intro="Stheno is a product of Syed Systems, the software and IT division of SMCTi (Syed Monsur Construction Technology & ideas Ltd.). SMCTi's business address is 395 Purba Goran, Adarsha School Road, Khilgaon, Dhaka-1219, Bangladesh. This notice remains a draft until its privacy contact, applicable jurisdictions, active subprocessors, and effective date are confirmed." />
      <section className="content-section section-shell"><h2>Roles and data ownership</h2><div className="content-prose"><p>Merchants retain ownership and control of the store, catalogue, customer, buyer, order, and other commerce data they bring to Stheno. For that merchant data, the merchant determines the business purpose and Stheno processes it only to provide the contracted commerce service.</p><p>SMCTi, through Syed Systems, controls the Stheno service, including product, security, operational, and aggregated platform analytics. Usage analytics that can identify a person remain personal data and are handled under this notice; Stheno must not use merchant buyer data for unrelated advertising or to build a buyer-marketing audience.</p><p>This division must be reflected in the final customer agreement and data-processing terms. Buyer data can enter Stheno through guest checkout, a social-order handoff, or a merchant import.</p></div></section>
      <section className="content-section section-shell content-section-tone"><h2>Data categories and operating purpose</h2><DataProcessingTable /></section>
      <section className="content-section section-shell"><h2>Deletion, access, and correction</h2><div className="content-prose"><p>Where a valid erasure request applies, Stheno is designed to delete authentication and incomplete-cart data, pseudonymise buyer and address fields in financial records, redact provider payloads, and rebuild affected search and read indexes. Financial records remain for the applicable accounting retention period without directly identifying fields.</p><p>The published notice must name a working privacy contact and explain requests for access, export, correction, objection, deletion, or marketing-consent change, including response timing and identity verification.</p></div></section>
      <section className="content-section section-shell"><h2>Cookies and similar storage</h2><div className="content-prose"><p>Stheno does not use advertising cookies. The service can use strictly necessary browser storage for authentication, CSRF protection, anonymous carts, security, and operating preferences. These are not optional analytics trackers.</p><p>If Stheno introduces non-essential analytics, advertising, or similar tracking, it will update this notice and provide the notice and consent controls required for the visitor's jurisdiction before enabling that tracking.</p></div></section>
      <section className="content-section section-shell content-section-tone"><h2>Security and international processing</h2><div className="content-prose"><p>Stheno's documented security design includes tenant isolation through Postgres row-level security, encrypted provider credentials, scoped authentication, audit logging, retention jobs, and payment flows where card data is entered into provider-hosted elements rather than Stheno servers.</p><p>Hosting is selected with the merchant's region and operational needs in mind. Static delivery may use Cloudflare Pages or GitHub Pages; application services may use a cloud region close to the merchant, including Microsoft Azure where selected. The published notice must identify the providers and regions actually active for the deployment, transfer safeguards, and the current subprocessor list.</p></div></section>
    </article>
  )
}

export function TermsPage() {
  return (
    <article className="content-page legal-page">
      <ContentHeader eyebrow="Terms of service - publication draft" title="The service terms should match how Stheno actually operates." intro="Stheno is a product of Syed Systems, the software and IT division of SMCTi (Syed Monsur Construction Technology & ideas Ltd.). SMCTi's business address is 395 Purba Goran, Adarsha School Road, Khilgaon, Dhaka-1219, Bangladesh. This product-specific draft still requires governing law, dispute process, service levels, payment terms, and an effective date before it becomes binding Terms of Service." />
      <section className="content-section section-shell"><h2>Service scope</h2><div className="content-prose"><p>Stheno provides merchant commerce operations, including catalogue and inventory management, guest checkout, order handling, supported storefront presentation, and the plan capabilities shown on the pricing page. The service is not a marketplace, a buyer-account platform, a merchant-uploaded-code hosting service, or a general-purpose page builder.</p><p>A merchant remains responsible for its catalogue, prices, tax obligations, fulfilment promises, buyer communications, consumer-law notices, and the lawful basis for buyer information it provides to Stheno. Stheno does not become the seller of a merchant's goods.</p></div></section>
      <section className="content-section section-shell content-section-tone"><h2>Plans, upgrades, and limits</h2><div className="content-prose"><p>Trial, Free, and Pro access is controlled through entitlements. A disabled paid capability is unavailable rather than partially enabled. Quotas apply to new creation, while existing data remains intact after trial expiry or downgrade. Pro requires a 6- or 12-month commitment.</p><p>A merchant may request a renewal, upgrade, or additional paid term at any time. The plan, term, price, currency, taxes, and start date are confirmed in the service order or invoice before a paid term begins. A renewal is not automatic unless the merchant expressly agrees to it in those commercial terms.</p></div></section>
      <section className="content-section section-shell" aria-labelledby="taxes-and-invoices"><h2 id="taxes-and-invoices">Taxes and invoices</h2><div className="content-prose"><p>Taxes on a Stheno subscription are separate from taxes a merchant charges buyers for the merchant's goods. Stheno charges, collects, and remits subscription taxes only where it is legally required or has agreed to do so; the merchant remains responsible for its own sales taxes unless a separate written tax service says otherwise.</p><p>Subscription tax is determined from billing country, verified business or consumer status, tax identifier where applicable, billing address, place-of-supply rules, and Stheno's registrations. Bangladesh VAT is applied at the applicable statutory rate (the standard rate is currently 15% when applicable). India requires GST and place-of-supply analysis; EU VAT depends on the relevant member state and customer status; US sales tax is state- and locality-specific. The final amount is shown before payment when tax must be charged.</p></div></section>
      <section className="content-section section-shell"><h2>Acceptable use and integrations</h2><div className="content-prose"><p>Merchants must not use Stheno to process unlawful goods, submit data they lack authority to use, bypass a payment provider's rules, probe other tenant data, or interfere with the service. A merchant must keep account credentials secure and give staff only the access needed for their work.</p><p>Payment gateways, couriers, domains, and other third-party services remain subject to their own terms. Pro features that rely on providers can depend on provider approval, credentials, regional availability, and merchant compliance. Card data is not accepted by Stheno servers; payment providers handle card-entry elements and payment processing.</p></div></section>
      <section className="content-section section-shell content-section-tone"><h2>Suspension, termination, and refunds</h2><div className="content-prose"><p>SMCTi may immediately restrict or suspend access where it reasonably believes there is unlawful activity, an attempt to gain unauthorised access, an attack on the service or another tenant, material fraud, or a serious breach of these Terms. It may preserve evidence and cooperate with lawful authorities, and may delay notice where notice would compromise security or an investigation.</p><p>Where legally permitted, fees are not refunded for a paid term suspended or terminated because of that conduct. This does not remove non-waivable consumer or statutory refund rights. The final Terms must define the review and appeal route, ordinary cancellation treatment, and refund timing for a non-fault suspension.</p></div></section>
      <section className="content-section section-shell"><h2>Support, changes, and legal terms still required</h2><div className="content-prose"><p>The published Terms need a contact channel, support hours, service-maintenance policy, notice method, change-control process, intellectual-property licence, confidentiality terms, limitation of liability, indemnity, governing law, venue, and consumer-law carve-outs. These details must be selected by SMCTi, through Syed Systems, and reviewed by counsel before publication.</p></div></section>
    </article>
  )
}
