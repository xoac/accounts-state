//! Protect before using negative amount for deposits and withdraws.

use std::{borrow::Borrow, ops::Deref};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represent positive financial amount of money
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Amount(Decimal);

impl Amount {
    /// Create new amount of that is guarantee to be positive
    pub fn new(num: u64, scale: u32) -> Amount {
        let inner = Decimal::from_i128_with_scale(num.into(), scale);
        Self(inner)
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("amount is negative")]
/// represent error when transaction want to operate on negative amount of money
pub struct NegativeAmountErr;

impl TryFrom<Decimal> for Amount {
    type Error = NegativeAmountErr;
    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        if value.is_sign_negative() {
            Err(NegativeAmountErr)
        } else {
            Ok(Self(value))
        }
    }
}

impl From<Amount> for Decimal {
    fn from(this: Amount) -> Self {
        this.0
    }
}

impl Borrow<Decimal> for Amount {
    fn borrow(&self) -> &Decimal {
        &self.0
    }
}

impl Deref for Amount {
    type Target = Decimal;
    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}
