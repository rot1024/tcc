use chrono::NaiveDate;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;

// https://www8.cao.go.jp/chosei/shukujitsu/syukujitsu.csv
const DATA: &[u8] = include_bytes!("syukujitsu.csv");

#[derive(Debug, Deserialize)]
struct Holiday {
    #[serde(rename = "国民の祝日・休日月日")]
    date: NaiveDate,
    #[serde(rename = "国民の祝日・休日名称")]
    name: String,
}

lazy_static! {
    pub static ref HOLIDAYS: HashMap<NaiveDate, String> = csv::ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .from_reader(DATA)
        .deserialize::<Holiday>()
        .into_iter()
        .filter_map(|h| h.ok().map(|h2| (h2.date, h2.name)))
        .collect();
}
