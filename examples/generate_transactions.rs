use accounts_state::csv::{RawTransaction, RawTrasactionType};
use rand::{Rng, RngCore};
use rust_decimal::Decimal;

fn main() -> anyhow::Result<()> {
    let loop_end = 144_665_677;

    let b = csv::WriterBuilder::new();
    let mut wr = b.from_path("massive_output.csv")?;

    let mut rng = rand::thread_rng();
    for tran_id in 0..loop_end {
        let client_id = rng.next_u32() as u16;
        let trans_type = if rng.gen_bool(0.40) {
            RawTrasactionType::Withdrawal
        } else {
            RawTrasactionType::Deposit
        };

        let s = RawTransaction {
            client: client_id,
            r#type: trans_type,
            amount: Some(Decimal::new(11323243423, 4)),
            transaction_id: u32::try_from(tran_id).unwrap(),
        };

        wr.serialize(s).unwrap();
    }

    Ok(())
}
