//! # Accounts-state
//! Application read transactions from input csv file and print current account balances to output.
//!
//! ## Input format
//! csv with columns `type`, `client`, `tx`, `amount`
//!
//! ```csv
//!
//! ```

#![deny(missing_docs)]

use accounts_state::{
    account::ClientAcc,
    csv::{self, RawTransaction},
    ClientID,
};
use anyhow::Context;
use futures::StreamExt;
use std::{collections::BTreeMap, env};
use tokio::{
    io::{stdout, BufReader},
    spawn,
    sync::mpsc::{channel, Receiver, Sender},
};
use tokio_stream::wrappers::ReceiverStream;

async fn read_trans_from_file(
    filename: String,
    sender: Sender<RawTransaction>,
) -> anyhow::Result<()> {
    let f = tokio::fs::File::open(filename)
        .await
        .context("access input file")?;

    let bf = BufReader::new(f);
    let mut stream_records = csv::deserialize_transactions_from_csv_reader(bf)
        .await
        .context("improper content of file")?;

    while let Some(item) = stream_records.next().await {
        sender.send(item).await.unwrap();
    }
    Result::<(), anyhow::Error>::Ok(())
}

type GroupedRawTransaction = (ClientID, Vec<RawTransaction>);

async fn group_by_clinet(
    input: Receiver<RawTransaction>,
    output: Sender<GroupedRawTransaction>,
) -> anyhow::Result<()> {
    let mut input = input;
    let mut per_client = BTreeMap::new();

    while let Some(raw_trans) = input.recv().await {
        let client_transactions = per_client
            .entry(raw_trans.client)
            .or_insert_with(|| Vec::with_capacity(8192));
        client_transactions.push(raw_trans);
    }

    for client_id_and_transactions in per_client {
        if output.send(client_id_and_transactions).await.is_err() {
            todo!()
        }
    }
    Ok(())
}

async fn create_clinet_account(
    input: Vec<GroupedRawTransaction>,
    output: Sender<ClientAcc>,
) -> anyhow::Result<()> {
    for (client_id, client_raw_trans) in input {
        let mut client_acc = ClientAcc::new(client_id);
        for raw_trans in client_raw_trans {
            // TODO: not ignore errors
            let _ = client_acc.try_apply_new_raw_transaction(raw_trans);
        }
        if output.send(client_acc).await.is_err() {
            todo!();
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // CLI handle
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(anyhow::Error::msg("expected exactly one path to csv file"));
    }

    // read data from csv file
    let (tx_raw_trans, rx_raw_trans) = channel(8192);
    let task_read_csv = spawn(read_trans_from_file(args[1].clone(), tx_raw_trans));

    // group transactions by client identificator
    let (tx_tran_per_clinet, rx_trans_per_client) = channel(8192);
    let task_group_by_client = spawn(group_by_clinet(rx_raw_trans, tx_tran_per_clinet));

    // split clients into chunks
    let rx_trans_per_clinet = ReceiverStream::new(rx_trans_per_client);
    let mut chunked_raw_clinet_trans = rx_trans_per_clinet.chunks(64);

    // for each chunk of clients run task that will apply transactions to client
    // collect result of operation in one channel `rx_account`
    let (tx_account, rx_account) = channel(8192);
    let mut account_create_tasks = Vec::with_capacity(128);
    while let Some(chunked_trans) = chunked_raw_clinet_trans.next().await {
        let acc_create_task = spawn(create_clinet_account(chunked_trans, tx_account.clone()));
        account_create_tasks.push(acc_create_task);
    }
    drop(tx_account);

    // read clients from `rx_account` and write summary to stdout
    let task_output = spawn(csv::summarize_accounts(
        ReceiverStream::new(rx_account),
        stdout(),
    ));

    // awiat for each task to complete and handle it errors if occurred
    task_read_csv.await??;
    task_group_by_client.await??;
    for task in account_create_tasks {
        task.await??;
    }
    task_output.await?.context("failed to save output")?;

    Ok(())
}
