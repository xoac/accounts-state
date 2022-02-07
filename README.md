## Assumptions

1. If account is frozen/locked, all withdraw and deposit transactions will fail. Transactions will be
   recorded, and account will still accept to make other disputes (but only for valid money
   transactions). Chargebacks and resolves operations wouldn't be possible until account is unlocked.
2. **amount** is a positive number that represents money.
3. The dispute operation (and because of that resolve and chargeback) behaves differently for withdraw and deposit operation. The specification was inconsistent in this case because it required:
    1. preparation for reversing a transaction
    2. increase of `held` founds by **`amount`** and decrease `avaliable` founds by **`amount`**
4. Format on input csv file is correct as described. For example the application will panic if there will be no amount for withdraw or deposit and ignore amount for dispute,resolve,chargeback even if present

When withdraw transaction will be disputed and chargebacked and we implement solution described in `3.2` then the account's balance will have incorrect balance (but hopefully account is also frozen). The total money in the account was decreased two times (first time with withdraw and second time with chargeback). This is why I implemented `3.1`.

## Correctnes

1. The basic cases like **dispute**, **resolve**, **chargeback**, **deposit** and **withdraw** are tested with unit tests in `./src/account/clinet_account.rs`
2. Input/Output format is tested in `./src/csv.rs`
3. Example massive csv file around ~5GB was generated with `./examples/generate_transactions.rs` and manually tested.

## Safety and Robustness

1. In this crate I do not use any unsafe code on my part.
2. I checked with  `cargo deny` for vulnerability
3. Errors are handled with [`thiserror`] crate in lib and [`anyhow`] in bin. In case of errors when applying transaction to account, these errors are ignored as described in specification (but are defined).

## Efficiency

1. The input file can be large because it's deserialized in chunks
2. The output is also written in chunks
3. Tasks are run in **tokio poll**. So the program can read data in one thread and group them by client in second and apply operation per client in others etc.
4. The limitation exists in `group by client` task because it collects in memory all transactions that was in CSV file! This is sth that could possible be inefficient and cause drain all RAM when input file is very large.
5. link time optimization is enabled on release build
6. `CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph` with data generated with `examples/generate_transactions.rs` can be used to further diagnosing.

## Maintainability

1. `#![deny(missing_docs)]` lint was used to make sure all necessary items were documented.
2. lints were checked with `cargo clippy` 
