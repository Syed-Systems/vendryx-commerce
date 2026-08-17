# Provider Authoring Guide

Providers integrate third-party services (payment gateways, shipping carriers, tax calculators, notification providers) into Stheno.

---

## 1. Provider Rules (AGENTS.md §5)
- Provider crates live under `crates/providers/`.
- Must implement the domain trait fully without `todo!()`.
- Mutating external calls must use `idempotent_call()` with unique idempotency keys.
- Amounts crossing provider boundaries must use `to_minor_units()` and `from_minor_units()`.

---

## 2. Implementing a Payment Provider
```rust
use async_trait::async_trait;
use stheno_payment::{PaymentProvider, PaymentSession, PaymentResult};

pub struct CustomPaymentProvider;

#[async_trait]
impl PaymentProvider for CustomPaymentProvider {
    async fn authorize(&self, session: &PaymentSession) -> PaymentResult {
        // Authorize via external API with idempotency key
        Ok(())
    }
}
```
