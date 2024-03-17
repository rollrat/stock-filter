use std::{
    borrow::BorrowMut,
    cmp::max,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
};

use chrono::NaiveDate;
use itertools::Itertools;
use moving_min_max::{MovingMax, MovingMin};
use std::ops::Bound::{Included, Unbounded};

use crate::{
    model::{DaySeriesData, Price, Stock},
    utils::MovingAverage,
};

#[derive(Debug, Copy, Clone)]
pub enum Action {
    Buy(Price),
    Sell(Price),
}

impl Action {
    pub fn is_buy(&self) -> bool {
        match self {
            Action::Buy(_) => true,
            _ => false,
        }
    }

    pub fn is_sell(&self) -> bool {
        match self {
            Action::Sell(_) => true,
            _ => false,
        }
    }
}

pub trait BuySellStrategy {
    fn buy(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Action)>;
    fn sell(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Action)>;

    fn buy_sell(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Action)> {
        let buys = self.buy(trades);
        let sells = self.sell(trades);

        let deltas = vec![buys, sells]
            .concat()
            .into_iter()
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .collect_vec();

        deltas
    }
}

/// buy: 현재 주가가 buy_move 일 최저가보다 작다
/// sell: 현재 주가가 sell_move 일 최고가보다 크다
pub struct NaiveStrategy {
    pub buy_move: usize,
    pub sell_move: usize,
}

impl BuySellStrategy for NaiveStrategy {
    fn buy(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Action)> {
        let mut slide = MovingMin::<Price>::new();
        let mut result = Vec::new();

        for (ix, (date, data)) in trades.iter().enumerate() {
            if ix < self.buy_move {
                slide.push(data.close);
                continue;
            }

            if data.open < *slide.min().unwrap() {
                result.push((*date, Action::Buy(data.open)));
            }

            slide.pop();
            slide.push(data.close);
        }

        result
    }

    fn sell(&self, trades: &BTreeMap<NaiveDate, DaySeriesData>) -> Vec<(NaiveDate, Action)> {
        let mut slide = MovingMax::<Price>::new();
        let mut result = Vec::new();

        for (ix, (date, data)) in trades.iter().enumerate() {
            if ix < self.buy_move {
                slide.push(data.close);
                continue;
            }

            if *slide.max().unwrap() < data.open {
                result.push((*date, Action::Sell(data.open)));
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

pub trait FoldStrategy
where
    Self: 'static,
{
    fn fold(
        &self,
        actions: Vec<(NaiveDate, Action)>,
        trades: &BTreeMap<NaiveDate, DaySeriesData>,
    ) -> Vec<(NaiveDate, Action)>;

    fn boxed(self) -> Box<dyn FoldStrategy>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

pub struct NeverSellStrategy {}

impl FoldStrategy for NeverSellStrategy {
    fn fold(
        &self,
        actions: Vec<(NaiveDate, Action)>,
        _: &BTreeMap<NaiveDate, DaySeriesData>,
    ) -> Vec<(NaiveDate, Action)> {
        actions
            .into_iter()
            .filter(|(_, act)| !act.is_sell())
            .collect()
    }
}

pub struct ConsecutiveBuyRemover {}

impl FoldStrategy for ConsecutiveBuyRemover {
    fn fold(
        &self,
        actions: Vec<(NaiveDate, Action)>,
        _: &BTreeMap<NaiveDate, DaySeriesData>,
    ) -> Vec<(NaiveDate, Action)> {
        actions
            .into_iter()
            .dedup_by(|(_, lact), (_, ract)| {
                (lact.is_buy() && ract.is_buy()) || (lact.is_sell() && ract.is_sell())
            })
            .collect()
    }
}

pub struct LossSellRemover {}

impl FoldStrategy for LossSellRemover {
    fn fold(
        &self,
        actions: Vec<(NaiveDate, Action)>,
        _: &BTreeMap<NaiveDate, DaySeriesData>,
    ) -> Vec<(NaiveDate, Action)> {
        let mut result = Vec::new();
        let mut max_buy_price = 0f64;

        for action in actions {
            match action.1 {
                Action::Buy(price) => {
                    max_buy_price = max_buy_price.max(price);
                    result.push(action);
                }
                Action::Sell(price) => {
                    if max_buy_price < price {
                        result.push(action);
                        max_buy_price = 0f64;
                    }
                }
            }
        }

        result
    }
}

#[derive(Debug)]
pub struct StrategyEvaluatorConfig {
    buy_factor: usize,
    sell_factor: f64,
    stoploss: Option<f64>,
    show_steps: bool,
}

impl Default for StrategyEvaluatorConfig {
    fn default() -> Self {
        Self {
            buy_factor: 1,
            sell_factor: 1.0,
            stoploss: None,
            show_steps: false,
        }
    }
}

impl StrategyEvaluatorConfig {
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
        folders: Vec<Box<dyn FoldStrategy>>,
        trades: &BTreeMap<NaiveDate, DaySeriesData>,
    ) -> StrategyEvaluatorResult
    where
        T: BuySellStrategy,
    {
        let actions = folders
            .into_iter()
            .fold(strategy.buy_sell(trades), |actions, folder| {
                folder.fold(actions, &trades)
            });

        let (first_buy, _) = actions
            .iter()
            .find_position(|(_, act)| act.is_buy())
            .unwrap();

        let mut stock = 0;
        let mut trading = 0;
        let mut balance = 0f64;

        let mut invest = 0f64;
        let mut income = 0f64;

        let mut avg = MovingAverage::default();

        let sells: BTreeSet<NaiveDate> = actions
            .iter()
            .filter(|(_, act)| act.is_sell())
            .map(|(date, _)| *date)
            .collect();

        for (date, act) in actions.into_iter().skip(first_buy) {
            // println!("{}", avg.avg());
            match act {
                Action::Buy(price) => {
                    let buy_stock = self.config.buy_factor;
                    invest += price * buy_stock as f64;
                    balance -= price * buy_stock as f64;
                    stock += buy_stock;
                    trading += buy_stock;
                    avg.feed(price, buy_stock);

                    if self.config.show_steps {
                        println!("{date} buy  {price}: {buy_stock}, {balance}");
                    }
                }
                Action::Sell(price) => {
                    if stock != 0 {
                        let sell_stock = stock as f64 * self.config.sell_factor;
                        income += price * sell_stock;
                        balance += price * sell_stock;
                        trading += sell_stock as usize;
                        stock -= sell_stock as usize;
                        avg.feed(-price, sell_stock as usize);

                        if self.config.show_steps {
                            println!("{date} sell {price}: {}, {balance}", sell_stock as usize);
                        }
                    }
                }
            }

            if let Some(stoploss) = self.config.stoploss {
                let next_sell = sells.range((Included(&date), Unbounded)).next();
                todo!();
                // if avg.avg() < -stoploss {}
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

//

pub struct Account {
    balance: Price,
    stocks: HashMap<String, usize>,
}

pub struct StockInfo<'a> {
    code: String,
    past_trades: &'a BTreeMap<NaiveDate, DaySeriesData>,
}

pub trait LinearBuySellStrategy {
    fn buy(&self, account: &Account, stock: &StockInfo) -> bool;
}

pub struct BackTester {}

impl BackTester {}

#[cfg(test)]
mod tests {
    use crate::{
        loader::{KospiLoader, StockDataLoader},
        strategy::{
            ConsecutiveBuyRemover, FoldStrategy, LossSellRemover, NaiveStrategy, NeverSellStrategy,
            StrategyEvaluator, StrategyEvaluatorConfig,
        },
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

        let folder = vec![
            ConsecutiveBuyRemover {}.boxed(),
            LossSellRemover {}.boxed(),
            // NeverSellStrategy {}.boxed(),
        ];

        let r = StrategyEvaluator {
            config: StrategyEvaluatorConfig::default().with_show_steps(true),
        }
        .evaluate(strategy, folder, &stock.trades);

        println!("result: {r:#?}");

        Ok(())
    }
}
