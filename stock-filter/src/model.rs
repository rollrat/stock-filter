use std::collections::BTreeMap;

use chrono::NaiveDate;

#[derive(Default, Debug, Copy, Clone)]
pub enum StockMarket {
    #[default]
    Kospi,
    Kosdaq,
    Nasdaq,
    Nyse,
}

#[derive(Default, Debug, Clone)]
pub struct Stock {
    pub market: StockMarket,
    pub code: String,
    pub name: String,
    pub trades: BTreeMap<NaiveDate, DaySeriesData>,
}

pub type Price = f64;

#[derive(Default, Debug, Copy, Clone)]
pub struct DaySeriesData {
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: usize,
}
