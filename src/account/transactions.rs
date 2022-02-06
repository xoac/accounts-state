//! Represents transactions
use crate::{amount::Amount, errors::TransUpdateErr};

/// Represent a single account transaction with it current state
#[derive(Debug, Clone)]
pub struct AccTrans {
    money: MoneyTrans,
    #[cfg(test)]
    pub state: AccTransState,
    #[cfg(not(test))]
    state: AccTransState,
}

impl AccTrans {
    pub fn succeed(money: MoneyTrans) -> Self {
        Self {
            money,
            state: AccTransState::Init,
        }
    }

    pub fn failed(money: MoneyTrans) -> Self {
        Self {
            money,
            state: AccTransState::Failed,
        }
    }

    pub fn mark_dispute(&mut self) -> Result<&MoneyTrans, TransUpdateErr> {
        match self.state {
            AccTransState::Resolved | AccTransState::Init => (),
            AccTransState::Dispute | AccTransState::Chargebacked => {
                return Err(TransUpdateErr::IncorectState)
            }
            AccTransState::Failed => return Err(TransUpdateErr::Failed),
        }

        self.state = AccTransState::Dispute;
        Ok(&self.money)
    }

    pub fn mark_resolved(&mut self) -> Result<&MoneyTrans, TransUpdateErr> {
        match self.state {
            AccTransState::Resolved | AccTransState::Init | AccTransState::Chargebacked => {
                return Err(TransUpdateErr::IncorectState)
            }
            AccTransState::Dispute => (),
            AccTransState::Failed => return Err(TransUpdateErr::Failed),
        }

        self.state = AccTransState::Resolved;
        Ok(&self.money)
    }

    pub fn mark_chargebacked(&mut self) -> Result<&MoneyTrans, TransUpdateErr> {
        match self.state {
            AccTransState::Resolved | AccTransState::Init | AccTransState::Chargebacked => {
                return Err(TransUpdateErr::IncorectState);
            }
            AccTransState::Dispute => (),
            AccTransState::Failed => return Err(TransUpdateErr::Failed),
        }

        self.state = AccTransState::Chargebacked;
        Ok(&self.money)
    }
}

/// Represnet one of many state in witch [`AccTrans`] can be
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccTransState {
    Init, // Succeed
    /// transaction could be erroneous and should be reversed after check
    Dispute,
    /// funds that were previously disputed are no longer disputed
    Resolved,
    /// final state, disputed transaction has been
    Chargebacked,
    /// Operation was only recorded but there is no impact on founds
    Failed, // for example Withdraw when there wan not enough money.
}

// Money transaction represent withdraw or deposit
#[derive(Debug, Clone)]
pub enum MoneyTrans {
    /// increase available founds
    Deposit(Amount),
    /// decrease available founds
    Withdraw(Amount),
}
