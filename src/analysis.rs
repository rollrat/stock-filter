use crate::strategy;

pub struct AnalysisStrategy {}

pub struct StockAnalyzer {}

impl StockAnalyzer {
    // pub fn evaluate (&self) -> {

    // }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::loader::{NasdaqLoader, StockDataLoader};

    #[test]
    fn 전날상한가종목_평균상승률() -> eyre::Result<()> {
        let stocks = NasdaqLoader::load()?;
        let 상한가_threashold = 1.0;

        let mut 상한가종목_날짜 = Vec::new();

        println!("load complete");

        for stock in stocks {
            let mut 상한가_dates = Vec::new();
            for (prev, next) in stock.trades.iter().tuple_windows() {
                let p = (next.1.close - prev.1.close) / prev.1.close;

                if p >= 상한가_threashold {
                    상한가_dates.push(*next.0);
                }
            }

            // if 상한가_dates.len() > 0 {
            //     println!("{}, {}, {:?}", stock.name, 상한가_dates.len(), 상한가_dates);
            // }

            상한가종목_날짜.push((stock.name, 상한가_dates));
        }

        상한가종목_날짜.sort_by_key(|(_, dates)| dates.len());

        for (name, dates) in 상한가종목_날짜.iter().rev().take(30) {
            println!("{name}, {}, {:?}", dates.len(), dates);
        }

        Ok(())
    }

    #[test]
    fn 전날상한가종목_P이상상승률_종목수() {}
}
