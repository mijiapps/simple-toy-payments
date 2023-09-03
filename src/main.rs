mod utils;
mod data;

use std::env;
use std::error::Error;
use std::ffi::OsString;
use crate::utils::csv_utils::{create_reader, parse_csv_data, write_csv_data};

fn main() {
    let file_path = get_first_arg().expect("Requires path to the transaction CSV: cargo run -- test_data/transactions_comma.csv");
    let mut reader = create_reader(file_path).expect("Could not open file");
    let accounts = parse_csv_data(&mut reader).expect("Error parsing csv data");
    write_csv_data(*accounts).expect("Error writing csv data");
}

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}
