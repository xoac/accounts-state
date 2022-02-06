use std::{collections::BTreeMap, fs::File, io::Read};

use accounts_state::{csv::AccountSummary, ClientID};
use assert_cmd::Command;

fn executable() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

fn read_summary(reader: impl Read) -> anyhow::Result<BTreeMap<ClientID, AccountSummary>> {
    let mut b = csv::ReaderBuilder::new();
    b.trim(csv::Trim::All);
    let mut rdr = b.from_reader(reader);

    let mut map = BTreeMap::new();
    for acc_summ in rdr.deserialize() {
        let acc_summ: AccountSummary = acc_summ?;

        assert!(map.insert(acc_summ.client, acc_summ).is_none());
    }

    Ok(map)
}

#[test]
fn system_test() {
    let tests = vec![1, 2];

    for test_no in tests {
        let in_file = format!("./tests/csvs/in{test_no}.csv");
        let out_file = format!("./tests/csvs/out{test_no}.csv");

        let out = executable()
            .arg(in_file)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        //deserialize_output
        let out = read_summary(out.as_slice()).unwrap();
        let exp = read_summary(File::open(out_file).unwrap()).unwrap();
        assert_eq!(out, exp);
    }
}
