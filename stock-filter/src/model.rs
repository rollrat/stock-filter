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

#[derive(Default, Debug, Copy, Clone)]
pub struct DaySeriesData {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: usize,
}
