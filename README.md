


## Assumptions

1. If account is frozen/locked all withdraw and deposit transaction will fail. Transactions will be
   recorded, and account will still accept to make other disputes (but only for valid money
   transactions). Other chargebacks and resolves wouldn't be possible until account is unlocked.
2. **amount** is a positive number that represents money.
3. The dispute operation behaves behaves differently for withdraw and deposit operation. The specification was inconsistent in this case because it required:
    1. prepare for reversing a transaction
    2. increase `held` founds by **`amount`** and decrease `avaliable` founds by **`amount`**
In case `2.` if withdraw transaction will be disputed and chargebacked. The account's balance will be in
incorrect state (but hopefully account is also frozen). Since the total money in the account was doubly decreased.

## Correctnes

1. The basic cases like **dispute**, **resolve**, **chargeback**, **deposit** and **withdraw** are tested with unit tests in `./src/account/clinet_account.rs`
2. Input/Output format is tested in `./src/csv.rs`

## Safety and Robustness
1. In this crate I do not use any unsafe code on my part.
2. I run `cargo deny` to check for vulnerability
3. Errors are handled with [`thiserror`] crate in lib and [`anyhow`] in bin. In case of errors when applying transaction to account there are ignored as described in specification (but are defined).

## Efficiency
1. The input file can be large it's deserialized in chunks
2. The output is also write in chunks
3. Tasks are runs in tokio poll. So the program can read data in one thread and group them by client in second and apply operation per client in others etc.
4. The limitation is `group by client` task since it collects in memory all transactions that was in CSV file! This is sth that could possible be inefficient and cause drain all RAM.

## Maintainability
1. `#![deny(missing_docs)]` lint was used to make sure all necessary items was documented.
2. lint were checked with `cargo clippy` 
