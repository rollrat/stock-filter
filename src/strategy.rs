use std::{borrow::BorrowMut, collections::BTreeMap};

use chrono::NaiveDate;
use itertools::Itertools;
use moving_min_max::{MovingMax, MovingMin};

use crate::model::{DaySeriesData, Price};

pub trait Strategy {
    fn buy(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Price)>;
    fn sell(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Price)>;
}

pub struct NeverSellStrategy<T: Strategy> {
    pub strategy: T,
}

impl<T: Strategy> Strategy for NeverSellStrategy<T> {
    fn buy(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Price)> {
        self.strategy.buy(trades)
    }

    fn sell(&self, _: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Price)> {
        Vec::new()
    }
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

#[derive(Debug)]
pub struct StrategyEvaluatorConfig {
    buy_factor: usize,
    sell_factor: f64,
    allow_consecutive_buy: bool,
    show_steps: bool,
}

impl Default for StrategyEvaluatorConfig {
    fn default() -> Self {
        Self {
            buy_factor: 1,
            sell_factor: 1.0,
            allow_consecutive_buy: true,
            show_steps: false,
        }
    }
}

impl StrategyEvaluatorConfig {
    pub fn with_allow_consecutive_buy(mut self, value: bool) -> Self {
        self.allow_consecutive_buy = value;
        self
    }

    pub fn with_show_steps(mut self, value: bool) -> Self {
        self.show_steps = value;
        self
    }
}

pub struct StrategyEvaluator {
    config: StrategyEvaluatorConfig,
}

#[derive(Debug, Copy, Clone)]
pub struct StrategyEvaluatorResult {
    stock: usize,
    trading: usize,
    balance: f64,
    invest: f64,
    income: f64,
    roi: f64,
}

impl StrategyEvaluator {
    pub fn evaluate<T>(
        &self,
        strategy: T,
        trades: &BTreeMap<NaiveDate, DaySeriesData>,
    ) -> StrategyEvaluatorResult
    where
        T: Strategy,
    {
        let buys = strategy.buy(trades);
        let sells = strategy.sell(trades);

        let mut deltas = vec![
            buys,
            sells
                .into_iter()
                .map(|(date, price)| (date, -price))
                .collect(),
        ]
        .concat()
        .into_iter()
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .collect_vec();

        if !self.config.allow_consecutive_buy {
            deltas = deltas
                .into_iter()
                .dedup_by(|(_, lprice), (_, rprice)| {
                    (*lprice > 0.0 && *rprice > 0.0) || (*lprice <= 0.0 && *rprice <= 0.0)
                })
                .collect();
        }

        let (first_buy, _) = deltas
            .iter()
            .find_position(|(_, price)| *price > 0f64)
            .unwrap();

        let mut stock = 0;
        let mut trading = 0;
        let mut balance = 0f64;

        let mut invest = 0f64;
        let mut income = 0f64;

        for (date, price) in deltas.into_iter().skip(first_buy) {
            if price > 0.0 {
                let buy_stock = self.config.buy_factor;
                invest += price * buy_stock as f64;
                balance -= price * buy_stock as f64;
                stock += buy_stock;
                trading += buy_stock;

                if self.config.show_steps {
                    println!("{date} buy  {price}: {buy_stock}, {balance}");
                }
            } else if stock > 0 {
                let sell_stock = stock as f64 * self.config.sell_factor;
                income -= price * sell_stock;
                balance -= price * sell_stock;
                trading += sell_stock as usize;
                stock -= sell_stock as usize;

                if self.config.show_steps {
                    println!("{date} sell {}: {}, {balance}", -price, sell_stock as usize);
                }
            }
        }

        StrategyEvaluatorResult {
            stock,
            trading,
            balance: balance + stock as f64 * trades.last_key_value().unwrap().1.close,
            invest,
            income,
            roi: (income + stock as f64 * trades.last_key_value().unwrap().1.close) / invest,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        loader::{KospiLoader, StockDataLoader},
        strategy::{NaiveStrategy, NeverSellStrategy, StrategyEvaluator, StrategyEvaluatorConfig},
    };

    #[test]
    fn unittest_naive_strategy() -> eyre::Result<()> {
        let stocks = KospiLoader::load()?;
        let stock = stocks.into_iter().find(|s| s.name == "SK하이닉스").unwrap();

        println!("code: {}", stock.code);

        let strategy = NaiveStrategy {
            buy_move: 20,
            sell_move: 20,
        };
        // let strategy = NeverSellStrategy { strategy };

        let config = StrategyEvaluatorConfig::default()
            .with_allow_consecutive_buy(false)
            .with_show_steps(true);
        let r = StrategyEvaluator { config }.evaluate(strategy, &stock.trades);

        println!("result: {r:#?}");

        Ok(())
    }
}
