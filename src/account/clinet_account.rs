use super::balance::Balance;
use super::transactions::*;
/// Represents client account
use super::Account;
use crate::{amount::Amount, csv::RawTransaction, errors::AccountErr, ClientID, TransID};
use rust_decimal::Decimal;
use std::borrow::Borrow;
use std::collections::BTreeMap;

/// Client current balance and transactions history
#[derive(Debug, Clone)]
pub struct ClientAcc {
    // TODO: if TransID is guarantee to be continuous this can be replaced with std::Vec
    client_id: ClientID,
    trans_history: BTreeMap<TransID, AccTrans>,
    balance: Balance,
    locked: bool,
}

impl ClientAcc {
    /// Create new empty unlocked [`ClientAcc`] for `client_id`
    pub fn new(client_id: ClientID) -> Self {
        Self {
            client_id,
            trans_history: BTreeMap::new(),
            balance: Default::default(),
            locked: false,
        }
    }

    #[cfg(test)]
    /// New account with available found 750.0 and held 0.0
    pub fn new_test_account() -> Self {
        let mut a = ClientAcc::new(1);
        for trans_id in 1..11 {
            a.record_deposit(trans_id, Amount::new(100, 0)).unwrap();
        }

        for trans_id in 11..16 {
            a.record_withdraw(trans_id, Amount::new(50, 0)).unwrap();
        }

        assert_eq!(a.available(), Decimal::new(1000, 0) - Decimal::new(250, 0));

        a
    }

    /// trying to apply next transaction.
    ///
    /// Be aware that operation must be applied with order of execution
    pub fn try_apply_new_raw_transaction(
        &mut self,
        raw_transaction: RawTransaction,
    ) -> Result<(), AccountErr> {
        match raw_transaction.r#type {
            crate::csv::RawTrasactionType::Withdrawal => {
                let amount = raw_transaction.amount.unwrap().try_into()?;
                self.record_withdraw(raw_transaction.transaction_id, amount)
            }
            crate::csv::RawTrasactionType::Deposit => {
                let amount = raw_transaction.amount.unwrap().try_into()?;
                self.record_deposit(raw_transaction.transaction_id, amount)
            }
            crate::csv::RawTrasactionType::Dispute => {
                self.try_dispute(raw_transaction.transaction_id)
            }
            crate::csv::RawTrasactionType::Resolve => {
                self.try_resolve(raw_transaction.transaction_id)
            }
            crate::csv::RawTrasactionType::Chargeback => {
                self.try_chargeback(raw_transaction.transaction_id)
            }
        }
    }

    fn try_chargeback(&mut self, ref_trans_id: TransID) -> Result<(), AccountErr> {
        self.check_locked()?;

        let ref_trans = self
            .get_trans_mut(&ref_trans_id)
            .ok_or(AccountErr::NoRefTransID)?;

        let money = ref_trans.mark_chargebacked()?.clone();
        self.locked = true;
        self.balance.chargeback(&money);

        Ok(())
    }

    fn try_resolve(&mut self, ref_trans_id: TransID) -> Result<(), AccountErr> {
        self.check_locked()?;

        let ref_trans = self
            .get_trans_mut(&ref_trans_id)
            .ok_or(AccountErr::NoRefTransID)?;

        let money = ref_trans.mark_resolved()?.clone();

        self.balance.resolve(&money);

        Ok(())
    }

    fn try_dispute(&mut self, ref_trans_id: TransID) -> Result<(), AccountErr> {
        let ref_trans = self
            .get_trans_mut(&ref_trans_id)
            .ok_or(AccountErr::NoRefTransID)?;

        let money = ref_trans.mark_dispute()?.clone();

        self.balance.dispute(&money);

        Ok(())
    }

    /// try to increase available founds
    ///
    /// If this function return error transaction is also recorded but account balance was not
    /// changed.
    fn record_withdraw(&mut self, new_trans_id: TransID, amount: Amount) -> Result<(), AccountErr> {
        let succes_or_faild_record = self
            .check_locked()
            .and(self.balance.try_withdraw(&amount).map_err(AccountErr::from));

        match succes_or_faild_record {
            Ok(_) => {
                // not frozen, and have enough founds
                let acc_trans = AccTrans::succeed(MoneyTrans::Withdraw(amount));
                self.record_money_transaction(new_trans_id, acc_trans);
                Ok(())
            }
            Err(e) => {
                // only record transaction
                let acc_trans = AccTrans::failed(MoneyTrans::Withdraw(amount));
                self.record_money_transaction(new_trans_id, acc_trans);
                Err(e)
            }
        }
    }

    /// try to increase available founds
    ///
    /// If this function return error transaction is also recorded but account balance was not
    /// changed.
    fn record_deposit(&mut self, new_trans_id: TransID, amount: Amount) -> Result<(), AccountErr> {
        let succes_or_faild_record = self.check_locked();

        match succes_or_faild_record {
            Ok(_) => {
                self.balance.deposit(&amount);
                let acc_trans = AccTrans::succeed(MoneyTrans::Deposit(amount));
                self.record_money_transaction(new_trans_id, acc_trans);
                Ok(())
            }
            Err(e) => {
                let acc_trans = AccTrans::failed(MoneyTrans::Deposit(amount));
                self.record_money_transaction(new_trans_id, acc_trans);
                Err(e)
            }
        }
    }

    fn record_money_transaction(&mut self, new_trans_id: TransID, acc_trans: AccTrans) {
        assert!(!self.trans_history.contains_key(&new_trans_id)); // FIXME: do not assume external data are correct!
        self.trans_history.insert(new_trans_id, acc_trans);
    }

    fn get_trans_mut(&mut self, ref_trans_id: impl Borrow<TransID>) -> Option<&mut AccTrans> {
        self.trans_history.get_mut(ref_trans_id.borrow())
    }

    fn check_locked(&self) -> Result<(), AccountErr> {
        if self.locked {
            Err(AccountErr::Frozen)
        } else {
            Ok(())
        }
    }
}

impl Account for ClientAcc {
    fn available(&self) -> Decimal {
        self.balance.available()
    }

    fn held(&self) -> Decimal {
        self.balance.held()
    }

    fn client_id(&self) -> ClientID {
        self.client_id
    }

    fn is_locked(&self) -> bool {
        self.locked
    }
}

#[cfg(test)]
mod test {
    use super::{AccTransState, Account, ClientAcc};
    use crate::amount::Amount;
    use crate::errors::TransUpdateErr;
    use rust_decimal::Decimal;

    #[test]
    fn preventing_debt_withdraw() {
        let mut a = ClientAcc::new(1);
        a.record_withdraw(1, Amount::new(100, 0)).unwrap_err();

        assert_eq!(a.get_trans_mut(1).unwrap().state, AccTransState::Failed);
        assert_eq!(a.available(), Decimal::ZERO);
        assert_eq!(a.held(), Decimal::ZERO);
    }

    #[test]
    fn deposit_money() {
        let mut a = ClientAcc::new(1);
        a.record_deposit(1, Amount::new(100, 1)).unwrap();

        assert_eq!(a.get_trans_mut(1).unwrap().state, AccTransState::Init);
        assert_eq!(a.available(), Decimal::new(100, 1));
        assert_eq!(a.held(), Decimal::ZERO);
    }

    #[test]
    fn many_deposits_adds_available_founds() {
        let mut a = ClientAcc::new(1);
        a.record_deposit(1, Amount::new(100, 1)).unwrap();
        a.record_deposit(2, Amount::new(50, 1)).unwrap();

        assert_eq!(a.get_trans_mut(1).unwrap().state, AccTransState::Init);
        assert_eq!(a.get_trans_mut(2).unwrap().state, AccTransState::Init);
        assert_eq!(a.available(), Decimal::new(150, 1));
        assert_eq!(a.held(), Decimal::ZERO);
    }

    #[test]
    fn deposit_and_withdraw() {
        let mut a = ClientAcc::new(1);
        a.record_deposit(1, Amount::new(100, 1)).unwrap();
        a.record_deposit(2, Amount::new(50, 1)).unwrap();

        a.record_withdraw(3, Amount::new(145, 1)).unwrap();
        a.record_withdraw(4, Amount::new(145, 1)).unwrap_err();

        assert_eq!(a.get_trans_mut(1).unwrap().state, AccTransState::Init);
        assert_eq!(a.get_trans_mut(2).unwrap().state, AccTransState::Init);
        assert_eq!(a.get_trans_mut(3).unwrap().state, AccTransState::Init);
        assert_eq!(a.get_trans_mut(4).unwrap().state, AccTransState::Failed);
        assert_eq!(a.available(), Decimal::new(5, 1));
        assert_eq!(a.held(), Decimal::ZERO);
    }

    #[test]
    fn chargeback_is_ignored_if_there_is_no_dispute() {
        let mut a = ClientAcc::new_test_account();

        // deposit
        let e = a.try_chargeback(1).unwrap_err();
        assert_eq!(e, TransUpdateErr::IncorectState.into());

        // withdraw
        let e = a.try_chargeback(11).unwrap_err();
        assert_eq!(e, TransUpdateErr::IncorectState.into());
    }

    #[test]
    fn chargeback_on_deposit_transaction() {
        let mut a = ClientAcc::new_test_account();

        a.try_dispute(1).unwrap();
        assert_eq!(a.get_trans_mut(1).unwrap().state, AccTransState::Dispute);
        assert_eq!(a.available(), Decimal::new(6500, 1));
        assert_eq!(a.held(), Decimal::new(100, 0));
        assert_eq!(a.total(), Decimal::new(750, 0));
        assert_eq!(a.locked, false);

        a.try_chargeback(1).unwrap();
        assert_eq!(
            a.get_trans_mut(1).unwrap().state,
            AccTransState::Chargebacked
        );
        assert_eq!(a.available(), Decimal::new(6500, 1));
        assert_eq!(a.held(), Decimal::ZERO);
        assert_eq!(a.locked, true);
    }

    #[test]
    fn chargeback_on_withdraw_transaction() {
        let mut a = ClientAcc::new_test_account();

        a.try_dispute(11).unwrap();
        assert_eq!(a.get_trans_mut(11).unwrap().state, AccTransState::Dispute);
        assert_eq!(a.available(), Decimal::new(750, 0));
        assert_eq!(a.held(), Decimal::ZERO);
        assert_eq!(a.total(), a.available());
        assert_eq!(a.locked, false);

        a.try_chargeback(11).unwrap();
        assert_eq!(
            a.get_trans_mut(11).unwrap().state,
            AccTransState::Chargebacked
        );
        assert_eq!(a.available(), Decimal::new(800, 0));
        assert_eq!(a.held(), Decimal::ZERO);
        assert_eq!(a.total(), a.available());
        assert_eq!(a.locked, true);
    }

    #[test]
    fn resolve_withdraw() {
        let mut a = ClientAcc::new_test_account();
        let a_snapshot = a.clone();

        a.try_dispute(11).unwrap();
        a.try_resolve(11).unwrap();

        assert_eq!(a.get_trans_mut(11).unwrap().state, AccTransState::Resolved);
        assert_eq!(a.available(), a_snapshot.available());
        assert_eq!(a.held(), a_snapshot.held());
        assert_eq!(a.total(), a_snapshot.total());
    }

    #[test]
    fn dispute_resolve_then_dispute_again_and_chargeback() {
        let mut a = ClientAcc::new_test_account();

        a.try_dispute(1).unwrap();
        a.try_resolve(1).unwrap();
        assert_eq!(a.total(), Decimal::new(750, 0));
        assert_eq!(a.available(), Decimal::new(750, 0));
        assert_eq!(a.held(), Decimal::ZERO);
        assert_eq!(a.locked, false);

        a.try_dispute(1).unwrap();
        assert_eq!(a.get_trans_mut(1).unwrap().state, AccTransState::Dispute);
        assert_eq!(a.available(), Decimal::new(6500, 1));
        assert_eq!(a.held(), Decimal::new(100, 0));
        assert_eq!(a.total(), Decimal::new(750, 0));

        a.try_chargeback(1).unwrap();
        assert_eq!(
            a.get_trans_mut(1).unwrap().state,
            AccTransState::Chargebacked
        );
        assert_eq!(a.available(), Decimal::new(6500, 1));
        assert_eq!(a.held(), Decimal::ZERO);
        assert_eq!(a.locked, true);
    }
}
