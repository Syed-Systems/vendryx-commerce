-- Phase: expand
-- Creates the currency module schema and seeds ISO 4217 reference data.
--
-- The currency list is reference data, not merchant data: it ships with the
-- schema so a fresh deployment can price an order before anyone has configured
-- anything. `decimal_digits` is load-bearing for money rounding (ADR-006), so a
-- missing row must be impossible rather than defaulted.

CREATE SCHEMA IF NOT EXISTS currency;

CREATE TABLE IF NOT EXISTS currency.currency (
    code           text PRIMARY KEY,
    symbol         text NOT NULL,
    symbol_native  text NOT NULL,
    name           text NOT NULL,
    decimal_digits smallint NOT NULL,
    rounding       integer NOT NULL DEFAULT 0,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    deleted_at     timestamptz NULL,
    CONSTRAINT currency_code_is_lowercase CHECK (code = lower(code)),
    CONSTRAINT currency_digits_in_iso_range CHECK (decimal_digits BETWEEN 0 AND 4)
);

INSERT INTO currency.currency (code, symbol, symbol_native, name, decimal_digits, rounding) VALUES
    ('usd', '$', '$', 'US Dollar', 2, 0),
    ('eur', '€', '€', 'Euro', 2, 0),
    ('gbp', '£', '£', 'British Pound', 2, 0),
    ('jpy', '¥', '￥', 'Japanese Yen', 0, 0),
    ('cad', 'CA$', '$', 'Canadian Dollar', 2, 0),
    ('aud', 'AU$', '$', 'Australian Dollar', 2, 0),
    ('chf', 'CHF', 'CHF', 'Swiss Franc', 2, 0),
    ('cny', 'CN¥', '¥', 'Chinese Yuan', 2, 0),
    ('inr', '₹', '₹', 'Indian Rupee', 2, 0),
    ('krw', '₩', '₩', 'South Korean Won', 0, 0),
    ('bhd', 'BD', 'د.ب', 'Bahraini Dinar', 3, 0),
    ('kwd', 'KD', 'د.ك', 'Kuwaiti Dinar', 3, 0),
    ('omr', 'OMR', 'ر.ع.', 'Omani Rial', 3, 0),
    ('sek', 'kr', 'kr', 'Swedish Krona', 2, 0),
    ('nok', 'kr', 'kr', 'Norwegian Krone', 2, 0),
    ('dkk', 'kr', 'kr', 'Danish Krone', 2, 0),
    ('sgd', 'S$', '$', 'Singapore Dollar', 2, 0),
    ('hkd', 'HK$', '$', 'Hong Kong Dollar', 2, 0),
    ('nzd', 'NZ$', '$', 'New Zealand Dollar', 2, 0),
    ('aed', 'AED', 'د.إ', 'United Arab Emirates Dirham', 2, 0)
ON CONFLICT (code) DO NOTHING;
