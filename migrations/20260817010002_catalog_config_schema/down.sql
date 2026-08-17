DROP TABLE IF EXISTS store.store_locale;
DROP TABLE IF EXISTS store.store_currency;
DROP TABLE IF EXISTS store.store;
DROP SCHEMA IF EXISTS store CASCADE;

DROP TABLE IF EXISTS tax.tax_provider;
DROP TABLE IF EXISTS tax.tax_rate_rule;
DROP TABLE IF EXISTS tax.tax_rate;
DROP TABLE IF EXISTS tax.tax_region;
DROP SCHEMA IF EXISTS tax CASCADE;

DROP TABLE IF EXISTS sales_channel.sales_channel;
DROP SCHEMA IF EXISTS sales_channel CASCADE;

DROP TABLE IF EXISTS stock_location.stock_location_address;
DROP TABLE IF EXISTS stock_location.stock_location;
DROP SCHEMA IF EXISTS stock_location CASCADE;
