//!"Simple toy payments engine for csv input"

#![deny(missing_docs)]

pub mod account;
pub mod amount;
pub mod csv;
pub mod errors;

/// Transaction identifier. Unique across all [`ClientID`]
pub type TransID = u32;
/// Client identyfier
pub type ClientID = u16;
