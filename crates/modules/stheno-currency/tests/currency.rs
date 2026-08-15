use stheno_currency::CurrencyService;

#[tokio::test]
async fn test_currency_iso_4217_precision_and_retrieval() {
    let service = CurrencyService::new();

    // 1. Check USD (2 decimals)
    let usd = service.get_currency("usd").await.expect("must get usd");
    assert_eq!(usd.code, "usd");
    assert_eq!(usd.decimal_digits, 2);
    assert_eq!(usd.symbol, "$");

    // 2. Check JPY (0 decimals - zero decimal currency)
    let jpy = service
        .get_currency("JPY")
        .await
        .expect("must get jpy case-insensitive");
    assert_eq!(jpy.decimal_digits, 0);

    // 3. Check BHD (3 decimals - three decimal currency)
    let bhd = service.get_currency("bhd").await.expect("must get bhd");
    assert_eq!(bhd.decimal_digits, 3);

    // 4. Check list_currencies
    let list = service
        .list_currencies()
        .await
        .expect("must list currencies");
    assert!(list.len() >= 20);

    // 5. Check unknown currency
    let unknown = service.get_currency("xyz").await;
    assert!(unknown.is_err());
}
