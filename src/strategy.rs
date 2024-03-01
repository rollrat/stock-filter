use std::collections::BTreeMap;

use chrono::NaiveDate;
use moving_min_max::{MovingMax, MovingMin};

use crate::model::{DaySeriesData, Price};

pub trait Strategy {
    fn buy(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Price)>;
    fn sell(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Price)>;
}

/// buy: 현재 주가가 buy_move 일 최저가보다 작다
/// sell: 현재 주가가 sell_move 일 최고가보다 크다
pub struct NaiveStrategy {
    pub buy_move: usize,
    pub sell_move: usize,
}

impl Strategy for NaiveStrategy {
    fn buy(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Price)> {
        let mut slide = MovingMin::<Price>::new();
        let mut result = Vec::new();

        for (ix, (date, data)) in trades.iter().enumerate() {
            if ix < self.buy_move {
                slide.push(data.close);
                continue;
            }

            if data.open < *slide.min().unwrap() {
                result.push((*date, data.open));
            }

            slide.pop();
            slide.push(data.close);
        }

        result
    }

    fn sell(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Price)> {
        let mut slide = MovingMax::<Price>::new();
        let mut result = Vec::new();

        for (ix, (date, data)) in trades.iter().enumerate() {
            if ix < self.buy_move {
                slide.push(data.close);
                continue;
            }

            if *slide.max().unwrap() < data.open {
                result.push((*date, data.open));
            }

            slide.pop();
            slide.push(data.close);
        }

        result
    }
}

/// buy:
pub struct NaiveMovingAverageStrategy {}

/// buy: 전날 rise % 만큼 을랐다
/// sell: 없음
pub struct BeginningSurpriseStrategy {
    rise: f64,
}

pub struct StrategyEvaluator {
    strategy: Box<dyn Strategy>,
}

#[cfg(test)]
mod tests {
    use crate::{
        loader::{DefaultStockDataLoader, KospiLoader, StockDataLoader},
        strategy::{NaiveStrategy, Strategy},
    };

    #[test]
    fn unittest_naive_strategy() -> eyre::Result<()> {
        let stocks = KospiLoader::load()?;
        let stock = stocks.into_iter().find(|s| s.name == "삼성전자").unwrap();

        println!("code: {}", stock.code);

        let s = NaiveStrategy {
            buy_move: 20,
            sell_move: 20,
        };

        println!("buy: {:?}", s.buy(&stock.trades));
        println!("sell: {:?}", s.sell(&stock.trades));

        Ok(())
    }
}
