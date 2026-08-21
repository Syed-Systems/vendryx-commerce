use std::collections::BTreeMap;

use stheno_api_core::Tier;
use stheno_registry::AppContext;
use stheno_server::{load_tier_config, AppServer};

#[tokio::test]
async fn gateway_factories_are_registered_even_when_boot_fallback_is_free() {
    let db = stheno_db::testing::TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("test pool must build");
    let entitlements = load_tier_config()
        .expect("tier config must parse")
        .resolve(Tier::Free, &BTreeMap::new())
        .expect("free tier resolves");

    let server = AppServer::new(AppContext::create_test_context(), entitlements, pool);

    server
        .payment_service
        .get_factory("pp_system_default")
        .expect("COD factory is always available");
    server
        .payment_service
        .get_factory("pp_stripe_stripe")
        .expect("gateway factory registration is not controlled by boot fallback entitlements");

    db.drop().await;
}
