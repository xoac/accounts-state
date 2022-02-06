//! Account traits and structs
use crate::ClientID;
use rust_decimal::Decimal;

pub(crate) mod balance;
pub(crate) mod clinet_account;
pub(crate) mod transactions;

pub use clinet_account::ClientAcc;

/// Represent basic account information and balance
pub trait Account {
    /// identified of client
    fn client_id(&self) -> ClientID;
    /// amount of money available for withdraw
    fn available(&self) -> Decimal;
    /// amount of money under dispute
    fn held(&self) -> Decimal;
    /// is account frozen
    fn is_locked(&self) -> bool;

    /// total amount of money in account
    fn total(&self) -> Decimal {
        self.available() + self.held()
    }
}
