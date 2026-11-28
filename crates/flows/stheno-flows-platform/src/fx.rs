use rust_decimal::Decimal;
use std::sync::Arc;
use stheno_platform::service::PlatformService;

pub struct BangladeshBankRateFetcher {
    platform: Arc<PlatformService>,
}

impl BangladeshBankRateFetcher {
    pub fn new(platform: Arc<PlatformService>) -> Self {
        Self { platform }
    }

    pub async fn fetch_and_store(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Integrate actual Bangladesh Bank API when available.
        // For C3, we implement the job structure and store a rate.
        let fetched_rate = Decimal::from(120);
        let source = "bangladesh_bank";
        let rate_kind = "WAR";

        self.platform
            .upsert_fx_rate("USD/BDT", fetched_rate, source, rate_kind)
            .await?;

        Ok(())
    }
}
