use super::transactions::MoneyTrans;
use crate::errors::BalanceErr;
use rust_decimal::Decimal;

/// Represents current client account balance
#[derive(Debug, Clone, Default)]
pub struct Balance {
    available: Decimal,
    held: Decimal,

    // amount of money that are under dispute
    // but at the moment of dispute do not belong to this account
    // FIXME: make sure it's correct implementation
    held_withdraw: Decimal,
}

impl Balance {
    pub fn deposit(&mut self, amount: &Decimal) {
        self.available += amount;
    }

    pub fn try_withdraw(&mut self, amount: &Decimal) -> Result<(), BalanceErr> {
        if self.available < *amount {
            return Err(BalanceErr::NotEnoughAvailableFounds);
        }

        self.available -= amount;
        Ok(())
    }

    pub fn dispute(&mut self, mop: &MoneyTrans) {
        match mop {
            MoneyTrans::Deposit(amount) => self.dispute_deposit(amount),
            MoneyTrans::Withdraw(amount) => self.dispute_withdraw(amount),
        }
    }

    pub fn dispute_deposit(&mut self, amount: &Decimal) {
        self.available -= amount;
        self.held += amount;
    }

    fn dispute_withdraw(&mut self, amount: &Decimal) {
        self.held_withdraw += amount;
    }

    pub fn resolve(&mut self, mop: &MoneyTrans) {
        match mop {
            MoneyTrans::Withdraw(amount) => self.resolve_withdraw(amount),
            MoneyTrans::Deposit(amount) => self.resolve_deposit(amount),
        }
    }

    fn resolve_deposit(&mut self, amount: &Decimal) {
        self.available += amount;
        self.held -= amount;
        debug_assert!(self.held >= Decimal::ZERO);
    }

    fn resolve_withdraw(&mut self, amount: &Decimal) {
        self.held_withdraw -= amount;
        debug_assert!(self.held_withdraw >= Decimal::ZERO);
    }

    pub fn chargeback(&mut self, mop: &MoneyTrans) {
        match mop {
            MoneyTrans::Withdraw(amount) => self.chargeback_withdraw(amount),
            MoneyTrans::Deposit(amount) => self.chargeback_deposit(amount),
        }
    }

    fn chargeback_withdraw(&mut self, amount: &Decimal) {
        self.held_withdraw -= amount;
        self.available += amount;
    }

    fn chargeback_deposit(&mut self, amount: &Decimal) {
        self.held -= amount;
        debug_assert!(self.held >= Decimal::ZERO);
    }
}

impl Balance {
    pub fn available(&self) -> Decimal {
        self.available
    }

    pub fn held(&self) -> Decimal {
        // FIXME: what with `self.held_withdraw`?
        self.held
    }
}
