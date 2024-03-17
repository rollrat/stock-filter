use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum StockMarket {
    #[default]
    Kospi,
    Kosdaq,
    Nasdaq,
    Nyse,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub market: StockMarket,
    pub code: String,
    pub name: String,
    pub trades: BTreeMap<NaiveDate, DaySeriesData>,
}

pub type Price = f64;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct DaySeriesData {
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: usize,
}
