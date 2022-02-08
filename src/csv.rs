//! csv input/output format and functions

use csv_async::{AsyncWriterBuilder, Terminator};
use rust_decimal::Decimal;

use crate::{
    account::{self, Account},
    ClientID, TransID,
};
use serde::{Deserialize, Serialize};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc::Sender,
};
use tokio_stream::{Stream, StreamExt};

// Allowed transaction types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs)]
pub enum RawTrasactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct RawTransaction {
    pub r#type: RawTrasactionType,
    #[serde(rename = "client")]
    pub client: ClientID,
    #[serde(rename = "tx")]
    pub transaction_id: TransID,
    pub amount: Option<Decimal>,
}

/// take a reader and continuously deserialize item from it into returned Stream
pub async fn deserialize_transactions_from_csv_reader<'r, R: AsyncRead + Unpin + Send + 'r>(
    input: R,
    sender: Sender<RawTransaction>,
) -> anyhow::Result<()> {
    let mut builder = csv_async::AsyncReaderBuilder::new();
    builder.trim(csv_async::Trim::All);

    let mut rdr = builder.create_deserializer(input);

    let _headers = rdr.headers().await?;

    let mut records = rdr.deserialize::<RawTransaction>();
    while let Some(record) = records.next().await {
        let record: RawTransaction = record?;
        sender.send(record).await?;
    }

    Ok(())
}

/// summary account balance and state for a client
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct AccountSummary {
    pub client: ClientID,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl<T: Account> From<T> for AccountSummary {
    fn from(oth: T) -> Self {
        let rp = 4; // round precision
        Self {
            client: oth.client_id(),
            available: oth.available().round_dp(rp),
            held: oth.held().round_dp(rp),
            total: oth.total().round_dp(rp),
            locked: oth.is_locked(),
        }
    }
}

/// read items from `in_stream` and save them as [`AccountSummary`] into `wr`. Headers will be
/// included automatically. Terminator is `\n` not `\n\r` as sp
pub async fn summarize_accounts(
    in_stream: impl Stream<Item = impl account::Account> + Unpin,
    wr: impl AsyncWrite + Unpin,
) -> anyhow::Result<()> {
    let mut in_stream = in_stream;

    let mut builder = AsyncWriterBuilder::new();
    builder.terminator(Terminator::CRLF);

    let mut wr = builder.create_serializer(wr);

    while let Some(acc) = in_stream.next().await {
        let acc_summary = AccountSummary::from(acc);
        wr.serialize(acc_summary).await?;
    }

    wr.flush().await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::{
        deserialize_transactions_from_csv_reader, summarize_accounts, RawTransaction,
        RawTrasactionType,
    };
    use crate::account::ClientAcc;
    use rust_decimal::Decimal;
    use tokio::sync::mpsc::channel;
    use tokio_stream::{wrappers::ReceiverStream, StreamExt};

    #[tokio::test]
    async fn ser_output_format() -> anyhow::Result<()> {
        let in_stream = tokio_stream::iter(vec![ClientAcc::new_test_account()]);
        let output = Vec::with_capacity(8192);

        let mut wr = tokio::io::BufWriter::new(output);

        summarize_accounts(in_stream, &mut wr).await?;

        let expected = "client,available,held,total,locked\r\n1,750,0.0000,750,false\r\n";

        let output_str = String::from_utf8(wr.into_inner()).unwrap();

        assert_eq!(output_str, expected);

        Ok(())
    }

    #[tokio::test]
    async fn des_input_fozmat() -> anyhow::Result<()> {
        let raw_in1 = r#"type, client, tx, amount
deposit, 1, 1, 1
deposit, 2, 2, 2.0
deposit, 1, 3,
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0"#;

        let (tx1, rx1) = channel(2);
        tokio::spawn(async move {
            deserialize_transactions_from_csv_reader(Vec::from(raw_in1).as_ref(), tx1).await
        });

        let in1_vec: Vec<RawTransaction> = ReceiverStream::new(rx1).collect().await;
        assert_eq!(
            in1_vec[2],
            RawTransaction {
                r#type: RawTrasactionType::Deposit,
                client: 1,
                transaction_id: 3,
                amount: None
            }
        );

        let raw_in2 = r#"type,client,tx,amount
deposit,1,1,1
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,3.0"#;

        let (tx2, rx2) = channel(2);

        tokio::spawn(async move {
            deserialize_transactions_from_csv_reader(raw_in2.as_ref(), tx2).await
        });
        let in2_vec: Vec<RawTransaction> = ReceiverStream::new(rx2).collect().await;
        assert_eq!(
            in2_vec[3],
            RawTransaction {
                r#type: RawTrasactionType::Withdrawal,
                client: 1,
                transaction_id: 4,
                amount: Some(Decimal::new(15, 1))
            }
        );

        Ok(())
    }
}
