use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use chrono::NaiveDate;
use itertools::Itertools;

use crate::model::{DaySeriesData, Stock, StockMarket};

pub trait StockDataLoader {
    fn load() -> eyre::Result<Vec<Stock>>;
}

pub struct DefaultStockDataLoader {}

impl StockDataLoader for DefaultStockDataLoader {
    fn load() -> eyre::Result<Vec<Stock>> {
        Ok(vec![
            load_market(StockMarket::Kospi)?,
            load_market(StockMarket::Kosdaq)?,
            load_market(StockMarket::Nasdaq)?,
        ]
        .concat())
    }
}

pub struct KospiLoader {}

impl StockDataLoader for KospiLoader {
    fn load() -> eyre::Result<Vec<Stock>> {
        Ok(load_market(StockMarket::Kospi)?)
    }
}

fn load_market(market: StockMarket) -> eyre::Result<Vec<Stock>> {
    let (name, volume_position) = match market {
        StockMarket::Kospi => ("KOSPI", 5),
        StockMarket::Kosdaq => ("KOSDAQ", 5),
        StockMarket::Nasdaq => ("NASDAQ", 6),
        StockMarket::Nyse => todo!(),
    };

    let mut stocks: HashMap<String, Stock> =
        load_stocks(format!("./data/{name}.txt"), StockMarket::Kospi)?
            .into_iter()
            .map(|s| (s.code.clone(), s))
            .collect();
    let trades = fs::read_dir(format!("./data/{name}"))
        .unwrap()
        .collect_vec();

    for trade in trades {
        let trade = trade?;
        let code = trade.file_name().into_string().unwrap();

        stocks.get_mut(&code).unwrap().trades = load_stock_trades(trade.path(), volume_position)?;
    }

    Ok(stocks.into_iter().map(|(_, s)| s).collect())
}

fn load_stocks(path: impl AsRef<Path>, market: StockMarket) -> eyre::Result<Vec<Stock>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut stocks = vec![];

    for line in reader.lines().into_iter().skip(1) {
        let line = line?;
        let splits = line.split(',').collect_vec();

        stocks.push(Stock {
            market,
            code: splits[1].to_owned(),
            name: splits[2].to_owned(),
            ..Default::default()
        })
    }

    Ok(stocks)
}

fn load_stock_trades(
    path: impl AsRef<Path>,
    volume_position: usize,
) -> eyre::Result<BTreeMap<NaiveDate, DaySeriesData>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut trades = BTreeMap::new();

    for line in reader.lines().into_iter().skip(1) {
        let line = line?;
        let splits = line.split(',').collect_vec();

        // Filter by Nasdaq CRVO (2023-08-18~09-21)
        if splits[1].len() == 0 {
            continue;
        }

        trades.insert(
            NaiveDate::parse_from_str(splits[0], "%Y-%m-%d")?,
            DaySeriesData {
                open: splits[1].parse()?,
                high: splits[2].parse()?,
                low: splits[3].parse()?,
                close: splits[4].parse()?,
                volume: splits[volume_position].parse::<f64>()? as usize,
            },
        );
    }

    let trades = trades
        .into_iter()
        .filter(|(_, d)| d.open != 0f64 && d.close != 0f64)
        .collect();

    Ok(trades)
}

#[cfg(test)]
mod tests {
    use super::{DefaultStockDataLoader, StockDataLoader};

    #[test]
    fn unittest_default_stock_data_loader() -> eyre::Result<()> {
        DefaultStockDataLoader::load()?;
        Ok(())
    }
}
