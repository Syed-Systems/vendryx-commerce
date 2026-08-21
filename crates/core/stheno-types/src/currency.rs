use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CurrencyCode {
    Usd,
    Eur,
    Gbp,
    Jpy,
    Cad,
    Aud,
    Chf,
    Cny,
    Sek,
    Nzd,
    /// Bangladeshi Taka. Two decimals, so it needs no entry in
    /// `minor_unit_exponent`. Added because the product sells subscriptions in BDT
    /// ([ADR-017](../../../docs/DECISIONS.md#adr-017)) and the money type could not
    /// express the only currency the business actually charges in.
    Bdt,
    // Zero-decimal currencies (PAYMENTS.md §5)
    Bif,
    Clp,
    Djf,
    Gnf,
    Kmf,
    Krw,
    Mga,
    Pyg,
    Rwf,
    Ugx,
    Vnd,
    Vuv,
    Xaf,
    Xof,
    Xpf,
    // Three-decimal currencies (PAYMENTS.md §5)
    Bhd,
    Iqd,
    Jod,
    Kwd,
    Omr,
    Tnd,
}

impl CurrencyCode {
    pub fn minor_unit_exponent(&self) -> u32 {
        match self {
            CurrencyCode::Bif
            | CurrencyCode::Clp
            | CurrencyCode::Djf
            | CurrencyCode::Gnf
            | CurrencyCode::Jpy
            | CurrencyCode::Kmf
            | CurrencyCode::Krw
            | CurrencyCode::Mga
            | CurrencyCode::Pyg
            | CurrencyCode::Rwf
            | CurrencyCode::Ugx
            | CurrencyCode::Vnd
            | CurrencyCode::Vuv
            | CurrencyCode::Xaf
            | CurrencyCode::Xof
            | CurrencyCode::Xpf => 0,

            CurrencyCode::Bhd
            | CurrencyCode::Iqd
            | CurrencyCode::Jod
            | CurrencyCode::Kwd
            | CurrencyCode::Omr
            | CurrencyCode::Tnd => 3,

            _ => 2,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CurrencyCode::Usd => "usd",
            CurrencyCode::Eur => "eur",
            CurrencyCode::Gbp => "gbp",
            CurrencyCode::Jpy => "jpy",
            CurrencyCode::Cad => "cad",
            CurrencyCode::Aud => "aud",
            CurrencyCode::Chf => "chf",
            CurrencyCode::Cny => "cny",
            CurrencyCode::Sek => "sek",
            CurrencyCode::Nzd => "nzd",
            CurrencyCode::Bdt => "bdt",
            CurrencyCode::Bif => "bif",
            CurrencyCode::Clp => "clp",
            CurrencyCode::Djf => "djf",
            CurrencyCode::Gnf => "gnf",
            CurrencyCode::Kmf => "kmf",
            CurrencyCode::Krw => "krw",
            CurrencyCode::Mga => "mga",
            CurrencyCode::Pyg => "pyg",
            CurrencyCode::Rwf => "rwf",
            CurrencyCode::Ugx => "ugx",
            CurrencyCode::Vnd => "vnd",
            CurrencyCode::Vuv => "vuv",
            CurrencyCode::Xaf => "xaf",
            CurrencyCode::Xof => "xof",
            CurrencyCode::Xpf => "xpf",
            CurrencyCode::Bhd => "bhd",
            CurrencyCode::Iqd => "iqd",
            CurrencyCode::Jod => "jod",
            CurrencyCode::Kwd => "kwd",
            CurrencyCode::Omr => "omr",
            CurrencyCode::Tnd => "tnd",
        }
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CurrencyCode {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "usd" => Ok(CurrencyCode::Usd),
            "eur" => Ok(CurrencyCode::Eur),
            "gbp" => Ok(CurrencyCode::Gbp),
            "jpy" => Ok(CurrencyCode::Jpy),
            "cad" => Ok(CurrencyCode::Cad),
            "aud" => Ok(CurrencyCode::Aud),
            "chf" => Ok(CurrencyCode::Chf),
            "cny" => Ok(CurrencyCode::Cny),
            "sek" => Ok(CurrencyCode::Sek),
            "nzd" => Ok(CurrencyCode::Nzd),
            "bdt" => Ok(CurrencyCode::Bdt),
            "bif" => Ok(CurrencyCode::Bif),
            "clp" => Ok(CurrencyCode::Clp),
            "djf" => Ok(CurrencyCode::Djf),
            "gnf" => Ok(CurrencyCode::Gnf),
            "kmf" => Ok(CurrencyCode::Kmf),
            "krw" => Ok(CurrencyCode::Krw),
            "mga" => Ok(CurrencyCode::Mga),
            "pyg" => Ok(CurrencyCode::Pyg),
            "rwf" => Ok(CurrencyCode::Rwf),
            "ugx" => Ok(CurrencyCode::Ugx),
            "vnd" => Ok(CurrencyCode::Vnd),
            "vuv" => Ok(CurrencyCode::Vuv),
            "xaf" => Ok(CurrencyCode::Xaf),
            "xof" => Ok(CurrencyCode::Xof),
            "xpf" => Ok(CurrencyCode::Xpf),
            "bhd" => Ok(CurrencyCode::Bhd),
            "iqd" => Ok(CurrencyCode::Iqd),
            "jod" => Ok(CurrencyCode::Jod),
            "kwd" => Ok(CurrencyCode::Kwd),
            "omr" => Ok(CurrencyCode::Omr),
            "tnd" => Ok(CurrencyCode::Tnd),
            _ => Err(crate::Error::InvalidCurrency(s.to_string())),
        }
    }
}
