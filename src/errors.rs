//! Possible errors

use crate::amount::NegativeAmountErr;
use thiserror::Error;

/// Group errors for account balance
#[allow(missing_docs)]
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum BalanceErr {
    #[error("not enough found available for this operation")]
    NotEnoughAvailableFounds,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
/// Group error that returns when transaction update fail
pub enum TransUpdateErr {
    /// Update of this transaction was not possible in this state
    #[error("incorrect state of transaction to perform update")]
    IncorectState,
    /// Transaction failed and no update can be applied
    #[error("failed transaction")]
    Failed,
}

/// Group all errors that can occurs within account module
#[allow(missing_docs)]
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum AccountErr {
    #[error("could not find referential transaction id for requested operation")]
    NoRefTransID,
    #[error("account money change error")]
    AccMoney(#[from] BalanceErr),
    #[error("transaction update error")]
    AccTrans(#[from] TransUpdateErr),
    #[error("amount was negative")]
    Amount(#[from] NegativeAmountErr),
    #[error("account is frozen")]
    Frozen,
}
